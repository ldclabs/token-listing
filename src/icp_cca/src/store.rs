use candid::{CandidType, Nat, Principal};
use ciborium::{from_reader, into_writer};
use ic_http_certification::{
    HttpCertification, HttpCertificationPath, HttpCertificationTree, HttpCertificationTreeEntry,
    cel::{DefaultCelBuilder, create_cel_expr},
};
use ic_stable_structures::{
    DefaultMemoryImpl, StableBTreeMap, StableCell, StableLog, Storable,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
};
use icrc_ledger_types::{
    icrc1::{account::Account, transfer::TransferArg},
    icrc2::transfer_from::{TransferFromArgs, TransferFromError},
};
use num_traits::cast::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteArray;
use std::{
    borrow::Cow,
    cell::RefCell,
    cmp,
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
    time::Duration,
};

use crate::{
    helper::{call, convert_amount, format_error},
    types::PublicKeyOutput,
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_ERROR_ROUNDS: u64 = 42;

#[derive(Clone, Serialize, Deserialize)]
pub struct State {
    // The currency being raised in the auction
    pub currency: Principal,
    // The token being sold in the auction
    pub token: Principal,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub token_logo: String,
    pub token_total_supply: u128,
    pub governance_canister: Option<Principal>,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct StateInfo {
    // The currency being raised in the auction
    pub currency: Principal,
    // The token being sold in the auction
    pub token: Principal,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub token_logo: String,
    pub token_total_supply: u128,
    pub governance_canister: Option<Principal>,
}

impl From<&State> for StateInfo {
    fn from(s: &State) -> Self {
        Self {
            currency: s.currency,
            token: s.token,
            token_name: s.token_name.clone(),
            token_symbol: s.token_symbol.clone(),
            token_decimals: s.token_decimals,
            token_logo: s.token_logo.clone(),
            token_total_supply: s.token_total_supply,
            governance_canister: s.governance_canister,
        }
    }
}

impl State {
    fn new() -> Self {
        Self {
            currency: Principal::anonymous(),
            token: Principal::anonymous(),
            token_name: String::new(),
            token_symbol: String::new(),
            token_decimals: 0,
            token_logo: String::new(),
            token_total_supply: 0,
            governance_canister: None,
        }
    }
}

type Timestamp = u64; // 纳秒
type Amount = u128; // 余额/代币数量

// 精度常量
const RATE_PRECISION: u128 = 1_000_000_000_000; // 流速精度 (1e12)
const PRICE_PRECISION: u128 = 1_000_000_000; // 价格精度 (1e9)
const ACC_PRECISION: u128 = 1_000_000_000_000_000_000; // 累积器精度 (1e18)

#[derive(Clone, CandidType, Default, Serialize, Deserialize)]
pub struct UserLogs {
    pub logs: BTreeSet<u64>,
}

impl Storable for UserLogs {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode UserLogs data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode UserLogs data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode UserLogs data")
    }
}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const USER_LOGS_MEMORY_ID: MemoryId = MemoryId::new(1);
const BRIDGE_LOGS_INDEX_MEMORY_ID: MemoryId = MemoryId::new(2);
const BRIDGE_LOGS_DATA_MEMORY_ID: MemoryId = MemoryId::new(3);

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

    static USER_LOGS: RefCell<StableBTreeMap<Principal, UserLogs, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(USER_LOGS_MEMORY_ID)),
        )
    );
}

pub mod state {
    use std::str::FromStr;

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
        let info = STATE.with_borrow(|s| StateInfo::from(s));
        info
    }
}
