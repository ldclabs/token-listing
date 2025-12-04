use serde::Deserialize;
use serde_json::Value;
use std::str::FromStr;

pub use solana_account_decoder_client_types::{
    UiAccount, UiAccountData,
    token::{TokenAccountType, UiTokenAmount},
};
pub use solana_program::{hash::Hash, pubkey::Pubkey};
pub use solana_transaction::{Message, Signature, Transaction};

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LatestBlockhash {
    pub blockhash: String,
    pub last_valid_block_height: u64,
}

impl LatestBlockhash {
    pub fn to_hash(&self) -> Result<Hash, String> {
        Hash::from_str(&self.blockhash).map_err(|e| format!("Failed to parse blockhash: {}", e))
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SignatureStatus {
    pub slot: u64,
    pub confirmations: Option<u64>,
    // processed、confirmed 或 finalized
    pub confirmation_status: Option<String>,
    pub err: Option<Value>,
}

impl SignatureStatus {
    pub fn is_finalized(&self) -> bool {
        self.confirmation_status
            .as_deref()
            .map(|s| s == "finalized")
            .unwrap_or(false)
    }

    pub fn is_error(&self) -> bool {
        self.err.is_some()
    }
}

pub fn get_token_account(val: UiAccount) -> Result<TokenAccountType, String> {
    match val.data {
        UiAccountData::Json(parsed_account) => {
            let account: TokenAccountType = serde_json::from_value(parsed_account.parsed)
                .map_err(|err| format!("failed to parse TokenAccountType: {}", err))?;
            Ok(account)
        }
        _ => Err("UiAccount data is not in JSON format".to_string()),
    }
}
