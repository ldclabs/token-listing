use candid::Principal;
use std::collections::BTreeSet;

use crate::{helper, store, types};

#[ic_cdk::query]
fn info() -> Result<types::StateInfo, String> {
    Ok(store::state::info())
}

#[ic_cdk::query]
fn get_image_metadata(id: u64) -> Result<types::ImageMetadata, String> {
    store::state::get_image_metadata(id).ok_or_else(|| format!("Image with id {} not found", id))
}

#[ic_cdk::update]
async fn update_image(token_id: u64, input: types::ImageInput) -> Result<u64, String> {
    let caller = helper::msg_caller()?;
    let now_ms = ic_cdk::api::time() / 1_000_000;
    if input.body.len() > 64 * 1024 {
        return Err("Image size exceeds 64 KB limit".to_string());
    }
    input.validate()?;
    let canister = store::state::with(|s| s.tokens_canister);
    let rt: Result<Vec<String>, String> =
        helper::call(canister, "check_permission", (token_id, caller), 0).await?;
    let locations = rt?;
    if locations.is_empty() {
        return Err("No location on token for storing images".to_string());
    }

    let image = store::ImageMetadataState {
        name: input.name,
        r#type: input.r#type,
        size: input.body.len(),
        created_at: now_ms,
        updated_at: now_ms,
        updated_by: caller,
        locations,
    };
    store::state::update_image(token_id, image, input.body.into_vec())
}
