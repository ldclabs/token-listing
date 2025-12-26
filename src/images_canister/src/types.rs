use candid::{CandidType, Principal};
use ic_auth_types::ByteBufB64;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize)]
pub struct StateInfo {
    pub total_images: u64,
    pub tokens_canister: Principal,
    pub governance_canister: Option<Principal>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct ImageMetadata {
    pub name: String,   // file name with extension
    pub r#type: String, // MIME, e.g., "image/png"
    pub size: usize,
    pub created_at: u64, // unix timestamp in milliseconds
    pub updated_at: u64, // unix timestamp in milliseconds
    pub updated_by: Principal,
    pub locations: Vec<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct ImageInput {
    pub name: String,   // file name with extension
    pub r#type: String, // MIME, e.g., "image/png"
    pub body: ByteBufB64,
}

impl ImageInput {
    pub fn validate(&self) -> Result<(), String> {
        let len = self.name.trim().len();
        if len == 0 || len > 64 {
            return Err("Invalid name length".to_string());
        }
        if len != self.name.len() {
            return Err("Name contains leading or trailing whitespace".to_string());
        }
        let len = self.r#type.trim().len();
        if len == 0 || len > 32 {
            return Err("Invalid type length".to_string());
        }
        if len != self.r#type.len() {
            return Err("Type contains leading or trailing whitespace".to_string());
        }
        Ok(())
    }
}
