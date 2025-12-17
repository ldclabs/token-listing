use alloy_consensus::{SignableTransaction, Signed, TxEip1559};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, Bytes, Signature, U256, hex};
use candid::{CandidType, Principal};
use ciborium::{from_reader, into_writer};
use ic_auth_types::ByteArrayB64;
use ic_http_certification::{
    HttpCertification, HttpCertificationPath, HttpCertificationTree, HttpCertificationTreeEntry,
    cel::{DefaultCelBuilder, create_cel_expr},
};
use ic_stable_structures::{
    DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
};
use icrc_ledger_types::icrc1::account::Account;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell, collections::HashMap, str::FromStr};

use crate::{
    ecdsa::{derive_public_key, ecdsa_public_key, sign_with_ecdsa},
    evm::{EvmClient, encode_erc20_transfer},
    helper::format_error,
    icp,
    outcall::DefaultHttpOutcall,
    schnorr::{derive_schnorr_public_key, schnorr_public_key, sign_with_schnorr},
    svm::{
        Message, Pubkey, SvmClient, Transaction, create_associated_token_account_idempotent,
        get_associated_token_address, instruction, transfer_checked_instruction,
    },
    types::{AuctionId, AuctionInfo, Chain, PublicKeyOutput, StateInfo},
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Serialize, Deserialize)]
pub struct State {
    pub key_name: String,
    pub icp_address: Principal,
    pub evm_address: Address,
    pub sol_address: Pubkey,
    pub chain_providers: HashMap<Chain, Vec<String>>,
    pub paying_public_keys: Vec<ByteArrayB64<32>>,
    pub ecdsa_public_key: PublicKeyOutput,
    pub ed25519_public_key: PublicKeyOutput,
    pub nonce_iv: ByteArrayB64<32>,
    pub governance_canister: Option<Principal>,
    pub auctions: Vec<AuctionAddress>,
    pub storages: Vec<Principal>,
    pub daos: Vec<Principal>,
    pub pending_deposits: HashMap<Principal, u64>,
    // (gas_updated_at, gas_price, max_priority_fee_per_gas)
    pub evm_latest_gas: (u64, u128, u128),
}

impl From<&State> for StateInfo {
    fn from(s: &State) -> Self {
        Self {
            key_name: s.key_name.clone(),
            icp_address: s.icp_address,
            evm_address: s.evm_address.to_string(),
            sol_address: s.sol_address.to_string(),
            chain_providers: s.chain_providers.clone(),
            paying_public_keys: s.paying_public_keys.clone(),
            governance_canister: s.governance_canister,
            auctions: s.auctions.iter().map(|v| v.into()).collect(),
            storages: s.storages.clone(),
            daos: s.daos.clone(),
        }
    }
}

