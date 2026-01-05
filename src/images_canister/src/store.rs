use candid::Principal;
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
use std::{borrow::Cow, cell::RefCell, collections::BTreeMap};

use crate::types::{ImageMetadata, StateInfo};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Serialize, Deserialize)]
pub struct State {
    pub images: BTreeMap<u64, ImageMetadataState>, // images 数量不会太多，不可能超过内存限制
    pub location_index: BTreeMap<String, u64>,     // key: unique location -> token id
    pub tokens_canister: Principal,
    pub governance_canister: Option<Principal>,
}

impl From<&State> for StateInfo {
    fn from(s: &State) -> Self {
        Self {
            total_images: s.images.len() as u64,
            tokens_canister: s.tokens_canister,
            governance_canister: s.governance_canister,
        }
    }
}

impl State {
    fn new() -> Self {
        Self {
            images: BTreeMap::new(),
            location_index: BTreeMap::new(),
            tokens_canister: Principal::anonymous(),
            governance_canister: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageMetadataState {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "t")]
    pub r#type: String,
    #[serde(rename = "s")]
    pub size: usize,
    #[serde(rename = "ca")]
    pub created_at: u64,
    #[serde(rename = "ua")]
    pub updated_at: u64,
    #[serde(rename = "u")]
    pub updated_by: Principal,
    #[serde(rename = "l")]
    pub locations: Vec<String>,
}

impl From<&ImageMetadataState> for ImageMetadata {
    fn from(s: &ImageMetadataState) -> Self {
        Self {
            name: s.name.clone(),
            r#type: s.r#type.clone(),
            size: s.size,
            created_at: s.created_at,
            updated_at: s.updated_at,
            updated_by: s.updated_by,
            locations: s.locations.clone(),
        }
    }
}

impl From<ImageMetadataState> for ImageMetadata {
    fn from(s: ImageMetadataState) -> Self {
        Self {
            name: s.name,
            r#type: s.r#type,
            size: s.size,
            created_at: s.created_at,
            updated_at: s.updated_at,
            updated_by: s.updated_by,
            locations: s.locations,
        }
    }
}

impl Storable for ImageMetadataState {
    const BOUND: Bound = Bound::Unbounded;

    fn into_bytes(self) -> Vec<u8> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode ImageMetadataState data");
        buf
    }

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![];
        into_writer(&self, &mut buf).expect("failed to encode ImageMetadataState data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode ImageMetadataState data")
    }
}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const IMAGES_MEMORY_ID: MemoryId = MemoryId::new(1);

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

    static IMAGES: RefCell<StableBTreeMap<u64, Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(IMAGES_MEMORY_ID)),
        )
    );
}

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

    pub fn update_image(
        id: u64,
        mut image: ImageMetadataState,
        data: Vec<u8>,
    ) -> Result<u64, String> {
        STATE.with_borrow_mut(|s| {
            if let Some(existing) = s.images.get_mut(&id) {
                // update existing image
                image.created_at = existing.created_at;

                for loc in &existing.locations {
                    s.location_index.remove(&loc.to_ascii_lowercase());
                }
            }

            for loc in &image.locations {
                s.location_index.insert(loc.to_ascii_lowercase(), id);
            }

            s.images.insert(id, image);
            IMAGES.with_borrow_mut(|i| {
                i.insert(id, data);
            });
            Ok(id)
        })
    }

    pub fn get_image(
        loc: &str,
        if_modified_since: Option<u64>, // millis
    ) -> Option<(ImageMetadata, Option<Vec<u8>>)> {
        STATE.with_borrow(|s| {
            let id: u64 = match s.location_index.get(loc) {
                Some(id) => *id,
                None => loc.parse().ok()?,
            };

            if let Some(image) = s.images.get(&id) {
                if let Some(ims) = if_modified_since
                    && image.updated_at <= ims
                {
                    return Some((image.into(), None));
                }

                if let Some(data) = IMAGES.with_borrow(|i| i.get(&id)) {
                    return Some((image.into(), Some(data)));
                }
            }

            None
        })
    }

    pub fn get_image_metadata(id: u64) -> Option<ImageMetadata> {
        STATE.with_borrow(|s| s.images.get(&id).map(|m| m.into()))
    }
}
