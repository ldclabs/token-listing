use candid::{CandidType, Principal};
use ic_auth_types::{ByteArrayB64, ByteBufB64, deterministic_cbor_into_vec};
use ic_ed25519::PublicKey;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

use crate::helper;

const TIME_EXPIRY_MS: u64 = 600 * 1000; // 10 minutes

#[derive(Clone, Serialize, Deserialize)]
pub struct X402State {
    pub canister: Principal,
    pub user_nonce: HashMap<Principal, u64>,
    pub paying_public_keys: Vec<ByteArrayB64<32>>,
    pub payment_requirements_extra: Option<Map<String, Value>>,
}

impl X402State {
    pub fn get_x402_exact_payment(
        &self,
        user: &Principal,
        network: String,
        asset: String,
        amount: u128,
        pay_to: String,
        now_ms: u64,
        resource: ResourceInfo,
        error_msg: Option<String>,
    ) -> Result<X402PaymentOutput, String> {
        let pr = PaymentRequirements {
            scheme: "exact".to_string(),
            network,
            amount: amount.to_string(),
            asset,
            pay_to,
            max_timeout_seconds: 120,
            extra: self.payment_requirements_extra.clone(),
        };

        let nonce_seed = deterministic_cbor_into_vec(&(
            self.canister.as_slice(),
            user.as_slice(),
            &pr,
            now_ms,
            self.user_nonce.get(user).copied().unwrap_or_default(),
        ))?;
        let nonce = ByteBufB64::from(helper::sha3_256(&nonce_seed)).to_string();
        let x402 = PaymentRequired {
            x402_version: 2,
            error: error_msg,
            resource,
            accepts: vec![pr],
            extensions: None,
        };

        Ok(X402PaymentOutput {
            x402: ByteBufB64::from(deterministic_cbor_into_vec(&x402)?),
            nonce,
            timestamp: now_ms,
        })
    }

    pub fn verify_response(
        &mut self,
        input: PayingResultInput,
        user: Principal,
        asset: &str,
        amount: u128,
        pay_to: &str,
        now_ms: u64,
    ) -> Result<PaymentVerifyResult, String> {
        let mut verified = false;
        for pk_bytes in &self.paying_public_keys {
            let pk = PublicKey::deserialize_raw(&pk_bytes.0).map_err(helper::format_error)?;
            if pk.verify_signature(&input.result, &input.signature).is_ok() {
                verified = true;
                break;
            }
        }

        if !verified {
            return Err("signature verification failed".to_string());
        }
        let pv: PaymentVerifyResult =
            ciborium::from_reader(&input.result[..]).map_err(helper::format_error)?;
        if pv.payment_requirements.asset != asset
            || pv.payment_requirements.amount != amount.to_string()
            || pv.payment_requirements.pay_to != pay_to
        {
            return Err("payment requirements mismatch".to_string());
        }

        if input.timestamp + TIME_EXPIRY_MS < now_ms {
            return Err("payment verification result expired".to_string());
        }
        if !pv.verify_response.is_valid {
            return Err(format!(
                "payment verification failed: {}",
                pv.verify_response
                    .invalid_reason
                    .unwrap_or("unknown reason".to_string())
            ));
        }

        let nonce_seed = deterministic_cbor_into_vec(&(
            self.canister.as_slice(),
            user.as_slice(),
            &pv.payment_requirements,
            input.timestamp,
            self.user_nonce.get(&user).copied().unwrap_or_default(),
        ))?;
        let nonce = ByteBufB64::from(helper::sha3_256(&nonce_seed)).to_string();
        if nonce != pv.nonce {
            return Err("nonce mismatch".to_string());
        }
        self.user_nonce
            .entry(user)
            .and_modify(|e| *e += 1)
            .or_insert(1);
        Ok(pv)
    }

