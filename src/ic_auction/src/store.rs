use alloy_primitives::Address;
use candid::{CandidType, Principal};
use ciborium::{from_reader, into_writer};
use ic_auth_types::{ByteArrayB64, ByteBufB64};
use ic_http_certification::{
    HttpCertification, HttpCertificationPath, HttpCertificationTree, HttpCertificationTreeEntry,
    cel::{DefaultCelBuilder, create_cel_expr},
};
use ic_stable_structures::{
    DefaultMemoryImpl, StableBTreeMap, StableCell, StableLog, Storable,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet},
};

use crate::{
    cca,
    ecdsa::{derive_public_key, ecdsa_public_key},
    evm::{EvmClient, TxHash},
    helper::get_icrc_transfer,
    outcall::DefaultHttpOutcall,
    schnorr::{derive_schnorr_public_key, schnorr_public_key},
    svm::{
        Pubkey, SvmClient,
        get_transfer_checked,
    },
    types::{
        AuctionInfo, BidInfo, Chain, ChainAddress, PublicKeyOutput, StateInfo, TransferChecked,
    },
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Serialize, Deserialize)]
pub struct State {
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
    // pub token_name: String,
    // pub token_symbol: String,
    // pub token_decimals: u8,
    // pub token_logo: String,
    // pub token_total_supply: u128,
    pub key_name: String,
    pub icp_address: Principal,
    pub evm_address: String,
    pub sol_address: String,
    pub chain_providers: Vec<String>,
    pub ecdsa_public_key: PublicKeyOutput,
    pub ed25519_public_key: PublicKeyOutput,
    pub nonce_iv: ByteArrayB64<32>,
    pub governance_canister: Option<Principal>,
    pub bidders: HashSet<Principal>,
    pub pending_deposits: HashMap<Principal, u64>,
    pub auction: Option<cca::Auction>,
}

impl From<&State> for StateInfo {
    fn from(s: &State) -> Self {
        Self {
            chain: s.chain.clone(),
            currency: s.currency.clone(),
            token: s.token.clone(),
            funds_recipient: s.funds_recipient.clone(),
            tokens_recipient: s.tokens_recipient.clone(),
            key_name: s.key_name.clone(),
            icp_address: s.icp_address,
            evm_address: s.evm_address.clone(),
            sol_address: s.sol_address.clone(),
            chain_providers: s.chain_providers.clone(),
            governance_canister: s.governance_canister,
        }
    }
}

impl State {
    fn new() -> Self {
        Self {
            chain: Chain::Icp,
            currency: "".to_string(),
            token: "".to_string(),
            funds_recipient: "".to_string(),
            tokens_recipient: "".to_string(),
            governance_canister: None,
            key_name: "dfx_test_key".to_string(),
            icp_address: ic_cdk::api::canister_self(),
            evm_address: "".to_string(),
            sol_address: Pubkey::default().to_string(), // 11111111111111111111111111111111
            chain_providers: Vec::new(),
            ecdsa_public_key: PublicKeyOutput::default(),
            ed25519_public_key: PublicKeyOutput::default(),
            nonce_iv: ByteArrayB64::default(),
            bidders: HashSet::new(),
            pending_deposits: HashMap::new(),
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
    pub deposits: Vec<ByteBufB64>,
    #[serde(rename = "w")]
    pub withdraws: Vec<u64>,
    #[serde(rename = "p")]
    pub pending_tx: Option<String>,
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
    pub sender: ByteBufB64,
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
    pub recipient: ByteBufB64,
    #[serde(rename = "a")]
    pub amount: u128,
    #[serde(rename = "i")]
    pub txid: ByteBufB64,
    #[serde(rename = "t")]
    pub timestamp: u64,
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
                    s.evm_address = self_pk.to_evm_adress().unwrap().to_string();
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
                    s.sol_address = self_pk.to_sol_pubkey().unwrap().to_string();
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

        let addr = STATE.with_borrow_mut(|s| {
            let addr = s.chain.parse_address(&sender)?;
            if let Some(ts) = s.pending_deposits.get(&caller)
                && *ts + 20 * 1000 >= now_ms {
                    return Err("pending deposit already exists".to_string());
                };

            // prevent DDoS attacks by limiting pending deposits
            s.pending_deposits.insert(caller, now_ms);
            Ok(addr)
        })?;

        let mut tx = DEPOSITS.with_borrow_mut(|d| {
            if d.contains_key(&txid) {
                return Err("transaction already processed".to_string());
            }
            let tx = DepositTx {
                user: caller,
                sender: addr.to_vec().into(),
                amount: 0,
                timestamp: now_ms,
            };
            d.insert(txid.clone(), tx.clone());
            Ok(tx)
        })?;

        let tx_status = match addr {
            ChainAddress::Sol(addr) => {
                check_sol_deposit_currency(caller, addr, txid.clone(), now_ms).await
            }
            ChainAddress::Icp(addr) => check_icp_deposit_currency(caller, addr, txid.clone()).await,
            ChainAddress::Evm(addr) => {
                check_evm_deposit_currency(caller, addr, txid.clone(), now_ms).await
            }
        };

        match tx_status {
            Ok(tx_status) => {
                let total_amount = USERS.with_borrow_mut(|u| {
                    let mut user = u.get(&caller).unwrap_or_default();
                    user.currency_amount += tx_status.amount;
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

    async fn check_sol_deposit_currency(
        caller: Principal,
        sender: Pubkey,
        txid: String, // 64 bytes in base58: transaction signature
        now_ms: u64,
    ) -> Result<TransferChecked, String> {
        let client = sol_client();
        let tx_status = client
            .get_transaction(now_ms, &txid, Some("base64"), None)
            .await?
            .ok_or("transaction not found".to_string())?;

        STATE.with_borrow_mut(|s| {
            s.pending_deposits.remove(&caller);

            let tx_status = get_transfer_checked(tx_status, &s.currency)?;
            if tx_status.from != sender.to_string() {
                return Err("transaction sender does not match sender".to_string());
            }
            if tx_status.to != s.sol_address {
                return Err("transaction recipient does not match auction contract".to_string());
            }
            Ok(tx_status)
        })
    }

    async fn check_icp_deposit_currency(
        caller: Principal,
        sender: Principal,
        txid: String, // u64: ICRC Ledger block index
    ) -> Result<TransferChecked, String> {
        let block_index = txid
            .parse::<u64>()
            .map_err(|_| "Invalid block index".to_string())?;

        let ledger_id = STATE.with_borrow(|s| {
            Principal::from_text(&s.currency)
                .map_err(|_| format!("Invalid currency principal: {}", s.currency))
        })?;

        let tx_status = get_icrc_transfer(ledger_id, block_index).await?;
        STATE.with_borrow_mut(|s| {
            s.pending_deposits.remove(&caller);

            if tx_status.from != sender.to_string() {
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
        sender: Address,
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
            // 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
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

            if from_addr != sender {
                return Err("transaction sender does not match sender".to_string());
            }

            let self_evm_addr = Address::parse_checksummed(&s.evm_address, None)
                .map_err(|_| "Invalid self EVM address".to_string())?;

            if to_addr != self_evm_addr {
                return Err("transaction recipient does not match auction contract".to_string());
            }

            Ok(TransferChecked {
                token: s.currency.clone(),
                from: sender.to_string(),
                to: s.evm_address.clone(),
                amount: amount_u128,
            })
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
}
