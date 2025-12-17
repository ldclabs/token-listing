use candid::{CandidType, Principal};
use ic_auth_types::ByteArrayB64;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_json::Value;
use std::collections::HashMap;

use crate::{evm::Address, svm::Pubkey};

#[derive(CandidType, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PublicKeyOutput {
    pub public_key: ByteBuf,
    pub chain_code: ByteBuf,
}

impl PublicKeyOutput {
    pub fn to_sol_pubkey(&self) -> Result<Pubkey, String> {
        Pubkey::try_from(self.public_key.as_slice())
            .map_err(|_| "Failed to convert to SOL pubkey".to_string())
    }

    pub fn to_evm_adress(&self) -> Result<Address, String> {
        use k256::elliptic_curve::sec1::ToEncodedPoint;
        let key = k256::PublicKey::from_sec1_bytes(self.public_key.as_slice())
            .map_err(|_| "Failed to convert to EVM address".to_string())?;
        let point = key.to_encoded_point(false);
        Ok(Address::from_raw_public_key(&point.as_bytes()[1..]))
    }
}

#[derive(Debug, Serialize)]
pub struct RPCRequest<'a> {
    pub jsonrpc: &'a str,
    pub method: &'a str,
    pub params: &'a [Value],
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct RPCResponse<T> {
    pub result: Option<T>,
    pub error: Option<Value>,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct StateInfo {
    pub key_name: String,
    pub icp_address: Principal,
    pub evm_address: String,
    pub sol_address: String,
    pub chain_providers: HashMap<Chain, Vec<String>>,
    pub paying_public_keys: Vec<ByteArrayB64<32>>,
    pub governance_canister: Option<Principal>,
    pub auctions: Vec<AuctionId>,
    pub storages: Vec<Principal>,
    pub daos: Vec<Principal>,
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Chain {
    Icp(u64),
    Sol(u64),
    Evm(u64),
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuctionId {
    Icp(String), // canister ID on ICP
    Sol(String), // smart contract on SOL
    Evm(String), // smart contract on EVM
}

/// Auction Information
#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct AuctionInfo {
    pub id: AuctionId,
    pub name: String,
    pub description: String,
    pub url: String,
    pub chain: Chain,
    pub currency: String,
    pub currency_decimals: u8,
    pub currency_name: String,
    pub currency_symbol: String,
    pub currency_logo_url: String,
    pub token: String,
    pub token_decimals: u8,
    pub token_name: String,
    pub token_symbol: String,
    pub token_logo_url: String,
    pub required_currency_raised: u128,
    pub clearing_price: u128,
    pub total_demand_raised: u128,
    pub total_supply_released: u128,
    pub is_graduated: bool,
    pub bids_count: u64,
    pub update_time: u64,
    pub start_time: u64,
    pub end_time: u64,
}

#[derive(CandidType, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransferChecked {
    pub token: String,
    pub from: String,
    pub to: String,
    pub amount: u128,
}
