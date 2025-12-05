use alloy_primitives::Address;
use candid::{CandidType, Principal};
use ciborium::{from_reader, into_writer};
use ic_http_certification::{
    HttpCertification, HttpCertificationPath, HttpCertificationTree, HttpCertificationTreeEntry,
    cel::{DefaultCelBuilder, create_cel_expr},
};
use ic_stable_structures::{
    DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell, collections::BTreeSet};

use crate::{
    cca,
    ecdsa::{derive_public_key, ecdsa_public_key},
    schnorr::{derive_schnorr_public_key, schnorr_public_key},
    svm::Pubkey,
    types::{AuctionInfo, AuctionToken, BidInfo, PublicKeyOutput, StateInfo},
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Serialize, Deserialize)]
pub struct State {
    // The token being sold in the auction
    pub token: AuctionToken,
    // The currency being raised in the auction
    pub currency: String,
    // The recipient of the raised Currency from the auction
    pub funds_recipient: String,
    // The recipient of any unsold tokens at the end of the auction
    pub tokens_recipient: String,
    // pub token_name: String,
    // pub token_symbol: String,
    // pub token_decimals: u8,
    // pub token_logo: String,
    // pub token_total_supply: u128,
    pub key_name: String,
    pub icp_address: Principal,
    pub evm_address: Address,
    pub svm_address: Pubkey,
    pub chain_providers: Vec<String>,
    pub ecdsa_public_key: PublicKeyOutput,
    pub ed25519_public_key: PublicKeyOutput,
    pub governance_canister: Option<Principal>,
    pub auction: Option<cca::Auction>,
}

impl From<&State> for StateInfo {
    fn from(s: &State) -> Self {
        Self {
            token: s.token.clone(),
            currency: s.currency.clone(),
            funds_recipient: s.funds_recipient.clone(),
            tokens_recipient: s.tokens_recipient.clone(),
            key_name: s.key_name.clone(),
            icp_address: s.icp_address,
            evm_address: s.evm_address.to_string(),
            svm_address: s.svm_address.to_string(),
            chain_providers: s.chain_providers.clone(),
            governance_canister: s.governance_canister,
        }
    }
}

impl State {
    fn new() -> Self {
        Self {
            token: AuctionToken::Icp(Principal::anonymous().to_string()),
            currency: Principal::anonymous().to_string(),
            funds_recipient: "".to_string(),
            tokens_recipient: "".to_string(),
            governance_canister: None,
            key_name: "dfx_test_key".to_string(),
            icp_address: ic_cdk::api::canister_self(),
            evm_address: [0u8; 20].into(),
            svm_address: Pubkey::default(), // 11111111111111111111111111111111
            chain_providers: Vec::new(),
            ecdsa_public_key: PublicKeyOutput::default(),
            ed25519_public_key: PublicKeyOutput::default(),
            auction: None,
        }
    }
}

#[derive(Clone, CandidType, Default, Serialize, Deserialize)]
pub struct UserState {
    pub currency_amount: u128,
    pub token_amount: u128,
    pub bids: BTreeSet<u64>,
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

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const USERS_MEMORY_ID: MemoryId = MemoryId::new(1);
const BIDS_MEMORY_ID: MemoryId = MemoryId::new(2);
const TX_LOGS_INDEX_MEMORY_ID: MemoryId = MemoryId::new(3);
const TX_LOGS_DATA_MEMORY_ID: MemoryId = MemoryId::new(4);

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

    static BIDS: RefCell<StableBTreeMap<u64, cca::Bid, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(BIDS_MEMORY_ID)),
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
        let key_name = STATE.with_borrow(|r| r.key_name.clone());
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
                    s.svm_address = self_pk.to_svm_pubkey().unwrap();
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

    pub fn svm_address(user: &Principal) -> Pubkey {
        STATE.with_borrow(|s| {
            let pk = derive_schnorr_public_key(
                &s.ed25519_public_key,
                vec![user.as_slice().to_vec()],
                None,
            )
            .expect("derive_schnorr_public_key failed");
            pk.to_svm_pubkey().unwrap()
        })
    }

    pub fn auction_info(now_ms: u64) -> Option<AuctionInfo> {
        STATE.with_borrow(|s| s.auction.as_ref().map(|a| a.get_info(now_ms)))
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
}
