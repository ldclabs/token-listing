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
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
};

use crate::{
    types::{
        Announcement, ChainLocation, LinkItem, LinkType, StateInfo, TokenMetadata, TokenProfile,
        TokenStatus, VerificationBadge,
    },
    x402::*,
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Serialize, Deserialize)]
pub struct State {
    pub tokens: BTreeMap<u64, TokenMetadataState>, // tokens 数量不会太多，不可能超过内存限制
    pub location_index: BTreeMap<String, u64>,     // key: unique location -> token id
    pub inverted_index: BTreeMap<String, BTreeSet<u64>>, // key: lowercased symbol/name/asset_reference -> token ids
    pub next_id: u64,
    pub x402: X402State,
    pub x402_prices: BTreeMap<String, u64>,
    pub x402_pay_to: String,
    pub governance_canister: Option<Principal>,
}

impl From<&State> for StateInfo {
    fn from(s: &State) -> Self {
        Self {
            total_tokens: s.tokens.len() as u64,
            x402_paying_public_keys: s.x402.paying_public_keys.clone(),
            x402_prices: s.x402_prices.clone(),
            x402_pay_to: s.x402_pay_to.clone(),
            governance_canister: s.governance_canister,
        }
    }
}

impl State {
    fn new() -> Self {
        Self {
            tokens: BTreeMap::new(),
            location_index: BTreeMap::new(),
            inverted_index: BTreeMap::new(),
            next_id: 1,
            x402: X402State {
                canister: ic_cdk::api::canister_self(),
                user_nonce: HashMap::new(), // 付费 user 的 nonce 记录，数量有限
                paying_public_keys: Vec::new(),
                payment_requirements_extra: None,
            },
            x402_prices: BTreeMap::new(),
            x402_pay_to: String::new(),
            governance_canister: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenMetadataState {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "d")]
    pub decimals: u8,
    #[serde(rename = "i")]
    pub image: String,
    #[serde(rename = "de")]
    pub description: String,
    #[serde(rename = "u")]
    pub external_url: String,
    #[serde(rename = "l")]
    pub links: Vec<LinkItemState>,
    #[serde(rename = "lo")]
    pub locations: Vec<String>,
}

impl From<&TokenMetadataState> for TokenMetadata {
    fn from(s: &TokenMetadataState) -> Self {
        Self {
            name: s.name.clone(),
            symbol: s.symbol.clone(),
            decimals: s.decimals,
            image: s.image.clone(),
            description: s.description.clone(),
            external_url: s.external_url.clone(),
            links: s.links.iter().map(|v| v.into()).collect(),
            locations: s.locations.clone(),
        }
    }
}

impl From<TokenMetadataState> for TokenMetadata {
    fn from(s: TokenMetadataState) -> Self {
        Self {
            name: s.name,
            symbol: s.symbol,
            decimals: s.decimals,
            image: s.image,
            description: s.description,
            external_url: s.external_url,
            links: s.links.into_iter().map(|v| v.into()).collect(),
            locations: s.locations,
        }
    }
}

impl Storable for TokenMetadataState {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode TokenMetadataState data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode TokenMetadataState data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode TokenMetadataState data")
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LinkItemState {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "u")]
    pub url: String,
    #[serde(rename = "r")]
    pub rel: LinkType,
}

impl From<&LinkItemState> for LinkItem {
    fn from(s: &LinkItemState) -> Self {
        Self {
            name: s.name.clone(),
            url: s.url.clone(),
            rel: s.rel.clone(),
        }
    }
}

impl From<LinkItemState> for LinkItem {
    fn from(s: LinkItemState) -> Self {
        Self {
            name: s.name,
            url: s.url,
            rel: s.rel,
        }
    }
}

