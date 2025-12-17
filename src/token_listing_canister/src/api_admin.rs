use std::str::FromStr;

use ic_auth_types::ByteArrayB64;
use url::Url;

use crate::{
    helper::{format_error, pretty_format},
    store, types,
};

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_providers(chain: types::Chain, providers: Vec<String>) -> Result<(), String> {
    for url in &providers {
        let v = Url::parse(url).map_err(|err| format!("invalid url {url}, error: {err}"))?;
        if v.scheme() != "https" {
            return Err(format!("url scheme must be https, got: {url}"));
        }
    }

    store::state::with_mut(|s| {
        s.chain_providers.insert(chain, providers);
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_providers(
    chain: types::Chain,
    providers: Vec<String>,
) -> Result<String, String> {
    for url in &providers {
        let v = Url::parse(url).map_err(|err| format!("invalid url {url}, error: {err}"))?;
        if v.scheme() != "https" {
            return Err(format!("url scheme must be https, got: {url}"));
        }
    }
    pretty_format(&(chain, providers))
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
fn admin_set_auction(auction: types::AuctionInfo) -> Result<(), String> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::set_auction(auction, now_ms)
}

#[ic_cdk::update]
fn validate_admin_set_auction(auction: types::AuctionInfo) -> Result<String, String> {
    pretty_format(&(auction,))
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
