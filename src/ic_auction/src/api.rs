use ic_auth_types::{ByteBufB64, deterministic_cbor_into_vec};
use ic_ed25519::PublicKey;

use crate::{
    helper::{format_error, msg_caller, sha3_256},
    store, types,
};

#[ic_cdk::query]
fn info() -> Result<types::StateInfo, String> {
    Ok(store::state::info())
}

#[ic_cdk::query]
fn auction_info() -> Option<types::AuctionInfo> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::auction_info(now_ms)
}

#[ic_cdk::query]
fn get_grouped_bids(precision: Option<u64>) -> Vec<(u128, u128)> {
    store::state::get_grouped_bids(precision.unwrap_or(10) as u128)
}

#[ic_cdk::query]
fn get_snapshots(from_timestamp: u64, take: usize) -> Vec<types::AuctionSnapshot> {
    store::state::with(|s| {
        s.snapshots
            .iter()
            .filter(|v| v.timestamp >= from_timestamp)
            .take(take.min(1000))
            .cloned()
            .collect()
    })
}

#[ic_cdk::query]
fn my_info() -> Result<types::UserInfo, String> {
    let caller = msg_caller()?;
    store::state::my_info(caller)
}

#[ic_cdk::query]
fn my_bids() -> Result<Vec<types::BidInfo>, String> {
    let caller = msg_caller()?;
    store::state::my_bids(caller)
}

#[ic_cdk::query]
fn my_deposits() -> Result<Vec<types::DepositTxInfo>, String> {
    let caller = msg_caller()?;
    store::state::my_deposits(caller)
}

#[ic_cdk::query]
fn my_withdraws() -> Result<Vec<types::WithdrawTxInfo>, String> {
    let caller = msg_caller()?;
    store::state::my_withdraws(caller)
}

#[ic_cdk::query]
fn estimate_max_price(amount: u128) -> (u128, u128) {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::estimate_max_price(amount, now_ms)
}

#[ic_cdk::query]
fn x402_payment(amount: u128, verify_only: bool) -> Result<types::X402PaymentOutput, String> {
    let caller = msg_caller()?;
    let timestamp = ic_cdk::api::time() / 1_000_000;
    store::state::with(|s| {
        let amount = if verify_only {
            amount.min(10u128.pow(s.currency_decimals.saturating_sub(2) as u32))
        } else {
            amount
        };

        let (network, pay_to) = match &s.chain {
            types::Chain::Icp(1) => ("icp:1".to_string(), s.icp_address.to_string()),
            types::Chain::Sol(0) => (
                "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1sK6u9hC4BXj".to_string(),
                s.sol_address.to_string(),
            ),
            types::Chain::Sol(1) => (
                "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                s.sol_address.to_string(),
            ),
            types::Chain::Evm(8453) => ("eip155:8453".to_string(), s.evm_address.to_string()),
            types::Chain::Evm(84532) => ("eip155:8453".to_string(), s.evm_address.to_string()),
            other => return Err(format!("{:?} is not supported yet", other)),
        };

        let pr = types::PaymentRequirements {
            scheme: "exact".to_string(),
            network,
            amount: amount.to_string(),
            asset: s.currency.to_string(),
            pay_to,
            max_timeout_seconds: 120,
            extra: None,
        };
        let nonce_seed =
            deterministic_cbor_into_vec(&(verify_only, &caller, timestamp, &pr, &s.nonce_iv))?;
        let nonce = ByteBufB64::from(sha3_256(&nonce_seed)).to_string();
        let x402 = types::PaymentRequired {
            x402_version: 2,
            error: if verify_only {
                Some(format!("Verification required for auction: {:?}", s.name))
            } else {
                Some(format!("Payment required for auction: {:?}", s.name))
            },
            resource: types::ResourceInfo {
                url: format!("https://tokenlist.ing/_/launchpad/{}", s.icp_address),
                description: if verify_only {
                    Some("Address verification only, no settlement will be made.".to_string())
                } else {
                    Some("Auction bid payment".to_string())
                },
                mime_type: None,
            },
            accepts: vec![pr],
            extensions: None,
        };

        Ok(types::X402PaymentOutput {
            x402: ByteBufB64::from(deterministic_cbor_into_vec(&x402)?),
            nonce,
            timestamp,
        })
    })
}

