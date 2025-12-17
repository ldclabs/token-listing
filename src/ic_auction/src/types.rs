use candid::{CandidType, Principal};
use ic_auth_types::{ByteArrayB64, ByteBufB64};
use icrc_ledger_types::icrc1::account::Account;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_json::{Map, Value};
use std::str::FromStr;

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
    pub name: String,
    pub description: String,
    pub url: String,
    pub restricted_countries: Vec<String>,
    // The blockchain this auction is running for
    pub chain: Chain,
    // The currency being raised in the auction
    pub currency: String,
    pub currency_decimals: u8,
    pub currency_name: String,
    pub currency_symbol: String,
    pub currency_logo_url: String,
    pub currency_program_id: Option<String>,
    // The token being sold in the auction
    pub token: String,
    pub token_decimals: u8,
    pub token_name: String,
    pub token_symbol: String,
    pub token_logo_url: String,
    pub token_program_id: Option<String>,
    // The recipient of the raised Currency from the auction
    pub funds_recipient: String,
    // The recipient of any unsold tokens at the end of the auction
    pub tokens_recipient: String,
    pub finalize_kind: FinalizeKind,
    pub key_name: String,
    pub icp_address: Principal,
    pub evm_address: String,
    pub sol_address: String,
    pub chain_providers: Vec<String>,
    pub paying_public_keys: Vec<ByteArrayB64<32>>,
    pub total_deposited_currency: u128,
    pub total_withdrawn_currency: u128,
    pub total_withdrawn_token: u128,
    pub total_bidders: u64,
    pub governance_canister: Option<Principal>,
    pub auction_config: Option<AuctionConfig>,
    pub finalize_output: Option<FinalizeOutput>,
}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Chain {
    Icp(u64),
    Sol(u64),
    Evm(u64),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChainAddress {
    Icp(Account), // ICP Account
    Sol(Pubkey),  // SOL Pubkey
    Evm(Address), // EVM Address
}

impl Chain {
    pub fn parse_address(&self, address: &str) -> Result<ChainAddress, String> {
        match self {
            Chain::Icp(_) => Account::from_str(address)
                .map(ChainAddress::Icp)
                .map_err(|_| format!("Invalid ICP Account: {address}")),
            Chain::Sol(_) => Pubkey::from_str(address)
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

#[derive(CandidType, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ProjectInput {
    pub name: String,
    pub description: String,
    pub url: String,
    pub restricted_countries: Vec<String>,
}

#[derive(CandidType, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TokenInput {
    pub token: String,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub logo_url: String,
    pub recipient: String,
    pub program_id: Option<String>,
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

/// Auction Information
#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct AuctionInfo {
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
    // Number of bids
    pub bids_count: u64,
}

/// Auction Snapshot
#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct AuctionSnapshot {
    // timestamp in milliseconds
    #[serde(rename = "t")]
    pub timestamp: u64,
    // Current clearing price
    #[serde(rename = "c")]
    pub clearing_price: u128,
    // Current total currency amount participating in the auction
    #[serde(rename = "f")]
    pub current_flow_rate: u128,
    // Cumulative currency raised
    #[serde(rename = "d")]
    pub cumulative_demand_raised: u128,
    // Cumulative tokens released
    #[serde(rename = "s")]
    pub cumulative_supply_released: u128,
}

/// Auction Configuration
/// Example scenario:
/// Auctioning PAY tokens. Total supply 1 billion. Auctioning 10% (100 million PAY).
/// Auction time: 2026-01-01 08:00:00 GMT+08 to 2026-01-04 08:00:00 GMT+08.
/// Currency: USDC. Minimum raise target: 500,000 USDC. Decimals: 6.
/// Floor price will be: 0.005 USDC / PAY.
/// Min bid: 100 USDC. Max bid: 10,000 USDC.
/// Example config:
/// AuctionConfig {
///     start_time: 1767225600000, // in milliseconds
///     end_time: 1767484800000,
///     min_bid_duration: 300000,          // 5 minutes
///     total_supply: 100_000_000_000_000_000,
///     liquidity_pool_amount: 80_000_000_000_000_000, // 80% of total supply
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
    // Total supply to be released linearly, in token atomic units
    pub total_supply: u128,
    // Amount of tokens to be added to the liquidity pool, in token atomic units
    pub liquidity_pool_amount: u128,
    // Minimum bid amount per transaction, in currency atomic units
    pub min_amount: u128,
    // Maximum bid amount per transaction, in currency atomic units
    pub max_amount: u128,
    // Amount of currency required to be raised for the auction to graduate, in currency atomic units
    pub required_currency_raised: u128,
}

