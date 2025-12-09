use alloy_consensus::{SignableTransaction, Signed, TxEip1559};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, Bytes, Signature, TxHash, U256, hex};
use candid::{CandidType, Principal};
use ciborium::{from_reader, into_writer};
use ic_auth_types::ByteArrayB64;
use ic_http_certification::{
    HttpCertification, HttpCertificationPath, HttpCertificationTree, HttpCertificationTreeEntry,
    cel::{DefaultCelBuilder, create_cel_expr},
};
use ic_stable_structures::{
    DefaultMemoryImpl, StableBTreeMap, StableCell, StableLog, Storable,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
};
use icrc_ledger_types::icrc1::account::Account;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet},
    str::FromStr,
};

use crate::{
    cca,
    ecdsa::{derive_public_key, ecdsa_public_key, sign_with_ecdsa},
    evm::{EvmClient, encode_erc20_transfer},
    helper::format_error,
    icp,
    outcall::DefaultHttpOutcall,
    schnorr::{derive_schnorr_public_key, schnorr_public_key, sign_with_schnorr},
    svm::{
        Message, Pubkey, SvmClient, Transaction, create_associated_token_account_idempotent,
        get_associated_token_address, get_transfer_checked, instruction,
        transfer_checked_instruction,
    },
    types::{
        AuctionConfig, AuctionInfo, BidInfo, Chain, DepositTxInfo, PublicKeyOutput, StateInfo,
        TransferChecked, WithdrawTxInfo,
    },
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Serialize, Deserialize)]
pub struct State {
    pub name: String,
    pub description: String,
    pub url: String,
    pub persons_excluded: Vec<String>,
    // The blockchain this auction is running for
    pub chain: Chain,
    // The currency being raised in the auction
    pub currency: String,
    // Currency decimals
    pub currency_decimals: u8,
    pub currency_name: String,
    pub currency_symbol: String,
    pub currency_logo_url: String,
    // Currency program ID (Solana SPL token only)
    pub currency_program_id: Option<String>,
    // The token being sold in the auction
    pub token: String,
    // Token decimals
    pub token_decimals: u8,
    pub token_name: String,
    pub token_symbol: String,
    pub token_logo_url: String,
    // Token program ID (Solana SPL token only)
    pub token_program_id: Option<String>,
    // The recipient of the raised Currency from the auction
    pub funds_recipient: String,
    // The recipient of any unsold tokens at the end of the auction
    pub tokens_recipient: String,
    pub key_name: String,
    pub icp_address: Principal,
    pub evm_address: Address,
    pub sol_address: Pubkey,
    pub chain_providers: Vec<String>,
    pub ecdsa_public_key: PublicKeyOutput,
    pub ed25519_public_key: PublicKeyOutput,
    pub nonce_iv: ByteArrayB64<32>,
    pub governance_canister: Option<Principal>,
    pub bidders: HashSet<Principal>,
    pub pending_deposits: HashMap<Principal, u64>,
    // (gas_updated_at, gas_price, max_priority_fee_per_gas)
    pub evm_latest_gas: (u64, u128, u128),
    pub auction_finalized: bool,
    pub auction_config: Option<AuctionConfig>,
    pub auction: Option<cca::Auction>,
}

impl From<&State> for StateInfo {
    fn from(s: &State) -> Self {
        Self {
            chain: s.chain.clone(),
            name: s.name.clone(),
            description: s.description.clone(),
            url: s.url.clone(),
            persons_excluded: s.persons_excluded.clone(),
            currency: s.currency.clone(),
            currency_decimals: s.currency_decimals,
            currency_name: s.currency_name.clone(),
            currency_symbol: s.currency_symbol.clone(),
            currency_logo_url: s.currency_logo_url.clone(),
            currency_program_id: s.currency_program_id.clone(),
            token: s.token.clone(),
            token_decimals: s.token_decimals,
            token_name: s.token_name.clone(),
            token_symbol: s.token_symbol.clone(),
            token_logo_url: s.token_logo_url.clone(),
            token_program_id: s.token_program_id.clone(),
            funds_recipient: s.funds_recipient.clone(),
            tokens_recipient: s.tokens_recipient.clone(),
            key_name: s.key_name.clone(),
            icp_address: s.icp_address,
            evm_address: s.evm_address.to_string(),
            sol_address: s.sol_address.to_string(),
            chain_providers: s.chain_providers.clone(),
            governance_canister: s.governance_canister,
            auction_finalized: s.auction_finalized,
            auction_config: s.auction_config.clone(),
        }
    }
}

impl State {
    fn new() -> Self {
        Self {
            chain: Chain::Icp,
            name: "".to_string(),
            description: "".to_string(),
            url: "".to_string(),
            persons_excluded: Vec::new(),
            currency: "".to_string(),
            currency_decimals: 0,
            currency_name: "".to_string(),
            currency_symbol: "".to_string(),
            currency_logo_url: "".to_string(),
            currency_program_id: None,
            token: "".to_string(),
            token_decimals: 0,
            token_name: "".to_string(),
            token_symbol: "".to_string(),
            token_logo_url: "".to_string(),
            token_program_id: None,
            funds_recipient: "".to_string(),
            tokens_recipient: "".to_string(),
            governance_canister: None,
            key_name: "dfx_test_key".to_string(),
            icp_address: ic_cdk::api::canister_self(),
            evm_address: Address::default(),
            sol_address: Pubkey::default(), // 11111111111111111111111111111111
            chain_providers: Vec::new(),
            ecdsa_public_key: PublicKeyOutput::default(),
            ed25519_public_key: PublicKeyOutput::default(),
            nonce_iv: ByteArrayB64::default(),
            bidders: HashSet::new(),
            pending_deposits: HashMap::new(),
            evm_latest_gas: (0, 0, 0),
            auction_finalized: false,
            auction_config: None,
            auction: None,
        }
    }
}

#[derive(Clone, CandidType, Default, Serialize, Deserialize)]
pub struct UserState {
    #[serde(rename = "c")]
    pub currency_amount: u128,
    #[serde(rename = "t")]
    pub token_amount: u128,
    #[serde(rename = "b")]
    pub bids: BTreeSet<u64>,
    #[serde(rename = "a")]
    pub bound_addresses: BTreeSet<String>,
    #[serde(rename = "d")]
    pub deposits: Vec<String>,
    #[serde(rename = "w")]
    pub withdraws: Vec<u64>,
    #[serde(rename = "at")]
    pub agreed_terms: bool,
    #[serde(rename = "ts")]
    pub timestamp: u64,
}

impl Storable for UserState {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode UserState data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode UserState data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode UserState data")
    }
}