#[ic_cdk::update]
fn x402_bind_address(input: types::PayingResultInput) -> Result<(), String> {
    let caller = msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;

    store::state::with(|s| {
        let mut verified = false;
        for pk_bytes in &s.paying_public_keys {
            let pk = PublicKey::deserialize_raw(&pk_bytes.0).map_err(format_error)?;
            if pk.verify_signature(&input.result, &input.signature).is_ok() {
                verified = true;
                break;
            }
        }
        if !verified {
            return Err("signature verification failed".to_string());
        }
        let pv: types::PaymentVerifyResult =
            ciborium::from_reader(&input.result[..]).map_err(format_error)?;
        if input.timestamp + 600 * 1000 < now_ms {
            return Err("payment verification result expired".to_string());
        }
        if !pv.verify_response.is_valid {
            return Err("payment verification failed".to_string());
        }

        let payer = pv
            .verify_response
            .payer
            .ok_or("missing payer in verification response")?;

        let nonce_seed = deterministic_cbor_into_vec(&(
            true,
            &caller,
            input.timestamp,
            &pv.payment_requirements,
            &s.nonce_iv,
        ))?;
        let nonce = ByteBufB64::from(sha3_256(&nonce_seed)).to_string();
        if nonce != pv.nonce {
            return Err("nonce mismatch".to_string());
        }

        store::state::bind_address(caller, payer, now_ms)?;

        Ok(())
    })
}

#[ic_cdk::update]
async fn x402_deposit_currency(input: types::PayingResultInput) -> Result<u128, String> {
    let caller = msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    let settle_response = store::state::with(|s| {
        let mut verified = false;
        for pk_bytes in &s.paying_public_keys {
            let pk = PublicKey::deserialize_raw(&pk_bytes.0).map_err(format_error)?;
            if pk.verify_signature(&input.result, &input.signature).is_ok() {
                verified = true;
                break;
            }
        }
        if !verified {
            return Err("signature verification failed".to_string());
        }
        let pv: types::PaymentSettleResult =
            ciborium::from_reader(&input.result[..]).map_err(format_error)?;
        if input.timestamp + 600 * 1000 < now_ms {
            return Err("payment verification result expired".to_string());
        }

        if !pv.settle_response.success {
            return Err("payment settlement failed".to_string());
        }
        if pv.settle_response.payer.is_none() {
            return Err("missing payer in settlement response".to_string());
        }

        let nonce_seed = deterministic_cbor_into_vec(&(
            true,
            &caller,
            input.timestamp,
            &pv.payment_requirements,
            &s.nonce_iv,
        ))?;
        let nonce = ByteBufB64::from(sha3_256(&nonce_seed)).to_string();
        if nonce != pv.nonce {
            return Err("nonce mismatch".to_string());
        }

        Ok(pv.settle_response)
    })?;

    store::state::deposit_currency(
        caller,
        settle_response.payer.unwrap(),
        settle_response.transaction,
        now_ms,
        true,
    )
    .await
}

#[ic_cdk::update]
fn submit_bid(amount: u128, max_price: u128) -> Result<types::BidInfo, String> {
    let caller = msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::submit_bid(caller, amount, max_price, now_ms)
}

#[ic_cdk::update]
fn claim(bid: u64) -> Result<types::BidInfo, String> {
    let caller = msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::claim(caller, bid, now_ms)
}

#[ic_cdk::update]
fn claim_all() -> Result<Vec<types::BidInfo>, String> {
    let caller = msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::claim_all(caller, now_ms)
}

// Deposit currency into the auction contract
// Returns the user account balance after deposit
#[ic_cdk::update]
async fn deposit_currency(input: types::DepositInput) -> Result<u128, String> {
    let caller = msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::deposit_currency(caller, input.sender, input.txid, now_ms, false).await
}

// Withdraw currency from the auction contract
#[ic_cdk::update]
async fn withdraw_currency(input: types::WithdrawInput) -> Result<types::WithdrawTxInfo, String> {
    let caller = msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::withdraw_currency(caller, input.recipient, now_ms).await
}

// Withdraw token from the auction contract
#[ic_cdk::update]
async fn withdraw_token(input: types::WithdrawInput) -> Result<types::WithdrawTxInfo, String> {
    let caller = msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::withdraw_token(caller, input.recipient, now_ms).await
}