impl State {
    fn new() -> Self {
        Self {
            key_name: "dfx_test_key".to_string(),
            icp_address: ic_cdk::api::canister_self(),
            evm_address: Address::default(),
            sol_address: Pubkey::default(), // 11111111111111111111111111111111
            chain_providers: HashMap::new(),
            ecdsa_public_key: PublicKeyOutput::default(),
            ed25519_public_key: PublicKeyOutput::default(),
            nonce_iv: ByteArrayB64::default(),
            governance_canister: None,
            paying_public_keys: Vec::new(),
            auctions: Vec::new(),
            storages: Vec::new(),
            daos: Vec::new(),
            pending_deposits: HashMap::new(),
            evm_latest_gas: (0, 0, 0),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AuctionAddress {
    Icp(Principal), // ICP Principal
    Sol(Pubkey),    // SOL Pubkey
    Evm(Address),   // EVM Address
}

impl From<&AuctionAddress> for AuctionId {
    fn from(addr: &AuctionAddress) -> Self {
        match addr {
            AuctionAddress::Icp(address) => AuctionId::Icp(address.to_string()),
            AuctionAddress::Sol(address) => AuctionId::Sol(address.to_string()),
            AuctionAddress::Evm(address) => AuctionId::Evm(address.to_string()),
        }
    }
}

impl TryFrom<&AuctionId> for AuctionAddress {
    type Error = String;

    fn try_from(value: &AuctionId) -> Result<Self, Self::Error> {
        match value {
            AuctionId::Icp(principal) => Ok(AuctionAddress::Icp(
                Principal::from_text(principal)
                    .map_err(|_| format!("invalid principal: {}", principal))?,
            )),
            AuctionId::Sol(pubkey) => Ok(AuctionAddress::Sol(
                Pubkey::from_str(pubkey).map_err(|_| format!("invalid sol pubkey: {}", pubkey))?,
            )),
            AuctionId::Evm(address) => Ok(AuctionAddress::Evm(
                Address::from_str(address)
                    .map_err(|_| format!("invalid evm address: {}", address))?,
            )),
        }
    }
}

impl Storable for AuctionAddress {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        match self {
            AuctionAddress::Icp(address) => address.to_bytes().into_owned(),
            AuctionAddress::Sol(address) => address.to_bytes().to_vec(),
            AuctionAddress::Evm(address) => address.to_bytes().to_vec(),
        }
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        match self {
            AuctionAddress::Icp(address) => address.to_bytes(),
            AuctionAddress::Sol(address) => address.as_array().into(),
            AuctionAddress::Evm(address) => address.to_bytes(),
        }
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        match bytes.len() {
            32 => {
                let pubkey: [u8; 32] = bytes.as_ref().try_into().unwrap();
                let pubkey = Pubkey::from(pubkey);
                AuctionAddress::Sol(pubkey)
            }
            20 => {
                let address = Address::from_slice(&bytes);
                AuctionAddress::Evm(address)
            }
            _ => {
                let principal = Principal::from_slice(&bytes);
                AuctionAddress::Icp(principal)
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AuctionState {
    #[serde(rename = "i")]
    pub id: AuctionAddress,
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "d")]
    pub description: String,
    #[serde(rename = "u")]
    pub url: String,
    #[serde(rename = "c")]
    pub chain: Chain,
    #[serde(rename = "ca")]
    pub currency: String,
    #[serde(rename = "cd")]
    pub currency_decimals: u8,
    #[serde(rename = "cn")]
    pub currency_name: String,
    #[serde(rename = "cs")]
    pub currency_symbol: String,
    #[serde(rename = "cl")]
    pub currency_logo_url: String,
    #[serde(rename = "ta")]
    pub token: String,
    #[serde(rename = "td")]
    pub token_decimals: u8,
    #[serde(rename = "tn")]
    pub token_name: String,
    #[serde(rename = "ts")]
    pub token_symbol: String,
    #[serde(rename = "tl")]
    pub token_logo_url: String,
    #[serde(rename = "cp")]
    pub clearing_price: u128,
    #[serde(rename = "tdr")]
    pub total_demand_raised: u128,
    #[serde(rename = "tsr")]
    pub total_supply_released: u128,
    #[serde(rename = "g")]
    pub is_graduated: bool,
    #[serde(rename = "bc")]
    pub bids_count: u64,
    #[serde(rename = "ut")]
    pub update_time: u64,
    #[serde(rename = "st")]
    pub start_time: u64,
    #[serde(rename = "et")]
    pub end_time: u64,
}

impl From<AuctionState> for AuctionInfo {
    fn from(s: AuctionState) -> Self {
        Self {
            id: (&s.id).into(),
            name: s.name,
            description: s.description,
            url: s.url,
            chain: s.chain,
            currency: s.currency,
            currency_decimals: s.currency_decimals,
            currency_name: s.currency_name,
            currency_symbol: s.currency_symbol,
            currency_logo_url: s.currency_logo_url,
            token: s.token,
            token_decimals: s.token_decimals,
            token_name: s.token_name,
            token_symbol: s.token_symbol,
            token_logo_url: s.token_logo_url,
            clearing_price: s.clearing_price,
            total_demand_raised: s.total_demand_raised,
            total_supply_released: s.total_supply_released,
            is_graduated: s.is_graduated,
            bids_count: s.bids_count,
            update_time: s.update_time,
            start_time: s.start_time,
            end_time: s.end_time,
        }
    }
}

impl Storable for AuctionState {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode AuctionState data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode AuctionState data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode AuctionState data")
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

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const AUCTIONS_MEMORY_ID: MemoryId = MemoryId::new(1);

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

    static AUCTIONS: RefCell<StableBTreeMap<AuctionAddress, AuctionState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(AUCTIONS_MEMORY_ID)),
        )
    );
}

static SOL_ADDRESS: &str = "So11111111111111111111111111111111111111111";
// Wrapping SOL: So11111111111111111111111111111111111111112

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

    #[allow(unused)]
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

    pub fn get_auction(id: AuctionAddress) -> Option<AuctionInfo> {
        AUCTIONS.with_borrow(|r| r.get(&id).map(|a| a.into()))
    }