impl AuctionConfig {
    pub fn validate(&self, token_decimals: u8, now_ms: u64) -> Result<(), String> {
        if self.start_time <= now_ms {
            return Err("Auction start time cannot be in the past".to_string());
        }
        if self.start_time + self.min_bid_duration >= self.end_time {
            return Err("Invalid auction time range".to_string());
        }
        if self.min_bid_duration < 1000 {
            return Err("Minimum bid duration too short".to_string());
        }
        if token_decimals > 18 {
            return Err("Token decimals too high".to_string());
        }
        let one_token = 10u128.pow(token_decimals as u32);
        if self.total_supply < one_token {
            return Err("Total supply too low".to_string());
        }
        if self.total_supply > MAX_TOTAL_SUPPLY {
            return Err("Total supply exceeds maximum allowed".to_string());
        }

        if self.liquidity_pool_amount > self.total_supply {
            return Err("Liquidity pool amount cannot exceed total auction supply".to_string());
        }

        if self.min_amount == 0 {
            return Err("Minimum bid amount must be greater than zero".to_string());
        }
        if self.min_amount >= self.max_amount {
            return Err("Invalid bid amount range".to_string());
        }
        if self.required_currency_raised as f64 / (self.total_supply as f64 / one_token as f64)
            < 1.0
        {
            return Err("Required currency raised too low".to_string());
        }
        Ok(())
    }
}

#[derive(CandidType, Clone, Default, Serialize, Deserialize)]
pub struct UserInfo {
    pub currency_amount: u128,
    pub token_amount: u128,
    pub bound_addresses: Vec<String>,
    pub agreed_terms: bool,
    pub timestamp: u64,
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

#[derive(CandidType, Clone, Serialize, Deserialize)]
pub struct DepositTxInfo {
    pub txid: String,
    pub user: Principal,
    pub sender: String,
    pub amount: u128,
    pub timestamp: u64,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
pub struct WithdrawTxInfo {
    pub id: u64,
    pub kind: u8, // 0: currency, 1: token
    pub user: Principal,
    pub recipient: String,
    pub amount: u128,
    pub txid: String,
    pub timestamp: u64,
}

#[derive(CandidType, Clone, Default, Serialize, Deserialize)]
pub enum FinalizeKind {
    CreatePool(String), // "KongSwap" or "Raydium"
    #[default]
    Transfer,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
pub struct FinalizeOutput {
    pub pool_id: String,
    pub txid: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct X402PaymentOutput {
    pub x402: ByteBufB64, // PaymentRequirementsResponse in CBOR
    pub nonce: String,
    pub timestamp: u64,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
pub struct PayingResultInput {
    pub result: ByteBufB64, // PaymentVerifyResult / PaymentSettleResult in CBOR,
    pub signature: ByteBufB64, // Signature over (result || nonce || timestamp)
    pub timestamp: u64,     // the timestamp from X402PaymentOutput
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentVerifyResult {
    pub payment_requirements: PaymentRequirements,
    pub verify_response: VerifyResponse,
    pub nonce: String, // the nonce from X402PaymentOutput
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSettleResult {
    pub payment_requirements: PaymentRequirements,
    pub settle_response: SettleResponse,
    pub nonce: String, // the nonce from X402PaymentOutput
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResponse {
    pub is_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<String>,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
    pub transaction: String,
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequired {
    /// Protocol version identifier
    pub x402_version: u8,
    /// Human-readable error message explaining why payment is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// ResourceInfo object describing the protected resource
    pub resource: ResourceInfo,
    /// Array of payment requirement objects defining acceptable payment methods
    pub accepts: Vec<PaymentRequirements>,
    /// Protocol extensions data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Payment requirements set by the payment-gated endpoint for an acceptable payment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirements {
    /// Payment scheme identifier (e.g., "exact")
    pub scheme: String,
    /// Blockchain network identifier (e.g., "icp")
    pub network: String,
    /// Required payment amount in atomic token units
    pub amount: String,
    /// Token ledger canister address
    pub asset: String,
    /// Recipient wallet address for the payment
    pub pay_to: String,
    /// Maximum time allowed for payment completion in seconds
    pub max_timeout_seconds: u64,
    /// Scheme-specific additional information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Map<String, Value>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceInfo {
    /// the protected resource, e.g., URL of the resource endpoint
    pub url: String,
    /// Human-readable description of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the expected response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Describes additional extension data for x402 payment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Extension-specific data provided by the server
    pub info: Map<String, Value>,
    /// JSON Schema defining the expected structure of `info`
    pub schema: Map<String, Value>,
}
