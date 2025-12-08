use url::Url;

use crate::{helper::pretty_format, store, types};

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_token(input: types::TokenInput) -> Result<(), String> {
    store::state::with_mut(|s| {
        if s.auction.is_some() {
            return Err("cannot change token when an auction is ongoing".to_string());
        }
        if matches!(s.chain, types::Chain::Sol) && input.program_id.is_none() {
            return Err("program_id is required for Solana tokens".to_string());
        }
        s.chain.parse_address(&input.token)?;
        s.chain.parse_address(&input.recipient)?;
        s.token = input.token;
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
        if matches!(s.chain, types::Chain::Sol) && input.program_id.is_none() {
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
        if matches!(s.chain, types::Chain::Sol)
            && input.token != "11111111111111111111111111111111"
            && input.program_id.is_none()
        {
            return Err("program_id is required for Solana tokens".to_string());
        }
        s.chain.parse_address(&input.token)?;
        s.chain.parse_address(&input.recipient)?;
        s.currency = input.token;
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
        if matches!(s.chain, types::Chain::Sol)
            && input.token != "11111111111111111111111111111111"
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
fn admin_set_auction(auction: types::AuctionConfig) -> Result<(), String> {
    todo!()
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
