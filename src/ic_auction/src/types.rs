use std::str::FromStr;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_json::Value;

use crate::{evm::Address, svm::Pubkey};

pub const MAX_TOTAL_SUPPLY: u128 = 1_000_000_000_000_000_000_000_000_000_000; // Maximum total supply (1e30)

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
    // The blockchain this auction is running for
    pub chain: Chain,
    // The currency being raised in the auction
    pub currency: String,
    // The token being sold in the auction
    pub token: String,
    // The recipient of the raised Currency from the auction
    pub funds_recipient: String,
    // The recipient of any unsold tokens at the end of the auction
    pub tokens_recipient: String,
    pub key_name: String,
    pub icp_address: Principal,
    pub evm_address: String,
    pub sol_address: String,
    pub chain_providers: Vec<String>,
    pub governance_canister: Option<Principal>,
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Chain {
    Icp,         // ICP Principal
    Sol,         // SOL Pubkey
    Evm(String), // EVM Address
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChainAddress {
    Icp(Principal), // ICP Principal
    Sol(Pubkey),    // SOL Pubkey
    Evm(Address),   // EVM Address
}

impl Chain {
    pub fn parse_address(&self, address: &str) -> Result<ChainAddress, String> {
        match self {
            Chain::Icp => Principal::from_text(address)
                .map(ChainAddress::Icp)
                .map_err(|_| format!("Invalid ICP principal: {address}")),
            Chain::Sol => Pubkey::from_str(address)
                .map(ChainAddress::Sol)
                .map_err(|_| format!("Invalid SOL pubkey: {address}")),
            Chain::Evm(_) => Address::from_str(address)
                .map(ChainAddress::Evm)
                .map_err(|_| format!("Invalid EVM address: {address}")),
        }
    }
}

impl std::fmt::Display for ChainAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainAddress::Icp(addr) => addr.fmt(f),
            ChainAddress::Sol(addr) => addr.fmt(f),
            ChainAddress::Evm(addr) => addr.fmt(f),
        }
    }
}

impl ChainAddress {
    pub fn chain(&self) -> Chain {
        match self {
            ChainAddress::Icp(_) => Chain::Icp,
            ChainAddress::Sol(_) => Chain::Sol,
            ChainAddress::Evm(_) => Chain::Evm("".to_string()), // EVM doesn't have a specific chain name
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            ChainAddress::Icp(addr) => addr.as_slice(),
            ChainAddress::Sol(addr) => addr.as_array(),
            ChainAddress::Evm(addr) => addr.as_ref(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }
}

#[derive(CandidType, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DepositInput {
    pub sender: String,
    pub txid: String,
}

#[derive(CandidType, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct WithdrawInput {
    pub recipient: String,
}

#[derive(CandidType, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransferChecked {
    pub token: String,
    pub from: String,
    pub to: String,
    pub amount: u128,
}

/// Auction Information Snapshot
#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct AuctionInfo {
    // Static auction configuration
    pub auction: AuctionConfig,
    // Current timestamp in milliseconds
    pub timestamp: u64,
    // Current clearing price
    pub clearing_price: u128,
    // Current total currency amount participating in the auction
    pub total_amount: u128,
    // Current total tokens filled
    pub total_tokens_filled: u128,
    // Current total currency refunded
    pub total_refunded: u128,
    // Cumulative currency raised
    pub cumulative_demand_raised: u128,
    // Cumulative tokens released
    pub cumulative_supply_released: u128,
    // Whether the auction has graduated
    pub is_graduated: bool,
    // Number of unique bidders
    pub bidders_count: u64,
}

/// Auction Configuration
/// Example scenario:
/// Auctioning PAY tokens. Total supply 1 billion. Auctioning 10% (100 million PAY). Decimals: 9.
/// Auction time: 2026-01-01 08:00:00 GMT+08 to 2026-01-04 08:00:00 GMT+08.
/// Currency: USDC. Minimum raise target: 500,000 USDC. Decimals: 6.
/// Floor price will be: 0.005 USDC / PAY.
/// Min bid: 100 USDC. Max bid: 10,000 USDC.
/// Example config:
/// AuctionConfig {
///     start_time: 1767225600000, // in milliseconds
///     end_time: 1767484800000,
///     min_bid_duration: 300000,          // 5 minutes
///     token_decimals: 9,
///     total_supply: 100_000_000_000_000_000,
///     min_amount: 100_000_000,
///     max_amount: 10_000_000_000,
///     required_currency_raised: 500_000_000_000,
/// }
///
#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct AuctionConfig {
    // Auction start and end time in milliseconds
    pub start_time: u64,
    pub end_time: u64,
    // Minimum bid duration in milliseconds. Prevents sniping attacks; longer duration increases sniping cost.
    pub min_bid_duration: u64,
    // Token decimals
    pub token_decimals: u8,
    // Total supply to be released linearly, in token atomic units
    pub total_supply: u128,
    // Minimum bid amount per transaction, in currency atomic units
    pub min_amount: u128,
    // Maximum bid amount per transaction, in currency atomic units
    pub max_amount: u128,
    // Amount of currency required to be raised for the auction to graduate, in currency atomic units
    pub required_currency_raised: u128,
}

impl AuctionConfig {
    pub fn check(&self) {
        if self.start_time + self.min_bid_duration >= self.end_time {
            panic!("Invalid auction time range");
        }
        if self.min_bid_duration < 1_00 {
            panic!("Minimum bid duration too short");
        }
        if self.token_decimals > 18 {
            panic!("Token decimals too high");
        }
        let one_token = 10u128.pow(self.token_decimals as u32);
        if self.total_supply < one_token {
            panic!("Total supply too low");
        }
        if self.total_supply > MAX_TOTAL_SUPPLY {
            panic!("Total supply exceeds maximum allowed");
        }
        if self.total_supply <= 1 {
            panic!("Total supply too low for the auction duration");
        }
        if self.min_amount == 0 {
            panic!("Minimum bid amount must be greater than zero");
        }
        if self.min_amount >= self.max_amount {
            panic!("Invalid bid amount range");
        }
        if self.required_currency_raised as f64 / (self.total_supply as f64 / one_token as f64)
            < 1.0
        {
            panic!("Required currency raised too low");
        }
    }
}

/// Bid Information
#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct BidInfo {
    pub id: u64,
    // User's currency amount
    pub amount: u128,
    // The max price of the bid
    pub max_price: u128,

    // --- Dynamic Settlement Fields ---
    pub flow_rate: u128,    // Flow rate (Currency / ns)
    pub acc_snapshot: u128, // Global accumulator snapshot at entry

    pub create_time: u64,                  // Creation time, allows early entry
    pub outbid_time: Option<u64>,          // Time when the bid was outbid
    pub outbid_acc_snapshot: Option<u128>, // Global accumulator snapshot when outbid

    pub tokens_filled: u128, // Amount of tokens filled
    pub refund: u128,        // Amount of currency refunded
    pub claim_time: u64,     // Claim/Settlement time
}