impl Storable for LinkItemState {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode LinkItemState data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode LinkItemState data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode LinkItemState data")
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenProfileState {
    #[serde(rename = "i")]
    pub id: u64,
    #[serde(rename = "c")]
    pub controllers: BTreeSet<Principal>,
    #[serde(rename = "s")]
    pub status: TokenStatus,
    #[serde(rename = "ca")]
    pub created_at: u64,
    #[serde(rename = "ua")]
    pub updated_at: u64,
    #[serde(rename = "m")]
    pub metadata: TokenMetadataState,
    #[serde(rename = "t")]
    pub tags: BTreeSet<String>,
    #[serde(rename = "v")]
    pub verification: VerificationBadgeState,
    #[serde(rename = "a")]
    pub announcements: Vec<AnnouncementState>,
}

impl From<&TokenProfileState> for TokenProfile {
    fn from(s: &TokenProfileState) -> Self {
        Self {
            id: s.id,
            controllers: s.controllers.iter().cloned().collect(),
            status: s.status.clone(),
            created_at: s.created_at,
            updated_at: s.updated_at,
            metadata: (&s.metadata).into(),
            tags: s.tags.iter().cloned().collect(),
            verification: (&s.verification).into(),
            announcements: s.announcements.iter().map(|v| v.into()).collect(),
        }
    }
}

impl From<TokenProfileState> for TokenProfile {
    fn from(s: TokenProfileState) -> Self {
        Self {
            id: s.id,
            controllers: s.controllers.into_iter().collect(),
            status: s.status,
            created_at: s.created_at,
            updated_at: s.updated_at,
            metadata: (&s.metadata).into(),
            tags: s.tags.into_iter().collect(),
            verification: s.verification.into(),
            announcements: s.announcements.into_iter().map(|v| v.into()).collect(),
        }
    }
}

impl Storable for TokenProfileState {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode TokenProfileState data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode TokenProfileState data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode TokenProfileState data")
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct VerificationBadgeState {
    #[serde(rename = "i")]
    pub is_verified: bool,
    #[serde(rename = "m")]
    pub methods: Vec<String>,
    #[serde(rename = "v")]
    pub verified_at: u64,
}

impl From<&VerificationBadgeState> for VerificationBadge {
    fn from(s: &VerificationBadgeState) -> Self {
        Self {
            is_verified: s.is_verified,
            methods: s.methods.clone(),
            verified_at: s.verified_at,
        }
    }
}

impl From<VerificationBadgeState> for VerificationBadge {
    fn from(s: VerificationBadgeState) -> Self {
        Self {
            is_verified: s.is_verified,
            methods: s.methods,
            verified_at: s.verified_at,
        }
    }
}

impl Storable for VerificationBadge {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode VerificationBadge data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode VerificationBadge data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode VerificationBadge data")
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AnnouncementState {
    #[serde(rename = "i")]
    pub id: u64,
    #[serde(rename = "t")]
    pub r#type: String,
    #[serde(rename = "ti")]
    pub title: String,
    #[serde(rename = "c")]
    pub content: String,
    #[serde(rename = "u")]
    pub url: Option<String>,
    #[serde(rename = "p")]
    pub published_at: u64,
}

impl From<&AnnouncementState> for Announcement {
    fn from(s: &AnnouncementState) -> Self {
        Self {
            id: s.id,
            r#type: s.r#type.clone(),
            title: s.title.clone(),
            content: s.content.clone(),
            url: s.url.clone(),
            published_at: s.published_at,
        }
    }
}

impl From<AnnouncementState> for Announcement {
    fn from(s: AnnouncementState) -> Self {
        Self {
            id: s.id,
            r#type: s.r#type,
            title: s.title,
            content: s.content,
            url: s.url,
            published_at: s.published_at,
        }
    }
}

impl Storable for AnnouncementState {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode AnnouncementState data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode AnnouncementState data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode AnnouncementState data")
    }
}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const TOKENS_MEMORY_ID: MemoryId = MemoryId::new(1);

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

    static TOKENS: RefCell<StableBTreeMap<u64, TokenProfileState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(TOKENS_MEMORY_ID)),
        )
    );
}

pub mod state {
    use std::str::FromStr;

    use crate::types::UniswapToken;

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