#[derive(Clone, CandidType, Serialize, Deserialize)]
pub struct DepositTx {
    #[serde(rename = "u")]
    pub user: Principal,
    #[serde(rename = "s")]
    pub sender: String,
    #[serde(rename = "a")]
    pub amount: u128,
    #[serde(rename = "t")]
    pub timestamp: u64,
}

impl Storable for DepositTx {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode DepositTx data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode DepositTx data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode DepositTx data")
    }
}

#[derive(Clone, CandidType, Serialize, Deserialize)]
pub struct WithdrawTx {
    #[serde(rename = "k")]
    pub kind: u8, // 0: currency, 1: token
    #[serde(rename = "u")]
    pub user: Principal,
    #[serde(rename = "r")]
    pub recipient: String,
    #[serde(rename = "a")]
    pub amount: u128,
    #[serde(rename = "i")]
    pub txid: String,
    #[serde(rename = "t")]
    pub timestamp: u64,
}

// todo
impl WithdrawTx {
    pub fn into_info(self, id: u64) -> WithdrawTxInfo {
        WithdrawTxInfo {
            id,
            kind: self.kind,
            user: self.user,
            recipient: self.recipient,
            amount: self.amount,
            txid: self.txid,
            timestamp: self.timestamp,
        }
    }
}

impl Storable for WithdrawTx {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode WithdrawTx data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode WithdrawTx data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode WithdrawTx data")
    }
}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const USERS_MEMORY_ID: MemoryId = MemoryId::new(1);
const DEPOSITS_MEMORY_ID: MemoryId = MemoryId::new(2);
const BIDS_MEMORY_ID: MemoryId = MemoryId::new(3);
const WITHDRAWS_INDEX_MEMORY_ID: MemoryId = MemoryId::new(4);
const WITHDRAWS_DATA_MEMORY_ID: MemoryId = MemoryId::new(5);

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::new());
    static HTTP_TREE: RefCell<HttpCertificationTree> = RefCell::new(HttpCertificationTree::default());

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static STATE_STORE: RefCell<StableCell<Vec<u8>, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(STATE_MEMORY_ID)),
            Vec::new()
        )
    );

    static USERS: RefCell<StableBTreeMap<Principal, UserState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(USERS_MEMORY_ID)),
        )
    );

    static DEPOSITS: RefCell<StableBTreeMap<String, DepositTx, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(DEPOSITS_MEMORY_ID)),
        )
    );

    static BIDS: RefCell<StableBTreeMap<u64, cca::Bid, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(BIDS_MEMORY_ID)),
        )
    );

    static WITHDRAWS: RefCell<StableLog<WithdrawTx, Memory, Memory>> = RefCell::new(
        StableLog::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(WITHDRAWS_INDEX_MEMORY_ID)),
            MEMORY_MANAGER.with_borrow(|m| m.get(WITHDRAWS_DATA_MEMORY_ID)),
        )
    );
}

struct BidStorage;

impl cca::BidStorage for BidStorage {
    fn get(&self, bid_id: u64) -> Option<cca::Bid> {
        BIDS.with_borrow(|r| r.get(&bid_id))
    }

    fn insert(&self, bid_id: u64, bid: cca::Bid) {
        BIDS.with_borrow_mut(|r| {
            r.insert(bid_id, bid);
        });
    }
}

static BS: BidStorage = BidStorage;
static SOL_ADDRESS: &str = "11111111111111111111111111111111";

pub mod state {

    use super::*;

    use lazy_static::lazy_static;
    use once_cell::sync::Lazy;

    lazy_static! {
        pub static ref DEFAULT_EXPR_PATH: HttpCertificationPath<'static> =
            HttpCertificationPath::wildcard("");
        pub static ref DEFAULT_CERTIFICATION: HttpCertification = HttpCertification::skip();
        pub static ref DEFAULT_CEL_EXPR: String =
            create_cel_expr(&DefaultCelBuilder::skip_certification());
    }

    pub static DEFAULT_CERT_ENTRY: Lazy<HttpCertificationTreeEntry> =
        Lazy::new(|| HttpCertificationTreeEntry::new(&*DEFAULT_EXPR_PATH, *DEFAULT_CERTIFICATION));

    pub async fn init_public_key() {
        let mut data = ic_cdk::management_canister::raw_rand()
            .await
            .expect("failed to generate IV");
        data.truncate(32);
        let nonce_iv: [u8; 32] = data.try_into().expect("failed to generate IV");

        let key_name = STATE.with_borrow_mut(|s| {
            s.nonce_iv = nonce_iv.into();
            s.key_name.clone()
        });
        match ecdsa_public_key(key_name.clone(), vec![]).await {
            Ok(root_pk) => {
                STATE.with_borrow_mut(|s| {
                    let self_pk =
                        derive_public_key(&root_pk, vec![s.icp_address.as_slice().to_vec()])
                            .expect("derive_public_key failed");
                    s.ecdsa_public_key = root_pk;
                    s.evm_address = self_pk.to_evm_adress().unwrap();
                });
            }
            Err(err) => {
                ic_cdk::api::debug_print(format!("failed to retrieve ECDSA public key: {err}"));
            }
        }

        match schnorr_public_key(key_name, vec![], None).await {
            Ok(root_pk) => {
                STATE.with_borrow_mut(|s| {
                    let self_pk = derive_schnorr_public_key(
                        &root_pk,
                        vec![s.icp_address.as_slice().to_vec()],
                        None,
                    )
                    .expect("derive_schnorr_public_key failed");

                    s.ed25519_public_key = root_pk;
                    s.sol_address = self_pk.to_sol_pubkey().unwrap();
                });
            }
            Err(err) => {
                ic_cdk::api::debug_print(format!("failed to retrieve Schnorr public key: {err}"));
            }
        }
    }

    pub fn with<R>(f: impl FnOnce(&State) -> R) -> R {
        STATE.with_borrow(f)
    }