    pub fn list_auctions(take: usize, prev_id: Option<AuctionId>) -> Vec<AuctionInfo> {
        let mut result = Vec::new();
        let ids = with(|s| match prev_id {
            Some(id) => {
                if let Ok(id) = AuctionAddress::try_from(&id) {
                    s.auctions
                        .iter()
                        .rev()
                        .skip_while(|&addr| addr != &id)
                        .skip(1)
                        .take(take)
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            }
            None => s
                .auctions
                .iter()
                .rev()
                .take(take)
                .cloned()
                .collect::<Vec<_>>(),
        });

        AUCTIONS.with_borrow(|r| {
            for id in ids {
                if let Some(auction) = r.get(&id) {
                    result.push(auction.into());
                }
            }
        });

        result
    }

    pub fn set_auction(auction: AuctionInfo, now_ms: u64) -> Result<(), String> {
        let auction_addr = AuctionAddress::try_from(&auction.id)?;

        let auction_state = AuctionState {
            id: auction_addr.clone(),
            name: auction.name,
            description: auction.description,
            url: auction.url,
            chain: auction.chain,
            currency: auction.currency,
            currency_decimals: auction.currency_decimals,
            currency_name: auction.currency_name,
            currency_symbol: auction.currency_symbol,
            currency_logo_url: auction.currency_logo_url,
            token: auction.token,
            token_decimals: auction.token_decimals,
            token_name: auction.token_name,
            token_symbol: auction.token_symbol,
            token_logo_url: auction.token_logo_url,
            clearing_price: auction.clearing_price,
            total_demand_raised: auction.total_demand_raised,
            total_supply_released: auction.total_supply_released,
            is_graduated: auction.is_graduated,
            bids_count: auction.bids_count,
            update_time: now_ms,
            start_time: auction.start_time,
            end_time: auction.end_time,
        };

        AUCTIONS.with_borrow_mut(|r| r.insert(auction_addr.clone(), auction_state));

        STATE.with_borrow_mut(|s| {
            if !s.auctions.contains(&auction_addr) {
                s.auctions.push(auction_addr);
            }
        });

        Ok(())
    }

    fn evm_client(chain: &Chain) -> Result<EvmClient<DefaultHttpOutcall>, String> {
        STATE.with_borrow(|s| {
            let chain = s
                .chain_providers
                .get(chain)
                .cloned()
                .ok_or(format!("{:?} provider not configured", chain))?;
            Ok(EvmClient::new(
                chain,
                11,
                None,
                DefaultHttpOutcall::new(s.icp_address),
            ))
        })
    }

    fn sol_client(chain: &Chain) -> Result<SvmClient<DefaultHttpOutcall>, String> {
        STATE.with_borrow(|s| {
            let chain = s
                .chain_providers
                .get(chain)
                .cloned()
                .ok_or(format!("{:?} provider not configured", chain))?;
            Ok(SvmClient::new(
                chain,
                None,
                None,
                DefaultHttpOutcall::new(s.icp_address),
            ))
        })
    }

    async fn withdraw_sol_token(
        chain: &Chain,
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
            build_sol_transfer_tx(chain, &to_addr, amount as u64, now_ms).await?
        } else {
            let token_program_id =
                token_program_id.ok_or("Token program ID is required".to_string())?;
            build_spl_transfer_tx(
                chain,
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
        icp::transfer(ledger, to, amount.into()).await
    }

    async fn withdraw_evm_token(
        chain: &Chain,
        token: &str,
        recipient: &str,
        chain_id: u64,
        amount: u128,
        now_ms: u64,
    ) -> Result<String, String> {
        let to_addr =
            Address::from_str(recipient).map_err(|_| "Invalid recipient address".to_string())?;
        let (client, signed_tx) = build_erc20_transfer_tx(
            chain,
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
        chain: &Chain,
        addr: &Pubkey,
        token: &Pubkey,
        token_program_id: &Pubkey,
        now_ms: u64,
    ) -> Result<u128, String> {
        let client = sol_client(chain)?;
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

    async fn sol_balance_of(chain: &Chain, addr: &Pubkey, now_ms: u64) -> Result<u128, String> {
        let client = sol_client(chain)?;
        let account_data = client.get_account_info(now_ms, addr.to_string()).await?;
        let account_data = account_data.ok_or("account not found".to_string())?;
        Ok(account_data.lamports as u128)
    }

    async fn erc20_balance_of(
        chain: &Chain,
        addr: &Address,
        token: &Address,
        now_ms: u64,
    ) -> Result<u128, String> {
        let client = evm_client(chain)?;
        let balance = client.erc20_balance(now_ms, token, addr).await?;
        Ok(balance)
    }

    async fn build_spl_transfer_tx(
        chain: &Chain,
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

        let client = sol_client(chain)?;
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
        chain: &Chain,
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

        let client = sol_client(chain)?;
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
        chain: &Chain,
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

        let client = evm_client(chain)?;
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