    fn inverted_index_keys(meta: &TokenMetadataState) -> Result<BTreeSet<String>, String> {
        if meta.locations.is_empty() {
            return Err("token locations cannot be empty".to_string());
        }

        let locations = meta
            .locations
            .iter()
            .map(|loc| ChainLocation::from_str(loc))
            .collect::<Result<Vec<_>, _>>()?;

        let mut keys = BTreeSet::new();
        keys.insert(meta.symbol.to_ascii_lowercase());
        keys.insert(meta.name.to_ascii_lowercase());
        for loc in locations {
            keys.insert(loc.asset_reference.to_ascii_lowercase());
        }
        Ok(keys)
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

    pub fn register_token(
        caller: Principal,
        token: TokenMetadata,
        now_ms: u64,
    ) -> Result<u64, String> {
        STATE.with_borrow_mut(|s| {
            if token.locations.is_empty() {
                return Err("token locations cannot be empty".to_string());
            }

            for loc in &token.locations {
                if s.location_index.contains_key(&loc.to_ascii_lowercase()) {
                    return Err(format!("token location {} already registered", loc));
                }
            }

            let id = s.next_id;
            s.next_id += 1;

            let token_state = TokenMetadataState {
                name: token.name,
                symbol: token.symbol,
                decimals: token.decimals,
                image: token.image,
                description: token.description,
                external_url: token.external_url,
                links: token
                    .links
                    .into_iter()
                    .map(|v| LinkItemState {
                        name: v.name,
                        url: v.url,
                        rel: v.rel,
                    })
                    .collect(),
                locations: token.locations,
            };
            let new_keys = inverted_index_keys(&token_state)?;
            for key in new_keys {
                s.inverted_index
                    .entry(key)
                    .and_modify(|v| {
                        v.insert(id);
                    })
                    .or_insert_with(|| BTreeSet::from([id]));
            }
            for loc in &token_state.locations {
                s.location_index.insert(loc.to_ascii_lowercase(), id);
            }

            s.tokens.insert(id, token_state.clone());
            TOKENS.with_borrow_mut(|t| {
                let profile = TokenProfileState {
                    id,
                    controllers: BTreeSet::from([caller]),
                    status: TokenStatus::Active,
                    created_at: now_ms,
                    updated_at: now_ms,
                    metadata: token_state,
                    tags: BTreeSet::new(),
                    verification: VerificationBadgeState::default(),
                    announcements: Vec::new(),
                };
                t.insert(id, profile);
            });
            Ok(id)
        })
    }

    pub fn update_token_metadata(
        id: u64,
        user: Principal,
        token: TokenMetadata,
        now_ms: u64,
    ) -> Result<(), String> {
        STATE.with_borrow_mut(|s| {
            let existing_state = s
                .tokens
                .get(&id)
                .ok_or_else(|| format!("Token with id {} not found", id))?;

            for loc in &token.locations {
                if let Some(_id) = s.location_index.get(&loc.to_ascii_lowercase())
                    && _id != &id {
                        return Err(format!("token location {} already registered", loc));
                    }
            }

            let token_state = TokenMetadataState {
                name: token.name,
                symbol: token.symbol,
                decimals: token.decimals,
                image: token.image,
                description: token.description,
                external_url: token.external_url,
                links: token
                    .links
                    .into_iter()
                    .map(|v| LinkItemState {
                        name: v.name,
                        url: v.url,
                        rel: v.rel,
                    })
                    .collect(),
                locations: token.locations,
            };

            TOKENS.with_borrow(|t| {
                let profile = t
                    .get(&id)
                    .ok_or_else(|| format!("Token with id {} not found", id))?;
                if !profile.controllers.contains(&user) && s.governance_canister != Some(user) {
                    return Err("user is not a controller of the token".to_string());
                }
                Ok::<(), String>(())
            })?;

            // Update inverted index if symbol/name/locations changed.
            let old_keys = inverted_index_keys(existing_state)?;
            let new_keys = inverted_index_keys(&token_state)?;

            let to_remove: Vec<String> = old_keys.difference(&new_keys).cloned().collect();
            for key in to_remove {
                if let Some(ids) = s.inverted_index.get_mut(&key) {
                    ids.remove(&id);
                    if ids.is_empty() {
                        s.inverted_index.remove(&key);
                    }
                }
            }

            for key in new_keys {
                s.inverted_index
                    .entry(key)
                    .and_modify(|v| {
                        v.insert(id);
                    })
                    .or_insert_with(|| BTreeSet::from([id]));
            }

            let old_keys: BTreeSet<String> = existing_state
                .locations
                .iter()
                .map(|loc| loc.to_ascii_lowercase())
                .collect();
            let new_keys: BTreeSet<String> = token_state
                .locations
                .iter()
                .map(|loc| loc.to_ascii_lowercase())
                .collect();
            let to_remove: Vec<String> = old_keys.difference(&new_keys).cloned().collect();
            for loc in to_remove {
                s.location_index.remove(&loc);
            }
            for loc in new_keys {
                s.location_index.insert(loc, id);
            }
            s.tokens.insert(id, token_state.clone());

            // Finally, persist profile changes.
            TOKENS.with_borrow_mut(|t| {
                let mut profile = t
                    .get(&id)
                    .ok_or_else(|| format!("Token with id {} not found", id))?;
                profile.metadata = token_state;
                profile.updated_at = now_ms;
                t.insert(id, profile);
                Ok::<(), String>(())
            })?;

            Ok(())
        })
    }

    pub fn update_token_controllers(
        id: u64,
        user: Principal,
        controllers: Vec<Principal>,
        now_ms: u64,
    ) -> Result<(), String> {
        TOKENS.with_borrow_mut(|t| {
            let mut profile = t
                .get(&id)
                .ok_or_else(|| format!("Token with id {} not found", id))?;
            if !profile.controllers.contains(&user) && !profile.controllers.is_empty() {
                return Err("user is not a controller of the token".to_string());
            }
            profile.controllers = controllers.into_iter().collect();
            profile.updated_at = now_ms;
            t.insert(id, profile);
            Ok(())
        })
    }

    pub fn set_location(id: u64) -> Result<(), String> {
        STATE.with_borrow_mut(|s| {
            let token = s
                .tokens
                .get(&id)
                .ok_or_else(|| format!("Token with id {} not found", id))?;

            let ticker = token.symbol.to_ascii_lowercase();
            if s.location_index.contains_key(&ticker) {
                return Err(format!("ticker location {} already registered", ticker));
            }
            s.location_index.insert(ticker, id);
            Ok(())
        })
    }

    pub fn set_announcement(
        id: u64,
        caller: Principal,
        input: Announcement,
        now_ms: u64,
    ) -> Result<(), String> {
        TOKENS.with_borrow_mut(|t| {
            let mut token = t
                .get(&id)
                .ok_or_else(|| format!("Token with id {} not found", id))?;
            if !token.controllers.contains(&caller) {
                return Err("caller is not a controller of the token".to_string());
            }
            if input.id > 0 {
                match token.announcements.last_mut() {
                    Some(last) => {
                        if last.id != input.id {
                            return Err("announcement id is invalid".to_string());
                        }
                        last.r#type = input.r#type;
                        last.title = input.title;
                        last.content = input.content;
                        last.url = input.url;
                        last.published_at = now_ms;
                    }
                    None => {
                        return Err("announcement id is invalid".to_string());
                    }
                }
            } else {
                let announcement = AnnouncementState {
                    id: token.announcements.len() as u64 + 1,
                    r#type: input.r#type,
                    title: input.title,
                    content: input.content,
                    url: input.url,
                    published_at: now_ms,
                };
                token.announcements.push(announcement);
            }
            t.insert(id, token);
            Ok(())
        })
    }

    pub fn admin_update_token_status(
        id: u64,
        status: TokenStatus,
        now_ms: u64,
    ) -> Result<(), String> {
        TOKENS.with_borrow_mut(|t| {
            let mut profile = t
                .get(&id)
                .ok_or_else(|| format!("Token with id {} not found", id))?;
            profile.status = status;
            profile.updated_at = now_ms;
            t.insert(id, profile);
            Ok(())
        })
    }

    pub fn admin_update_token_tags(id: u64, tags: Vec<String>, now_ms: u64) -> Result<(), String> {
        TOKENS.with_borrow_mut(|t| {
            let mut profile = t
                .get(&id)
                .ok_or_else(|| format!("Token with id {} not found", id))?;
            profile.tags = tags.into_iter().collect();
            profile.updated_at = now_ms;
            t.insert(id, profile);
            Ok(())
        })
    }

    pub fn admin_update_token_verification_badge(
        id: u64,
        badge: VerificationBadge,
        now_ms: u64,
    ) -> Result<(), String> {
        TOKENS.with_borrow_mut(|t| {
            let mut profile = t
                .get(&id)
                .ok_or_else(|| format!("Token with id {} not found", id))?;
            profile.verification = VerificationBadgeState {
                is_verified: badge.is_verified,
                methods: badge.methods,
                verified_at: now_ms,
            };
            profile.updated_at = now_ms;
            t.insert(id, profile);
            Ok(())
        })
    }

    pub fn query_token(q: String) -> Vec<(u64, TokenMetadata)> {
        STATE.with_borrow(|s| {
            let mut result: Vec<(u64, TokenMetadata)> = Vec::with_capacity(10);
            if let Some(id) = s.location_index.get(&q)
                && let Some(token) = s.tokens.get(id) {
                    result.push((*id, token.into()));
                    return result;
                }

            if let Some(ids) = s.inverted_index.get(&q) {
                for id in ids {
                    if let Some(token) = s.tokens.get(id) {
                        result.push((*id, token.into()));
                    }
                }
                if result.len() >= 10 {
                    return result;
                }
            }

            for (id, token) in s.tokens.iter() {
                if token.symbol.eq_ignore_ascii_case(&q) || token.name.eq_ignore_ascii_case(&q) {
                    result.push((*id, token.into()));
                }
                if result.len() >= 10 {
                    break;
                }
            }
            result
        })
    }

    pub fn list_tokens(take: usize, prev_id: Option<u64>) -> Vec<(u64, TokenMetadata)> {
        STATE.with_borrow(|s| {
            let started = prev_id.unwrap_or_default();
            s.tokens
                .iter()
                .filter(|(k, _)| k > &&started)
                .take(take)
                .map(|(id, token)| (*id, token.into()))
                .collect()
        })
    }

    pub fn get_token_by_location(loc: &str) -> Result<TokenMetadata, String> {
        STATE.with_borrow(|s| {
            let id = s
                .location_index
                .get(loc)
                .ok_or_else(|| format!("Token with location {} not found", loc))?;
            let token = s
                .tokens
                .get(id)
                .ok_or_else(|| format!("Token with id {} not found", id))?;
            Ok(token.into())
        })
    }

    pub fn get_token_profile(id: u64) -> Result<TokenProfile, String> {
        TOKENS.with_borrow(|t| {
            t.get(&id)
                .map(|token| token.into())
                .ok_or_else(|| format!("Token with id {} not found", id))
        })
    }

    pub fn list_uniswap_tokens() -> Vec<UniswapToken> {
        STATE.with_borrow(|s| {
            let mut result: Vec<UniswapToken> = Vec::new();
            for (_, token) in s.tokens.iter() {
                for loc in &token.locations {
                    if let Ok(chain) = ChainLocation::from_str(loc)
                        && let Ok(chain_id) = u64::from_str(&chain.reference) {
                            result.push(UniswapToken {
                                chain_id,
                                symbol: token.symbol.clone(),
                                name: token.name.clone(),
                                decimals: token.decimals,
                                logo_uri: token.image.clone(),
                                address: chain.asset_reference.clone(),
                            });
                        }
                }
                if result.len() >= 10000 {
                    break;
                }
            }
            result
        })
    }
}