    pub fn with_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
        STATE.with_borrow_mut(f)
    }

    pub fn http_tree_with<R>(f: impl FnOnce(&HttpCertificationTree) -> R) -> R {
        HTTP_TREE.with(|r| f(&r.borrow()))
    }

    pub fn init_http_certified_data() {
        HTTP_TREE.with(|r| {
            let mut tree = r.borrow_mut();
            tree.insert(&DEFAULT_CERT_ENTRY);
            ic_cdk::api::certified_data_set(tree.root_hash())
        });
    }

    pub fn load() {
        STATE_STORE.with_borrow(|r| {
            STATE.with_borrow_mut(|h| {
                let bytes = r.get();
                if bytes.is_empty() {
                    return;
                }
                let v: State = from_reader(&bytes[..]).expect("failed to decode STATE_STORE data");
                *h = v;
            });
        });
    }

    pub fn save() {
        STATE.with_borrow(|h| {
            STATE_STORE.with_borrow_mut(|r| {
                let mut buf = vec![];
                into_writer(h, &mut buf).expect("failed to encode STATE_STORE data");
                r.set(buf);
            });
        });
    }

    pub fn info() -> StateInfo {
        STATE.with_borrow(|s| StateInfo::from(s))
    }

    pub fn evm_address(user: &Principal) -> Address {
        STATE.with_borrow(|s| {
            let pk = derive_public_key(&s.ecdsa_public_key, vec![user.as_slice().to_vec()])
                .expect("derive_public_key failed");
            pk.to_evm_adress().unwrap()
        })
    }

    pub fn sol_address(user: &Principal) -> Pubkey {
        STATE.with_borrow(|s| {
            let pk = derive_schnorr_public_key(
                &s.ed25519_public_key,
                vec![user.as_slice().to_vec()],
                None,
            )
            .expect("derive_schnorr_public_key failed");
            pk.to_sol_pubkey().unwrap()
        })
    }

    pub async fn set_auction(cfg: AuctionConfig) -> Result<(), String> {
        STATE.with_borrow_mut(|s| {
            if s.auction.is_some() {
                return Err("auction is already initialized".to_string());
            }
            s.auction_config = Some(cfg);
            Ok(())
        })
    }

    pub async fn init_auction(now_ms: u64) -> Result<(), String> {
        let (chain, cfg, icp_address, sol_address, evm_address, token, decimals, token_program_id) =
            STATE.with_borrow(|s| {
                if s.auction.is_some() {
                    return Err("auction is already initialized".to_string());
                }

                let auction_config = match &s.auction_config {
                    Some(cfg) => cfg,
                    None => return Err("auction configuration is not set".to_string()),
                };

                if now_ms >= auction_config.start_time {
                    return Err("auction start time must be in the future".to_string());
                }

                Ok((
                    s.chain.clone(),
                    auction_config.clone(),
                    s.icp_address,
                    s.sol_address,
                    s.evm_address,
                    s.token.clone(),
                    s.token_decimals,
                    s.token_program_id.clone(),
                ))
            })?;

        let amount = match chain {
            Chain::Sol => {
                let token_addr = Pubkey::from_str(&token)
                    .map_err(|_| "invalid Solana token address".to_string())?;
                let program_id = token_program_id
                    .as_ref()
                    .ok_or("missing Solana token program ID".to_string())?;
                let program_id = Pubkey::from_str(program_id)
                    .map_err(|_| "invalid Solana token program ID".to_string())?;
                spl_balance_of(&sol_address, &token_addr, &program_id, now_ms).await?
            }
            Chain::Icp => {
                let ledger = Principal::from_text(&token)
                    .map_err(|_| "invalid ICP ledger principal".to_string())?;
                icp::balance_of(ledger, icp_address.into()).await?
            }
            Chain::Evm(_) => {
                let token_addr = Address::from_str(&token)
                    .map_err(|_| "invalid EVM token address".to_string())?;
                erc20_balance_of(&evm_address, &token_addr, now_ms).await?
            }
        };

        if cfg.token_decimals != decimals {
            return Err("token decimals do not match the auction configuration".to_string());
        }

        let min_lp_amount = (cfg.total_supply as f64 * 0.2) as u128;
        if cfg.total_supply + min_lp_amount > amount {
            return Err("insufficient token balance for the auction".to_string());
        }

        STATE.with_borrow_mut(|s| {
            if s.auction.is_some() {
                return Err("auction is already initialized".to_string());
            }

            s.auction = Some(cca::Auction::new(cfg)?);
            Ok(())
        })
    }

    pub async fn finalize_auction(now_ms: u64) -> Result<(), String> {
        let is_graduated = STATE.with_borrow(|s| {
            if s.auction_finalized {
                return Err("auction is already finalized".to_string());
            }

            match &s.auction {
                None => Err("auction is not initialized".to_string()),
                Some(auction) => {
                    if !auction.is_ended(now_ms) {
                        return Err("auction is not ended yet".to_string());
                    }
                    Ok(auction.is_graduated())
                }
            }
        })?;

        if is_graduated {
            // graduated auction finalization logic
            todo!("graduated auction finalization logic here");
        }

        STATE.with_borrow_mut(|s| {
            s.auction_finalized = true;
            Ok(())
        })
    }

    pub fn auction_info(now_ms: u64) -> Option<AuctionInfo> {
        STATE.with_borrow(|s| {
            s.auction.as_ref().map(|a| {
                let mut info = a.get_info(now_ms);
                info.bidders_count = s.bidders.len() as u64;
                info
            })
        })
    }

    pub fn get_grouped_bids(precision: u128) -> Vec<(u128, u128)> {
        STATE.with_borrow(|s| {
            if let Some(auction) = &s.auction {
                auction.get_grouped_bids(precision)
            } else {
                vec![]
            }
        })
    }

    pub fn estimate_max_price(amount: u128, now_ms: u64) -> u128 {
        STATE.with_borrow(|s| {
            if let Some(auction) = &s.auction {
                auction.estimate_max_price(amount, now_ms)
            } else {
                0
            }
        })
    }

    pub fn submit_bid(
        caller: Principal,
        amount: u128,
        max_price: u128,
        now_ms: u64,
    ) -> Result<BidInfo, String> {
        STATE.with_borrow_mut(|s| {
            let auction = s
                .auction
                .as_mut()
                .ok_or_else(|| "auction is not ready".to_string())?;
            USERS.with_borrow_mut(|u| {
                let mut user = u.get(&caller).unwrap_or_default();
                if user.currency_amount < amount {
                    return Err("insufficient currency balance".to_string());
                }
                let bid = auction.submit_bid(&BS, amount, max_price, now_ms)?;
                user.currency_amount -= amount;
                user.bids.insert(bid.id);
                u.insert(caller, user);
                s.bidders.insert(caller);

                Ok(bid)
            })
        })
    }

    pub fn claim(caller: Principal, bid_id: u64, now_ms: u64) -> Result<BidInfo, String> {
        STATE.with_borrow_mut(|s| {
            let auction = s
                .auction
                .as_mut()
                .ok_or_else(|| "auction is not ready".to_string())?;
            USERS.with_borrow_mut(|u| {
                let mut user = u.get(&caller).unwrap_or_default();
                if !user.bids.contains(&bid_id) {
                    return Err("bid not found for user".to_string());
                }

                let bid = auction.claim(&BS, bid_id, now_ms)?;
                user.currency_amount += bid.refund;
                user.token_amount += bid.tokens_filled;
                u.insert(caller, user);

                Ok(bid)
            })
        })
    }

    pub fn claim_all(caller: Principal, now_ms: u64) -> Result<Vec<BidInfo>, String> {
        STATE.with_borrow_mut(|s| {
            let auction = s
                .auction
                .as_mut()
                .ok_or_else(|| "auction is not ready".to_string())?;
            USERS.with_borrow_mut(|u| {
                let mut user = u.get(&caller).unwrap_or_default();
                if user.bids.is_empty() {
                    return Ok(vec![]);
                }

                let mut rt: Vec<BidInfo> = Vec::new();
                for id in user.bids.clone() {
                    if let Ok(bid) = auction.claim(&BS, id, now_ms) {
                        user.currency_amount += bid.refund;
                        user.token_amount += bid.tokens_filled;
                        rt.push(bid);
                    }
                }
                u.insert(caller, user);

                Ok(rt)
            })
        })
    }

    pub fn my_bids(caller: Principal) -> Result<Vec<BidInfo>, String> {
        USERS.with_borrow(|u| {
            let user = u.get(&caller).unwrap_or_default();
            BIDS.with_borrow(|b| {
                let mut rt: Vec<BidInfo> = Vec::with_capacity(user.bids.len());
                for id in user.bids {
                    if let Some(bid) = b.get(&id) {
                        rt.push(bid.into_info(id));
                    }
                }

                Ok(rt)
            })
        })
    }

    pub fn my_deposits(caller: Principal) -> Result<Vec<DepositTxInfo>, String> {
        let txs = USERS.with_borrow(|u| {
            let user = u.get(&caller).unwrap_or_default();
            user.deposits.clone()
        });
        DEPOSITS.with_borrow(|d| {
            let mut rt: Vec<DepositTxInfo> = Vec::new();
            for txid in txs {
                if let Some(tx) = d.get(&txid) {
                    rt.push(DepositTxInfo {
                        txid: txid.clone(),
                        user: tx.user,
                        sender: tx.sender.clone(),
                        amount: tx.amount,
                        timestamp: tx.timestamp,
                    });
                }
            }
            Ok(rt)
        })
    }

    pub fn my_withdraws(caller: Principal) -> Result<Vec<WithdrawTxInfo>, String> {
        let tx_ids = USERS.with_borrow(|u| {
            let user = u.get(&caller).unwrap_or_default();
            user.withdraws.clone()
        });
        WITHDRAWS.with_borrow(|w| {
            let mut rt: Vec<WithdrawTxInfo> = Vec::new();
            for id in tx_ids {
                if let Some(tx) = w.get(id) {
                    rt.push(tx.into_info(id));
                }
            }
            Ok(rt)
        })
    }

    pub fn bind_address(caller: Principal, address: String, now_ms: u64) -> Result<(), String> {
        STATE.with_borrow(|s| {
            s.chain.parse_address(&address)?;
            USERS.with_borrow_mut(|u| {
                let mut info = u.get(&caller).unwrap_or_default();
                if !info.agreed_terms {
                    info.timestamp = now_ms;
                    info.agreed_terms = true;
                }
                if info.bound_addresses.insert(address) {
                    u.insert(caller, info);
                }
                Ok(())
            })
        })
    }

    pub async fn deposit_currency(
        caller: Principal,
        sender: String,
        txid: String,
        now_ms: u64,
    ) -> Result<u128, String> {
        USERS.with_borrow(|u| {
            let info = u.get(&caller).unwrap_or_default();
            if !info.bound_addresses.contains(&sender) {
                return Err("sender address is not bound to user".to_string());
            }

            Ok(())
        })?;

        let chain = STATE.with_borrow_mut(|s| {
            match &s.auction {
                Some(auction) => {
                    if now_ms + auction.cfg.min_bid_duration >= auction.cfg.end_time {
                        return Err("auction is ending soon, deposits are not allowed".to_string());
                    }
                }
                None => {
                    return Err("auction is not ready".to_string());
                }
            }

            s.chain.parse_address(&sender)?;
            if let Some(ts) = s.pending_deposits.get(&caller)
                && *ts + 20 * 1000 >= now_ms
            {
                return Err(
                    "pending deposit already exists, please wait before trying again".to_string(),
                );
            };

            // prevent DDoS attacks by limiting pending deposits
            s.pending_deposits.insert(caller, now_ms);
            Ok(s.chain.clone())
        })?;

        let mut tx = DEPOSITS.with_borrow_mut(|d| {
            if d.contains_key(&txid) {
                return Err("transaction already processed".to_string());
            }
            let tx = DepositTx {
                user: caller,
                sender: sender.clone(),
                amount: 0,
                timestamp: now_ms,
            };
            d.insert(txid.clone(), tx.clone());
            Ok(tx)
        })?;

        let tx_status = match chain {
            Chain::Sol => check_sol_deposit_currency(caller, sender, txid.clone(), now_ms).await,
            Chain::Icp => check_icp_deposit_currency(caller, sender, txid.clone()).await,
            Chain::Evm(_) => check_evm_deposit_currency(caller, sender, txid.clone(), now_ms).await,
        };

        match tx_status {
            Ok(tx_status) => {
                let total_amount = USERS.with_borrow_mut(|u| {
                    let mut user = u.get(&caller).unwrap_or_default();
                    user.currency_amount += tx_status.amount;
                    user.deposits.push(txid.clone());
                    let total_amount = user.currency_amount;
                    u.insert(caller, user);
                    total_amount
                });

                tx.amount = tx_status.amount;
                DEPOSITS.with_borrow_mut(|d| {
                    d.insert(txid, tx);
                });
                Ok(total_amount)
            }

            Err(err) => {
                DEPOSITS.with_borrow_mut(|d| {
                    d.remove(&txid);
                });
                Err(err)
            }
        }
    }

    pub async fn withdraw_currency(
        caller: Principal,
        recipient: String,
        now_ms: u64,
    ) -> Result<WithdrawTxInfo, String> {
        let (chain, token, decimals, program_id) = STATE.with_borrow_mut(|s| {
            match &s.auction {
                Some(auction) => {
                    if now_ms <= auction.cfg.end_time {
                        return Err(
                            "auction is not ended yet, withdrawals are not allowed".to_string()
                        );
                    }
                }
                None => {
                    return Err("auction is not ready".to_string());
                }
            }

            s.chain.parse_address(&recipient)?;

            Ok((
                s.chain.clone(),
                s.currency.clone(),
                s.currency_decimals,
                s.currency_program_id.clone(),
            ))
        })?;

        let amount = USERS.with_borrow_mut(|u| {
            let mut info = u.get(&caller).unwrap_or_default();
            if !info.bound_addresses.contains(&recipient) {
                return Err("recipient address is not bound to user".to_string());
            }
            let amount = info.currency_amount;
            if amount == 0 {
                return Err("insufficient balance".to_string());
            }
            info.currency_amount = 0;
            u.insert(caller, info);
            Ok(amount)
        })?;

        let tx_status = match chain {
            Chain::Sol => {
                withdraw_sol_token(&token, program_id, decimals, &recipient, amount, now_ms).await
            }
            Chain::Icp => withdraw_icp_token(&token, &recipient, amount).await,
            Chain::Evm(chain_id) => {
                withdraw_evm_token(&token, &recipient, chain_id, amount, now_ms).await
            }
        };

        match tx_status {
            Ok(txid) => {
                let tx = WithdrawTx {
                    kind: 0,
                    user: caller,
                    recipient,
                    amount,
                    txid,
                    timestamp: now_ms,
                };
                let id = WITHDRAWS
                    .with_borrow_mut(|w| w.append(&tx))
                    .expect("append WithdrawTx failed");
                USERS.with_borrow_mut(|u| {
                    let mut info = u.get(&caller).unwrap_or_default();
                    info.withdraws.push(id);
                    u.insert(caller, info);
                });
                Ok(tx.into_info(id))
            }

            Err(err) => {
                USERS.with_borrow_mut(|u| {
                    let mut info = u.get(&caller).unwrap_or_default();
                    info.currency_amount = amount;
                    u.insert(caller, info);
                });
                Err(err)
            }
        }
    }

    pub async fn withdraw_token(
        caller: Principal,
        recipient: String,
        now_ms: u64,
    ) -> Result<WithdrawTxInfo, String> {
        let (chain, token, decimals, program_id) = STATE.with_borrow_mut(|s| {
            match &s.auction {
                Some(auction) => {
                    if now_ms <= auction.cfg.end_time {
                        return Err(
                            "auction is not ended yet, withdrawals are not allowed".to_string()
                        );
                    }
                }
                None => {
                    return Err("auction is not ready".to_string());
                }
            }

            s.chain.parse_address(&recipient)?;

            Ok((
                s.chain.clone(),
                s.token.clone(),
                s.token_decimals,
                s.token_program_id.clone(),
            ))
        })?;

        let amount = USERS.with_borrow_mut(|u| {
            let mut info = u.get(&caller).unwrap_or_default();
            if !info.bound_addresses.contains(&recipient) {
                return Err("recipient address is not bound to user".to_string());
            }
            let amount = info.token_amount;
            if amount == 0 {
                return Err("insufficient balance".to_string());
            }
            info.token_amount = 0;
            u.insert(caller, info);
            Ok(amount)
        })?;

        let tx_status = match chain {
            Chain::Sol => {
                withdraw_sol_token(&token, program_id, decimals, &recipient, amount, now_ms).await
            }
            Chain::Icp => withdraw_icp_token(&token, &recipient, amount).await,
            Chain::Evm(chain_id) => {
                withdraw_evm_token(&token, &recipient, chain_id, amount, now_ms).await
            }
        };

        match tx_status {
            Ok(txid) => {
                let tx = WithdrawTx {
                    kind: 1,
                    user: caller,
                    recipient,
                    amount,
                    txid,
                    timestamp: now_ms,
                };
                let id = WITHDRAWS
                    .with_borrow_mut(|w| w.append(&tx))
                    .expect("append WithdrawTx failed");
                USERS.with_borrow_mut(|u| {
                    let mut info = u.get(&caller).unwrap_or_default();
                    info.withdraws.push(id);
                    u.insert(caller, info);
                });
                Ok(tx.into_info(id))
            }

            Err(err) => {
                USERS.with_borrow_mut(|u| {
                    let mut info = u.get(&caller).unwrap_or_default();
                    info.token_amount = amount;
                    u.insert(caller, info);
                });
                Err(err)
            }
        }
    }

    pub async fn sweep_token(now_ms: u64) -> Result<WithdrawTxInfo, String> {
        let (
            chain,
            icp_address,
            sol_address,
            evm_address,
            recipient,
            token,
            decimals,
            token_program_id,
        ) = STATE.with_borrow_mut(|s| {
            if !s.auction_finalized {
                return Err("cannot sweep tokens before auction is finalized".to_string());
            }

            Ok((
                s.chain.clone(),
                s.icp_address,
                s.sol_address,
                s.evm_address,
                s.tokens_recipient.clone(),
                s.token.clone(),
                s.token_decimals,
                s.token_program_id.clone(),
            ))
        })?;

        let (amount, txid) = match chain {
            Chain::Sol => {
                let token_addr = Pubkey::from_str(&token)
                    .map_err(|_| "invalid Solana token address".to_string())?;
                let program_id = token_program_id
                    .as_ref()
                    .ok_or("missing Solana token program ID".to_string())?;
                let program_id = Pubkey::from_str(program_id)
                    .map_err(|_| "invalid Solana token program ID".to_string())?;
                let amount = spl_balance_of(&sol_address, &token_addr, &program_id, now_ms).await?;
                if amount == 0 {
                    return Err("no tokens to sweep".to_string());
                }

                let txid = withdraw_sol_token(
                    &token,
                    token_program_id,
                    decimals,
                    &recipient,
                    amount,
                    now_ms,
                )
                .await?;
                (amount, txid)
            }
            Chain::Icp => {
                let ledger = Principal::from_text(&token)
                    .map_err(|_| "invalid ICP ledger principal".to_string())?;
                let to = Account::from_str(&recipient)
                    .map_err(|_| "invalid ICP account format".to_string())?;
                let amount = icp::balance_of(ledger, icp_address.into()).await?;
                if amount == 0 {
                    return Err("no tokens to sweep".to_string());
                }
                let txid = icp::transfer(ledger, to, amount).await?;
                (amount, txid)
            }
            Chain::Evm(chain_id) => {
                let token_addr = Address::from_str(&token)
                    .map_err(|_| "invalid EVM token address".to_string())?;
                let amount = erc20_balance_of(&evm_address, &token_addr, now_ms).await?;
                if amount == 0 {
                    return Err("no tokens to sweep".to_string());
                }

                let txid = withdraw_evm_token(&token, &recipient, chain_id, amount, now_ms).await?;
                (amount, txid)
            }
        };

        let tx = WithdrawTx {
            kind: 1,
            user: icp_address,
            recipient,
            amount,
            txid,
            timestamp: now_ms,
        };
        let id = WITHDRAWS
            .with_borrow_mut(|w| w.append(&tx))
            .expect("append WithdrawTx failed");
        USERS.with_borrow_mut(|u| {
            let mut info = u.get(&icp_address).unwrap_or_default();
            info.withdraws.push(id);
            u.insert(icp_address, info);
        });
        Ok(tx.into_info(id))
    }

    pub async fn sweep_currency(now_ms: u64) -> Result<WithdrawTxInfo, String> {
        let (
            chain,
            icp_address,
            sol_address,
            evm_address,
            recipient,
            currency,
            decimals,
            currency_program_id,
        ) = STATE.with_borrow_mut(|s| {
            if !s.auction_finalized {
                return Err("cannot sweep currency before auction is finalized".to_string());
            }

            if !s.auction.as_ref().is_some_and(|a| a.is_graduated()) {
                return Err("cannot sweep currency before auction is graduated".to_string());
            }

            Ok((
                s.chain.clone(),
                s.icp_address,
                s.sol_address,
                s.evm_address,
                s.funds_recipient.clone(),
                s.currency.clone(),
                s.currency_decimals,
                s.currency_program_id.clone(),
            ))
        })?;

        let (amount, txid) = match chain {
            Chain::Sol => {
                let currency_addr = Pubkey::from_str(&currency)
                    .map_err(|_| "invalid Solana token address".to_string())?;

                let amount = if currency == SOL_ADDRESS {
                    sol_balance_of(&sol_address, now_ms).await?
                } else {
                    let program_id = currency_program_id
                        .as_ref()
                        .ok_or("missing Solana token program ID".to_string())?;
                    let program_id = Pubkey::from_str(program_id)
                        .map_err(|_| "invalid Solana token program ID".to_string())?;
                    spl_balance_of(&sol_address, &currency_addr, &program_id, now_ms).await?
                };

                if amount == 0 {
                    return Err("no tokens to sweep".to_string());
                }

                let txid = withdraw_sol_token(
                    &currency,
                    currency_program_id,
                    decimals,
                    &recipient,
                    amount,
                    now_ms,
                )
                .await?;
                (amount, txid)
            }
            Chain::Icp => {
                let ledger = Principal::from_text(&currency)
                    .map_err(|_| "invalid ICP ledger principal".to_string())?;
                let to = Account::from_str(&recipient)
                    .map_err(|_| "invalid ICP account format".to_string())?;
                let amount = icp::balance_of(ledger, icp_address.into()).await?;
                if amount == 0 {
                    return Err("no tokens to sweep".to_string());
                }
                let txid = icp::transfer(ledger, to, amount).await?;
                (amount, txid)
            }
            Chain::Evm(chain_id) => {
                let token_addr = Address::from_str(&currency)
                    .map_err(|_| "invalid EVM token address".to_string())?;
                let amount = erc20_balance_of(&evm_address, &token_addr, now_ms).await?;
                if amount == 0 {
                    return Err("no tokens to sweep".to_string());
                }

                let txid =
                    withdraw_evm_token(&currency, &recipient, chain_id, amount, now_ms).await?;
                (amount, txid)
            }
        };

        let tx = WithdrawTx {
            kind: 0,
            user: icp_address,
            recipient,
            amount,
            txid,
            timestamp: now_ms,
        };
        let id = WITHDRAWS
            .with_borrow_mut(|w| w.append(&tx))
            .expect("append WithdrawTx failed");
        USERS.with_borrow_mut(|u| {
            let mut info = u.get(&icp_address).unwrap_or_default();
            info.withdraws.push(id);
            u.insert(icp_address, info);
        });
        Ok(tx.into_info(id))
    }

    fn evm_client() -> EvmClient<DefaultHttpOutcall> {
        STATE.with_borrow(|s| {
            EvmClient::new(
                s.chain_providers.clone(),
                11,
                None,
                DefaultHttpOutcall::new(s.icp_address),
            )
        })
    }

    fn sol_client() -> SvmClient<DefaultHttpOutcall> {
        STATE.with_borrow(|s| {
            SvmClient::new(
                s.chain_providers.clone(),
                None,
                None,
                DefaultHttpOutcall::new(s.icp_address),
            )
        })
    }

    async fn check_sol_deposit_currency(
        caller: Principal,
        sender: String,
        txid: String, // 64 bytes in base58: transaction signature
        now_ms: u64,
    ) -> Result<TransferChecked, String> {
        let client = sol_client();
        let tx_status = client
            .get_transaction(now_ms, txid, Some("base64"), None)
            .await?
            .ok_or("transaction not found".to_string())?;

        STATE.with_borrow_mut(|s| {
            s.pending_deposits.remove(&caller);

            let tx_status = get_transfer_checked(tx_status, &s.currency)?;
            if tx_status.from != sender {
                return Err("transaction sender does not match sender".to_string());
            }
            if tx_status.to != s.sol_address.to_string() {
                return Err("transaction recipient does not match auction contract".to_string());
            }
            Ok(tx_status)
        })
    }

    async fn check_icp_deposit_currency(
        caller: Principal,
        sender: String,
        txid: String, // u64: ICRC Ledger block index
    ) -> Result<TransferChecked, String> {
        let block_index = txid
            .parse::<u64>()
            .map_err(|_| "Invalid block index".to_string())?;

        let ledger_id = STATE.with_borrow(|s| {
            Principal::from_text(&s.currency)
                .map_err(|_| format!("Invalid currency principal: {}", s.currency))
        })?;

        let tx_status = icp::verify_transfer_token(ledger_id, block_index).await?;
        STATE.with_borrow_mut(|s| {
            s.pending_deposits.remove(&caller);

            if tx_status.from != sender {
                return Err("transaction sender does not match sender".to_string());
            }
            if tx_status.to != s.icp_address.to_string() {
                return Err("transaction recipient does not match auction contract".to_string());
            }
            Ok(tx_status)
        })
    }

    async fn check_evm_deposit_currency(
        caller: Principal,
        sender: String,
        txid: String, // 32 bytes in hex: transaction hash
        now_ms: u64,
    ) -> Result<TransferChecked, String> {
        use alloy_primitives::hex::FromHex;
        let tx_hash =
            TxHash::from_hex(&txid).map_err(|_| "Invalid transaction hash".to_string())?;
        let client = evm_client();
        let receipt = client
            .get_transaction_receipt(now_ms, &tx_hash)
            .await?
            .ok_or("transaction not found".to_string())?;

        if !receipt.status() {
            return Err("transaction failed".to_string());
        }

        STATE.with_borrow_mut(|s| {
            s.pending_deposits.remove(&caller);

            // Check if it's a native token transfer or ERC20 transfer
            // For simplicity, we assume native token transfer if currency is empty or "ETH"
            // In a real implementation, you would check logs for ERC20 Transfer events
            // if s.currency is an ERC20 contract address.

            // Since we don't have the full transaction object here (only receipt),
            // and receipt doesn't contain value for native transfers,
            // we might need to fetch the transaction itself if we want to support native transfers.
            // However, for ERC20, we can parse logs from the receipt.

            // Assuming s.currency is the ERC20 contract address for now.
            let currency_contract = Address::parse_checksummed(&s.currency, None)
                .map_err(|_| "Invalid currency contract address".to_string())?;

            // Find the Transfer event log
            // Topic0 for Transfer(address,address,uint256) is keccak256("Transfer(address,address,uint256)")
            let transfer_topic = alloy_primitives::B256::from_hex(
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
            )
            .unwrap();

            let log = receipt
                .inner
                .logs()
                .iter()
                .find(|l| {
                    l.address() == currency_contract
                        && l.topics().first() == Some(&transfer_topic)
                        && l.topics().len() == 3
                })
                .ok_or("Transfer event not found in transaction receipt".to_string())?;

            // Parse from address (topic 1)
            let from_topic = log.topics()[1];
            let from_addr = Address::from_word(from_topic);

            // Parse to address (topic 2)
            let to_topic = log.topics()[2];
            let to_addr = Address::from_word(to_topic);

            // Parse amount (data)
            let amount = alloy_primitives::U256::from_be_slice(&log.data().data);
            let amount_u128 = u128::try_from(amount).map_err(|_| "Amount too large".to_string())?;

            if from_addr.to_string() != sender {
                return Err("transaction sender does not match sender".to_string());
            }
            if to_addr != s.evm_address {
                return Err("transaction recipient does not match auction contract".to_string());
            }

            Ok(TransferChecked {
                token: s.currency.clone(),
                from: sender,
                to: s.evm_address.to_string(),
                amount: amount_u128,
            })
        })
    }

    async fn withdraw_sol_token(
        token: &str,
        token_program_id: Option<String>,
        decimals: u8,
        recipient: &str,
        amount: u128,
        now_ms: u64,
    ) -> Result<String, String> {
        let to_addr =
            Pubkey::from_str(recipient).map_err(|_| "Invalid recipient address".to_string())?;
        let (client, signed_tx) = if token == SOL_ADDRESS {
            build_sol_transfer_tx(&to_addr, amount as u64, now_ms).await?
        } else {
            let token_program_id =
                token_program_id.ok_or("Token program ID is required".to_string())?;
            build_spl_transfer_tx(
                &to_addr,
                &Pubkey::from_str(token).map_err(|_| "Invalid token address".to_string())?,
                &Pubkey::from_str(&token_program_id.to_string())
                    .map_err(|_| "Invalid token program ID".to_string())?,
                decimals,
                amount,
                now_ms,
            )
            .await?
        };
        let txid = signed_tx.signatures[0].to_string();
        let data = bincode::serialize(&signed_tx).map_err(|err| format!("SOL: {err}"))?;

        let _ = client
            .send_transaction(now_ms, data.into(), true)
            .await
            .map_err(|err| format!("SOL: {err}"))?;
        Ok(txid)
    }

    async fn withdraw_icp_token(
        token: &str,
        recipient: &str,
        amount: u128,
    ) -> Result<String, String> {
        let ledger = Principal::from_text(token)
            .map_err(|_| format!("Invalid token principal: {}", token))?;
        let to =
            Account::from_str(recipient).map_err(|_| "Invalid recipient address".to_string())?;
        icp::transfer(ledger, to, amount).await
    }

    async fn withdraw_evm_token(
        token: &str,
        recipient: &str,
        chain_id: u64,
        amount: u128,
        now_ms: u64,
    ) -> Result<String, String> {
        let to_addr =
            Address::from_str(recipient).map_err(|_| "Invalid recipient address".to_string())?;
        let (client, signed_tx) = build_erc20_transfer_tx(
            &to_addr,
            Address::from_str(token).map_err(|_| "Invalid token address".to_string())?,
            chain_id,
            amount,
            now_ms,
        )
        .await?;

        let txid = signed_tx.hash().to_string();
        let data = signed_tx.encoded_2718();
        let _ = client
            .send_raw_transaction(now_ms, Bytes::from(data).to_string())
            .await
            .map_err(format_error)?;
        Ok(txid)
    }

    async fn spl_balance_of(
        addr: &Pubkey,
        token: &Pubkey,
        token_program_id: &Pubkey,
        now_ms: u64,
    ) -> Result<u128, String> {
        let client = sol_client();
        let ata = get_associated_token_address(addr, token, token_program_id);
        let account_data = client
            .get_token_account_balance(now_ms, ata.to_string())
            .await?;

        let balance: u128 = account_data
            .amount
            .parse()
            .map_err(|_| "failed to parse SPL token balance".to_string())?;

        Ok(balance)
    }

    async fn sol_balance_of(addr: &Pubkey, now_ms: u64) -> Result<u128, String> {
        let client = sol_client();
        let account_data = client.get_account_info(now_ms, addr.to_string()).await?;
        let account_data = account_data.ok_or("account not found".to_string())?;
        Ok(account_data.lamports as u128)
    }

    async fn erc20_balance_of(
        addr: &Address,
        token: &Address,
        now_ms: u64,
    ) -> Result<u128, String> {
        let client = evm_client();
        let balance = client.erc20_balance(now_ms, token, addr).await?;
        Ok(balance)
    }

    async fn build_spl_transfer_tx(
        to_addr: &Pubkey,
        token: &Pubkey,
        token_program_id: &Pubkey,
        decimals: u8,
        amount: u128,
        now_ms: u64,
    ) -> Result<(SvmClient<DefaultHttpOutcall>, Transaction), String> {
        let (key_name, from, from_addr, ixs) = STATE.with_borrow(|s| {
            let amount: u64 = amount
                .try_into()
                .map_err(|_| format!("amount is too large: {}", amount))?;
            let from_pk = derive_schnorr_public_key(
                &s.ed25519_public_key,
                vec![s.icp_address.as_slice().to_vec()],
                None,
            )
            .map_err(|e| format!("derive_schnorr_public_key failed: {e}"))?;
            let from_addr = from_pk.to_sol_pubkey()?;
            if &from_addr == to_addr {
                return Err("from and to cannot be the same".to_string());
            }

            let from_pubkey = get_associated_token_address(&from_addr, token, token_program_id);
            let to_pubkey = get_associated_token_address(to_addr, token, token_program_id);
            let ix0 = create_associated_token_account_idempotent(
                &from_addr,
                to_addr,
                token,
                token_program_id,
            );
            let ix = transfer_checked_instruction(
                token_program_id,
                &from_pubkey,
                token,
                &to_pubkey,
                &from_addr,
                &[],
                amount,
                decimals,
            );

            Ok::<_, String>((s.key_name.clone(), s.icp_address, from_addr, vec![ix0, ix]))
        })?;

        let client = sol_client();
        let block = client
            .get_latest_blockhash(now_ms)
            .await
            .map_err(|err| format!("SOL: failed to get latest blockhash, error: {}", err))?;

        let message = Message::new_with_blockhash(&ixs, Some(&from_addr), &block);
        let msg = bincode::serialize(&message).map_err(|err| format!("SOL: {err}"))?;
        let sig = sign_with_schnorr(key_name, vec![from.as_slice().to_vec()], msg, None).await?;
        let signature: [u8; 64] = sig.try_into().map_err(|_| "invalid signature length")?;
        let transaction = Transaction {
            message,
            signatures: vec![signature.into()],
        };

        Ok((client, transaction))
    }

    async fn build_sol_transfer_tx(
        to_addr: &Pubkey,
        amount: u64,
        now_ms: u64,
    ) -> Result<(SvmClient<DefaultHttpOutcall>, Transaction), String> {
        let (key_name, from, from_addr, ixs) = STATE.with_borrow(|s| {
            let from_pk = derive_schnorr_public_key(
                &s.ed25519_public_key,
                vec![s.icp_address.as_slice().to_vec()],
                None,
            )
            .map_err(|_e| "derive_schnorr_public_key failed".to_string())?;
            let from_addr = from_pk.to_sol_pubkey()?;
            if &from_addr == to_addr {
                return Err("from and to cannot be the same".to_string());
            }

            let ix = instruction::transfer(&from_addr, to_addr, amount);
            Ok::<_, String>((s.key_name.clone(), s.icp_address, from_addr, vec![ix]))
        })?;

        let client = sol_client();
        let block = client
            .get_latest_blockhash(now_ms)
            .await
            .map_err(|err| format!("failed to get latest blockhash, error: {}", err))?;

        let message = Message::new_with_blockhash(&ixs, Some(&from_addr), &block);
        let msg = bincode::serialize(&message).map_err(|err| format!("SOL: {err}"))?;
        let sig = sign_with_schnorr(key_name, vec![from.as_slice().to_vec()], msg, None).await?;
        let signature: [u8; 64] = sig.try_into().map_err(|_| "invalid signature length")?;
        let transaction = Transaction {
            message,
            signatures: vec![signature.into()],
        };

        Ok((client, transaction))
    }

    async fn build_erc20_transfer_tx(
        to_addr: &Address,
        token: Address,
        chain_id: u64,
        amount: u128,
        now_ms: u64,
    ) -> Result<(EvmClient<DefaultHttpOutcall>, Signed<TxEip1559>), String> {
        let (key_name, from, from_pk, mut tx, gas_updated_at) = STATE.with_borrow(|s| {
            let from_pk =
                derive_public_key(&s.ecdsa_public_key, vec![s.icp_address.as_slice().to_vec()])
                    .map_err(|_e| "derive_public_key failed".to_string())?;

            let input = encode_erc20_transfer(to_addr, amount);
            let (gas_updated_at, gas_price, max_priority_fee_per_gas) = s.evm_latest_gas;
            let max_priority_fee_per_gas = max_priority_fee_per_gas + max_priority_fee_per_gas / 5;
            Ok::<_, String>((
                s.key_name.clone(),
                s.icp_address,
                from_pk,
                TxEip1559 {
                    chain_id,
                    nonce: 0u64,
                    gas_limit: 84_000u64, // sample: ~53,696
                    max_fee_per_gas: gas_price * 2 + max_priority_fee_per_gas,
                    max_priority_fee_per_gas,
                    to: token.into(),
                    input: input.into(),
                    ..Default::default()
                },
                gas_updated_at,
            ))
        })?;

        let from_addr = from_pk.to_evm_adress()?;
        if &from_addr == to_addr {
            return Err("from and to cannot be the same".to_string());
        }

        let client = evm_client();
        if gas_updated_at + 120_000 >= now_ms {
            let nonce = client.get_transaction_count(now_ms, &from_addr).await?;
            tx.nonce = nonce;
        } else {
            let (nonce, gas_price, max_priority_fee_per_gas) = futures::future::try_join3(
                client.get_transaction_count(now_ms, &from_addr),
                client.gas_price(now_ms),
                client.max_priority_fee_per_gas(now_ms),
            )
            .await?;
            tx.nonce = nonce;
            tx.max_priority_fee_per_gas = max_priority_fee_per_gas + max_priority_fee_per_gas / 5;
            tx.max_fee_per_gas = gas_price * 2 + tx.max_priority_fee_per_gas;
            STATE.with_borrow_mut(|s| {
                s.evm_latest_gas = (now_ms, gas_price, max_priority_fee_per_gas);
            })
        }

        let msg_hash = tx.signature_hash();
        let sig =
            sign_with_ecdsa(key_name, vec![from.as_slice().to_vec()], msg_hash.to_vec()).await?;
        let signature = Signature::new(
            U256::from_be_slice(&sig[0..32]),  // r
            U256::from_be_slice(&sig[32..64]), // s
            y_parity(msg_hash.as_slice(), &sig, from_pk.public_key.as_slice())?,
        );

        let signed_tx = tx.into_signed(signature);
        Ok((client, signed_tx))
    }
}

fn y_parity(prehash: &[u8], sig: &[u8], pubkey: &[u8]) -> Result<bool, String> {
    use alloy_signer::k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

    let orig_key = VerifyingKey::from_sec1_bytes(pubkey).map_err(format_error)?;
    let signature = Signature::try_from(sig).map_err(format_error)?;
    for parity in [0u8, 1] {
        let recid = RecoveryId::try_from(parity).map_err(format_error)?;
        let recovered_key = match VerifyingKey::recover_from_prehash(prehash, &signature, recid) {
            Ok(k) => k,
            Err(_) => continue, // 尝试另一 parity
        };
        if recovered_key == orig_key {
            return Ok(parity == 1);
        }
    }

    Err(format!(
        "failed to recover the parity bit from a signature; sig: {}, pubkey: {}",
        hex::encode(sig),
        hex::encode(pubkey)
    ))
}