    pub fn settle_response(
        &mut self,
        input: PayingResultInput,
        user: Principal,
        asset: &str,
        amount: u128,
        pay_to: &str,
        now_ms: u64,
    ) -> Result<PaymentSettleResult, String> {
        let mut verified = false;
        for pk_bytes in &self.paying_public_keys {
            let pk = PublicKey::deserialize_raw(&pk_bytes.0).map_err(helper::format_error)?;
            if pk.verify_signature(&input.result, &input.signature).is_ok() {
                verified = true;
                break;
            }
        }

        if !verified {
            return Err("signature verification failed".to_string());
        }
        let pv: PaymentSettleResult =
            ciborium::from_reader(&input.result[..]).map_err(helper::format_error)?;
        if pv.payment_requirements.asset != asset
            || pv.payment_requirements.amount != amount.to_string()
            || pv.payment_requirements.pay_to != pay_to
        {
            return Err("payment requirements mismatch".to_string());
        }

        if input.timestamp + TIME_EXPIRY_MS < now_ms {
            return Err("payment settlement result expired".to_string());
        }
        if !pv.settle_response.success {
            return Err(format!(
                "payment settlement failed: {}",
                pv.settle_response
                    .error_reason
                    .unwrap_or("unknown reason".to_string())
            ));
        }

        let nonce_seed = deterministic_cbor_into_vec(&(
            self.canister.as_slice(),
            user.as_slice(),
            &pv.payment_requirements,
            input.timestamp,
            self.user_nonce.get(&user).copied().unwrap_or_default(),
        ))?;
        let nonce = ByteBufB64::from(helper::sha3_256(&nonce_seed)).to_string();
        if nonce != pv.nonce {
            return Err("nonce mismatch".to_string());
        }
        self.user_nonce
            .entry(user)
            .and_modify(|e| *e += 1)
            .or_insert(1);
        Ok(pv)
    }
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct X402PaymentOutput {
    pub x402: ByteBufB64, // PaymentRequirementsResponse in CBOR
    pub nonce: String,
    pub timestamp: u64,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
pub struct PayingResultInput {
    pub result: ByteBufB64, // PaymentVerifyResult / PaymentSettleResult in CBOR,
    pub signature: ByteBufB64, // Signature over (result || nonce || timestamp)
    pub timestamp: u64,     // the timestamp from X402PaymentOutput
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentVerifyResult {
    pub payment_requirements: PaymentRequirements,
    pub verify_response: VerifyResponse,
    pub nonce: String, // the nonce from X402PaymentOutput
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSettleResult {
    pub payment_requirements: PaymentRequirements,
    pub settle_response: SettleResponse,
    pub nonce: String, // the nonce from X402PaymentOutput
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResponse {
    pub is_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<String>,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
    pub transaction: String,
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequired {
    /// Protocol version identifier
    pub x402_version: u8,
    /// Human-readable error message explaining why payment is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// ResourceInfo object describing the protected resource
    pub resource: ResourceInfo,
    /// Array of payment requirement objects defining acceptable payment methods
    pub accepts: Vec<PaymentRequirements>,
    /// Protocol extensions data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Payment requirements set by the payment-gated endpoint for an acceptable payment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirements {
    /// Payment scheme identifier (e.g., "exact")
    pub scheme: String,
    /// Blockchain network identifier (e.g., "icp")
    pub network: String,
    /// Required payment amount in atomic token units
    pub amount: String,
    /// Token ledger canister address
    pub asset: String,
    /// Recipient wallet address for the payment
    pub pay_to: String,
    /// Maximum time allowed for payment completion in seconds
    pub max_timeout_seconds: u64,
    /// Scheme-specific additional information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Map<String, Value>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceInfo {
    /// the protected resource, e.g., URL of the resource endpoint
    pub url: String,
    /// Human-readable description of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the expected response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Describes additional extension data for x402 payment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Extension-specific data provided by the server
    pub info: Map<String, Value>,
    /// JSON Schema defining the expected structure of `info`
    pub schema: Map<String, Value>,
}
