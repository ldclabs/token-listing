use alloy_primitives::Address;
use candid::Principal;
use std::collections::BTreeSet;
use url::Url;

use crate::{helper::pretty_format, store, types};

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_token(token: types::AuctionToken, tokens_recipient: String) -> Result<(), String> {
    token.validate()?;
    token.validate_address(&tokens_recipient)?;
    store::state::with_mut(|s| {
        if s.auction.is_some() {
            return Err("cannot change token when an auction is ongoing".to_string());
        }
        s.token = token;
        s.tokens_recipient = tokens_recipient;
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_token(
    token: types::AuctionToken,
    tokens_recipient: String,
) -> Result<String, String> {
    token.validate()?;
    token.validate_address(&tokens_recipient)?;
    pretty_format(&(token, tokens_recipient))
}

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_currency(currency: String, funds_recipient: String) -> Result<(), String> {
    store::state::with_mut(|s| {
        if s.auction.is_some() {
            return Err("cannot change currency when an auction is ongoing".to_string());
        }
        s.token.validate_address(&currency)?;
        s.token.validate_address(&funds_recipient)?;
        s.currency = currency;
        s.funds_recipient = funds_recipient;
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_currency(
    currency: String,
    funds_recipient: String,
) -> Result<String, String> {
    store::state::with(|s| {
        if s.auction.is_some() {
            return Err("cannot change currency when an auction is ongoing".to_string());
        }
        s.token.validate_address(&currency)?;
        s.token.validate_address(&funds_recipient)?;
        Ok(())
    })?;
    pretty_format(&(currency, funds_recipient))
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
