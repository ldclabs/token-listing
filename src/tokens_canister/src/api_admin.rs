use ic_auth_types::ByteArrayB64;
use serde_json::{Map, Value};
use std::str::FromStr;

use crate::{
    helper::{format_error, pretty_format},
    store, types,
};

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_paying_public_keys(public_keys: Vec<String>) -> Result<(), String> {
    let mut paying_public_keys = Vec::new();
    for key in public_keys {
        paying_public_keys.push(ByteArrayB64::from_str(&key).map_err(format_error)?)
    }

    store::state::with_mut(|s| {
        s.x402.paying_public_keys = paying_public_keys;
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_paying_public_keys(public_keys: Vec<String>) -> Result<String, String> {
    for key in &public_keys {
        ByteArrayB64::<32>::from_str(key).map_err(format_error)?;
    }
    pretty_format(&(public_keys,))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_x402_prices(prices: Vec<(String, u64)>) -> Result<(), String> {
    store::state::with_mut(|s| {
        s.x402_prices = prices.into_iter().collect();
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_x402_prices(prices: Vec<(String, u64)>) -> Result<String, String> {
    pretty_format(&(prices,))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_x402_pay_to(x402_pay_to: String) -> Result<(), String> {
    store::state::with_mut(|s| {
        s.x402_pay_to = x402_pay_to;
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_x402_pay_to(x402_pay_to: String) -> Result<String, String> {
    pretty_format(&(x402_pay_to,))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_payment_requirements_extra(payment_requirements_extra: String) -> Result<(), String> {
    let val: Map<String, Value> =
        serde_json::from_str(&payment_requirements_extra).map_err(format_error)?;
    store::state::with_mut(|s| {
        s.x402.payment_requirements_extra = Some(val);
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_payment_requirements_extra(
    payment_requirements_extra: String,
) -> Result<String, String> {
    let _: Map<String, Value> =
        serde_json::from_str(&payment_requirements_extra).map_err(format_error)?;
    pretty_format(&(payment_requirements_extra,))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_batch_register_tokens(tokens: Vec<types::TokenMetadata>) -> Result<Vec<u64>, String> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    let caller = ic_cdk::api::msg_caller();
    let mut rt = Vec::new();
    for token in tokens {
        if token.validate().is_err() {
            continue;
        }
        if let Ok(id) = store::state::register_token(caller, token, now_ms) {
            rt.push(id);
        }
    }
    Ok(rt)
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_update_token_status(token_id: u64, status: String) -> Result<(), String> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    let status: types::TokenStatus = status.parse()?;
    store::state::admin_update_token_status(token_id, status, now_ms)
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_update_token_tags(token_id: u64, tags: Vec<String>) -> Result<(), String> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::admin_update_token_tags(token_id, tags, now_ms)
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_update_token_verification_badge(
    token_id: u64,
    badge: types::VerificationBadge,
) -> Result<(), String> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::admin_update_token_verification_badge(token_id, badge, now_ms)
}

fn is_controller() -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    if ic_cdk::api::is_controller(&caller)
        || store::state::with(|s| s.governance_canister == Some(caller))
    {
        Ok(())
    } else {
        Err("user is not a controller".to_string())
    }
}
