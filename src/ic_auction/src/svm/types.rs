use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};

pub use solana_account_decoder_client_types::{
    UiAccount, UiAccountData,
    token::{TokenAccountType, UiTokenAmount},
};
pub use solana_program::{hash::Hash, pubkey::Pubkey};
pub use solana_transaction::{Message, Transaction};
pub use solana_transaction_status_client_types::EncodedTransactionWithStatusMeta;

use crate::types::TransferChecked;

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
    #[allow(unused)]
    pub fn is_finalized(&self) -> bool {
        self.confirmation_status
            .as_deref()
            .map(|s| s == "finalized")
            .unwrap_or(false)
    }

    #[allow(unused)]
    pub fn is_error(&self) -> bool {
        self.err.is_some()
    }
}

#[allow(unused)]
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

pub fn get_transfer_checked(
    data: EncodedTransactionWithStatusMeta,
    token: &str,
) -> Result<TransferChecked, String> {
    let meta = data.meta.ok_or("No transaction meta found".to_string())?;
    let pre_token_balances = meta
        .pre_token_balances
        .ok_or("No preTokenBalances found".to_string())?;
    let post_token_balances = meta
        .post_token_balances
        .ok_or("No postTokenBalances found".to_string())?;

    // 使用 HashMap 存储 pre_token_balances，key 为 account_index
    let mut pre_balances = HashMap::new();
    for balance in pre_token_balances {
        if balance.mint == token {
            pre_balances.insert(balance.account_index, balance);
        }
    }

    let mut from = None;
    let mut to = None;
    let mut amount = 0;

    for post in post_token_balances {
        if post.mint != token {
            continue;
        }

        // 获取对应的 pre_balance，如果不存在则默认为 0
        let pre = pre_balances.get(&post.account_index);
        let pre_amount: u128 = match pre {
            Some(p) => p.ui_token_amount.amount.parse().unwrap_or(0),
            None => 0,
        };

        let post_amount: u128 = post
            .ui_token_amount
            .amount
            .parse()
            .map_err(|_| "Failed to parse post amount".to_string())?;

        if post_amount > pre_amount {
            // 余额增加，是接收方
            to = post.owner.clone().into();
            amount = post_amount - pre_amount;
        } else if post_amount < pre_amount {
            // 余额减少，是发送方
            // 优先使用 pre 中的 owner，如果不存在则尝试使用 post 中的 owner
            from = pre
                .and_then(|p| p.owner.clone().into())
                .or(post.owner.clone().into());
        }
    }

    if let (Some(from), Some(to)) = (from, to)
        && amount > 0
    {
        return Ok(TransferChecked {
            token: token.to_string(),
            from,
            to,
            amount,
        });
    }

    Err("No transfer checked found in the transaction".to_string())
}
