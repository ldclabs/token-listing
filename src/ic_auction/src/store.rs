use alloy_primitives::{Address, Bytes, Signature, TxHash, U256, hex};
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
    cca,
    ecdsa::{cost_sign_with_ecdsa, derive_public_key, ecdsa_public_key, sign_with_ecdsa},
    evm::{EvmClient, encode_erc20_transfer},
    helper::{call, convert_amount, format_error},
    outcall::DefaultHttpOutcall,
    schnorr::{derive_schnorr_public_key, schnorr_public_key, sign_with_schnorr},
    svm::{
        Message, Pubkey, Signature as SvmSignature, SignatureStatus, SvmClient, Transaction,
        create_associated_token_account_idempotent, get_associated_token_address, instruction,
        transfer_checked_instruction,
    },
    types::{PublicKeyOutput, StateInfo},
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

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
    pub key_name: String,
    pub icp_address: Principal,
    pub evm_address: Address,
    pub svm_address: Pubkey,
    pub ecdsa_public_key: PublicKeyOutput,
    pub ed25519_public_key: PublicKeyOutput,
    pub governance_canister: Option<Principal>,
    pub auction: Option<cca::Auction>,
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
            key_name: "dfx_test_key".to_string(),
            icp_address: ic_cdk::api::canister_self(),
            evm_address: [0u8; 20].into(),
            svm_address: Pubkey::default(), // 11111111111111111111111111111111
            ecdsa_public_key: PublicKeyOutput::default(),
            ed25519_public_key: PublicKeyOutput::default(),
            auction: None,
        }
    }
}

#[derive(Clone, CandidType, Default, Serialize, Deserialize)]
pub struct UserBids {
    pub bids: BTreeSet<u64>,
}

impl Storable for UserBids {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode UserBids data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode UserBids data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode UserBids data")
    }
}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const BIDS_MEMORY_ID: MemoryId = MemoryId::new(1);
const USER_BIDS_MEMORY_ID: MemoryId = MemoryId::new(2);
const BRIDGE_LOGS_INDEX_MEMORY_ID: MemoryId = MemoryId::new(3);
const BRIDGE_LOGS_DATA_MEMORY_ID: MemoryId = MemoryId::new(4);

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

    static BIDS: RefCell<StableBTreeMap<u64, cca::Bid, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(BIDS_MEMORY_ID)),
        )
    );

    static USER_BIDS: RefCell<StableBTreeMap<Principal, UserBids, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(USER_BIDS_MEMORY_ID)),
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
        let info = STATE.with_borrow(|s| StateInfo::from(s));
        info
    }
}
