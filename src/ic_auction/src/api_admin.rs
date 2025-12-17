use std::str::FromStr;

use ic_auth_types::ByteArrayB64;
use url::Url;

use crate::{
    helper::{format_error, pretty_format},
    store, types,
};

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_project(input: types::ProjectInput) -> Result<(), String> {
    store::state::with_mut(|s| {
        if s.auction.is_some() {
            return Err("cannot change token when an auction is ongoing".to_string());
        }

        s.name = input.name;
        s.description = input.description;
        s.url = input.url;
        s.restricted_countries = input.restricted_countries;
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_project(input: types::ProjectInput) -> Result<String, String> {
    store::state::with(|s| {
        if s.auction.is_some() {
            return Err("cannot change token when an auction is ongoing".to_string());
        }
        Ok(())
    })?;
    pretty_format(&(input,))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_token(input: types::TokenInput) -> Result<(), String> {
    store::state::with_mut(|s| {
        if s.auction.is_some() {
            return Err("cannot change token when an auction is ongoing".to_string());
        }
        if matches!(s.chain, types::Chain::Sol(_)) && input.program_id.is_none() {
            return Err("program_id is required for Solana tokens".to_string());
        }
        s.chain.parse_address(&input.token)?;
        s.chain.parse_address(&input.recipient)?;
        s.token = input.token;
        s.token_name = input.name;
        s.token_symbol = input.symbol;
        s.token_logo_url = input.logo_url;
        s.token_program_id = input.program_id;
        s.token_decimals = input.decimals;
        s.tokens_recipient = input.recipient;
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_token(input: types::TokenInput) -> Result<String, String> {
    store::state::with(|s| {
        if s.auction.is_some() {
            return Err("cannot change token when an auction is ongoing".to_string());
        }
        if matches!(s.chain, types::Chain::Sol(_)) && input.program_id.is_none() {
            return Err("program_id is required for Solana tokens".to_string());
        }
        s.chain.parse_address(&input.token)?;
        s.chain.parse_address(&input.recipient)?;
        Ok(())
    })?;
    pretty_format(&(input,))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_currency(input: types::TokenInput) -> Result<(), String> {
    store::state::with_mut(|s| {
        if s.auction.is_some() {
            return Err("cannot change currency when an auction is ongoing".to_string());
        }
        if matches!(s.chain, types::Chain::Sol(_))
            && input.token != "So11111111111111111111111111111111111111111"
            && input.program_id.is_none()
        {
            return Err("program_id is required for Solana tokens".to_string());
        }
        s.chain.parse_address(&input.token)?;
        s.chain.parse_address(&input.recipient)?;
        s.currency = input.token;
        s.currency_name = input.name;
        s.currency_symbol = input.symbol;
        s.currency_logo_url = input.logo_url;
        s.currency_program_id = input.program_id;
        s.currency_decimals = input.decimals;
        s.funds_recipient = input.recipient;
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_currency(input: types::TokenInput) -> Result<String, String> {
    store::state::with(|s| {
        if s.auction.is_some() {
            return Err("cannot change currency when an auction is ongoing".to_string());
        }
        if matches!(s.chain, types::Chain::Sol(_))
            && input.token != "So11111111111111111111111111111111111111111"
            && input.program_id.is_none()
        {
            return Err("program_id is required for Solana tokens".to_string());
        }
        s.chain.parse_address(&input.token)?;
        s.chain.parse_address(&input.recipient)?;
        Ok(())
    })?;
    pretty_format(&(input,))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_finalize(input: types::FinalizeKind) -> Result<(), String> {
    store::state::with_mut(|s| {
        if s.auction.is_some() {
            return Err("cannot change token when an auction is ongoing".to_string());
        }
        s.finalize_kind = input;
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_finalize(input: types::FinalizeKind) -> Result<String, String> {
    pretty_format(&(input,))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_providers(providers: Vec<String>) -> Result<(), String> {
    for url in &providers {
        let v = Url::parse(url).map_err(|err| format!("invalid url {url}, error: {err}"))?;
        if v.scheme() != "https" {
            return Err(format!("url scheme must be https, got: {url}"));
        }
    }

    store::state::with_mut(|s| {
        if s.auction.is_some() {
            return Err("cannot change token when an auction is ongoing".to_string());
        }
        s.chain_providers = providers;
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_providers(providers: Vec<String>) -> Result<String, String> {
    for url in &providers {
        let v = Url::parse(url).map_err(|err| format!("invalid url {url}, error: {err}"))?;
        if v.scheme() != "https" {
            return Err(format!("url scheme must be https, got: {url}"));
        }
    }
    pretty_format(&(providers,))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_paying_public_keys(public_keys: Vec<String>) -> Result<(), String> {
    let mut paying_public_keys = Vec::new();
    for key in public_keys {
        paying_public_keys.push(ByteArrayB64::from_str(&key).map_err(format_error)?)
    }

    store::state::with_mut(|s| {
        s.paying_public_keys = paying_public_keys;
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
async fn admin_set_auction(auction: types::AuctionConfig) -> Result<(), String> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    let token_decimals = store::state::with(|s| s.token_decimals);
    auction.validate(token_decimals, now_ms)?;
    store::state::set_auction(auction).await
}

#[ic_cdk::update]
fn validate_admin_set_auction(auction: types::AuctionConfig) -> Result<String, String> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    let token_decimals = store::state::with(|s| s.token_decimals);
    auction.validate(token_decimals, now_ms)?;
    pretty_format(&(auction,))
}

#[ic_cdk::update(guard = "is_controller")]
async fn admin_setup_auction() -> Result<(), String> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::setup_auction(now_ms).await
}

#[ic_cdk::update(guard = "is_controller")]
async fn admin_finalize_auction() -> Result<Option<types::FinalizeOutput>, String> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::finalize_auction(now_ms).await
}

#[ic_cdk::update(guard = "is_controller")]
async fn admin_sweep_currency() -> Result<types::WithdrawTxInfo, String> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::sweep_currency(now_ms, None).await
}

#[ic_cdk::update(guard = "is_controller")]
async fn admin_sweep_token() -> Result<types::WithdrawTxInfo, String> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::sweep_token(now_ms).await
}

#[ic_cdk::update]
fn validate_empty_input() -> Result<String, String> {
    pretty_format(&())
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
