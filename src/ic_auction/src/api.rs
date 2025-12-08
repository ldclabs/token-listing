use candid::Principal;

use crate::{
    helper::{check_auth, msg_caller},
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
fn evm_address(user: Option<Principal>) -> Result<String, String> {
    let user = user.unwrap_or_else(ic_cdk::api::msg_caller);
    check_auth(&user)?;
    let addr = store::state::evm_address(&user);
    Ok(addr.to_string())
}

#[ic_cdk::query]
fn sol_address(user: Option<Principal>) -> Result<String, String> {
    let user = user.unwrap_or_else(ic_cdk::api::msg_caller);
    check_auth(&user)?;
    let addr = store::state::sol_address(&user);
    Ok(addr.to_string())
}

#[ic_cdk::query]
fn my_bids() -> Result<Vec<types::BidInfo>, String> {
    let caller = msg_caller()?;
    store::state::my_bids(caller)
}

#[ic_cdk::query]
fn estimate_max_price(amount: u128) -> u128 {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    store::state::estimate_max_price(amount, now_ms)
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
    store::state::deposit_currency(caller, input.sender, input.txid, now_ms).await
}

// Withdraw currency from the auction contract
// Returns the transaction id of the withdrawal
#[ic_cdk::update]
async fn withdraw_currency(input: types::WithdrawInput) -> Result<String, String> {
    todo!()
}

// Withdraw token from the auction contract
// Returns the transaction id of the withdrawal
#[ic_cdk::update]
async fn withdraw_token(input: types::WithdrawInput) -> Result<String, String> {
    todo!()
}
