use candid::Principal;
use std::collections::BTreeSet;

use crate::{helper, store, types, x402};

static X402_NETWORK: &str = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";
static X402_ASSET: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[ic_cdk::query]
fn info() -> Result<types::StateInfo, String> {
    Ok(store::state::info())
}

#[ic_cdk::query]
fn my_txs() -> Result<Vec<String>, String> {
    let caller = helper::msg_caller()?;
    Ok(store::state::list_txs(caller))
}

#[ic_cdk::query]
fn query_token(q: String) -> Vec<(u64, types::TokenMetadata)> {
    if q.len() < 3 {
        return vec![];
    }
    store::state::query_token(q.to_ascii_lowercase())
}

#[ic_cdk::query]
fn get_token_profile(id: u64) -> Result<types::TokenProfile, String> {
    store::state::get_token_profile(id)
}

#[ic_cdk::query]
fn list_tokens(take: usize, prev_id: Option<u64>) -> Vec<(u64, types::TokenMetadata)> {
    store::state::list_tokens(take.min(1000), prev_id)
}

#[ic_cdk::query]
fn x402_payment(action: String) -> Result<x402::X402PaymentOutput, String> {
    let caller = helper::msg_caller()?;
    let timestamp = ic_cdk::api::time() / 1_000_000;
    store::state::with(|s| {
        let amount = s.x402_prices.get(&action).cloned().ok_or_else(|| {
            format!(
                "No price set for action: {}, please contact the administrator",
                action
            )
        })?;

        s.x402.get_x402_exact_payment(
            &caller,
            X402_NETWORK.to_string(),
            X402_ASSET.to_string(),
            amount.into(),
            s.x402_pay_to.clone(),
            timestamp,
            x402::ResourceInfo {
                url: "https://tokenlist.ing".to_string(),
                description: Some(format!("Payment for action: {}", action)),
                mime_type: None,
            },
            Some(format!("Payment required for action: {}", action)),
        )
    })
}

#[ic_cdk::query]
fn check_permission(token_id: u64, user: Principal) -> Vec<String> {
    store::state::check_permission(token_id, user)
}

#[ic_cdk::update]
fn register_token(
    input: types::TokenMetadata,
    payment: x402::PayingResultInput,
) -> Result<u64, String> {
    let caller = helper::msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    x402_settle(caller, "register_token", payment, now_ms)?;
    input.validate()?;
    store::state::register_token(caller, input, now_ms)
}

#[ic_cdk::update]
fn update_token_metadata(token_id: u64, input: types::TokenMetadata) -> Result<(), String> {
    let caller = helper::msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    input.validate()?;
    store::state::update_token_metadata(token_id, caller, input, now_ms)
}

#[ic_cdk::update]
fn update_token_controllers(token_id: u64, input: BTreeSet<Principal>) -> Result<(), String> {
    let caller = helper::msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::update_token_controllers(token_id, caller, input.into_iter().collect(), now_ms)
}

#[ic_cdk::update]
fn set_announcement(
    token_id: u64,
    input: types::Announcement,
    payment: x402::PayingResultInput,
) -> Result<(), String> {
    let caller = helper::msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    x402_settle(caller, "set_announcements", payment, now_ms)?;
    store::state::set_announcement(token_id, caller, input, now_ms)
}

fn x402_settle(
    caller: Principal,
    action: &str,
    payment: x402::PayingResultInput,
    now_ms: u64,
) -> Result<(), String> {
    store::state::with_mut(|s| {
        let amount = s.x402_prices.get(action).cloned().ok_or_else(|| {
            format!(
                "No price set for action: {}, please contact the administrator",
                action
            )
        })?;
        let rt = s.x402.settle_response(
            payment,
            caller,
            X402_ASSET,
            amount.into(),
            &s.x402_pay_to,
            now_ms,
        )?;

        s.total_incoming += amount as u128;

        store::state::add_tx(caller, rt.settle_response.transaction);
        Ok::<_, String>(())
    })?;
    Ok(())
}
