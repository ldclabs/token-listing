use candid::{CandidType, Principal};
use ic_auth_types::ByteArrayB64;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(CandidType, Serialize, Deserialize)]
pub struct StateInfo {
    pub total_tokens: u64,
    pub x402_paying_public_keys: Vec<ByteArrayB64<32>>,
    pub x402_prices: BTreeMap<String, u64>,
    pub x402_pay_to: String,
    pub governance_canister: Option<Principal>,
}

/// Token Metadata following CAIP-390 standard:
// ```json
// {
//   "name": "ICPanda",
//   "symbol": "PANDA",
//   "decimals": 8,
//   "image": "https://panda.fans/_assets/logo.svg",
//   "description": "Building the open-source stack for AI agents to remember, transact, and evolve as first-class citizens in Web3.",
//   "external_url": "https://panda.fans",
//   "links": [
//     {
//       "name": "Twitter",
//       "url": "https://x.com/ICPandaDAO",
//       "rel": "social"
//     },
//     {
//       "name": "Source Code",
//       "url": "https://github.com/ldclabs/ic-panda",
//       "rel": "source_code"
//     },
//     {
//       "name": "ICPSwap",
//       "url": "https://app.icpswap.com/swap/pro?input=ryjl3-tyaaa-aaaaa-aaaba-cai&output=druyg-tyaaa-aaaaq-aactq-cai",
//       "rel": "exchange"
//     },
//     {
//       "name": "Official Bridge",
//       "url": "https://1bridge.app/?token=PANDA",
//       "rel": "bridge"
//     }
//   ],
//   "locations": [
//     "icp:1/token:druyg-tyaaa-aaaaq-aactq-cai",
//     "eip155:56/bep20:0xe74583edaff618d88463554b84bc675196b36990",
//     "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp/token:PANDAvvWniWYKRbrCYQEAyeSJ5uUk1nc49eLqT6yQyL"
//   ]
// }
// ```
#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub image: String,
    pub description: String,
    pub external_url: String,
    pub links: Vec<LinkItem>,
    pub locations: Vec<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct UniswapToken {
    #[serde(rename = "chainId")]
    pub chain_id: u64,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(rename = "logoURI")]
    pub logo_uri: String,
    pub address: String,
}

/// CAIP-2: chain_id = namespace + ":" + reference
/// CAIP-19: asset_id = chain_id + "/" + asset_namespace + ":" + asset_reference
#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct ChainLocation {
    pub namespace: String,       // e.g., "icp", "eip155", "solana"
    pub reference: String,       // e.g., 1 (ETH), 56 (BSC). ICP 1
    pub asset_namespace: String, // e.g., "erc20", "token", "bep20"
    pub asset_reference: String, // e.g., contract address or canister ID
}

impl std::str::FromStr for ChainLocation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid chain location format".to_string());
        }
        let chain_parts: Vec<&str> = parts[0].split(':').collect();
        if chain_parts.len() != 2 {
            return Err("Invalid chain ID format".to_string());
        }
        let asset_parts: Vec<&str> = parts[1].split(':').collect();
        if asset_parts.len() != 2 {
            return Err("Invalid asset ID format".to_string());
        }
        Ok(ChainLocation {
            namespace: chain_parts[0].to_string(),
            reference: chain_parts[1].to_string(),
            asset_namespace: asset_parts[0].to_string(),
            asset_reference: asset_parts[1].to_string(),
        })
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct LinkItem {
    pub name: String,
    pub url: String,
    pub rel: LinkType,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    /// Main project website.
    Homepage,
    /// Technical paper or economic model.
    Whitepaper,
    /// Developer docs or wikis.
    Documentation,
    /// Code repositories (GitHub, GitLab).
    SourceCode,
    /// Voting portals or forums.
    Governance,
    /// Security audit reports.
    Audit,
    /// Social media profiles (X/Twitter, Discord, Telegram).
    Social,
    /// Block explorer links.
    Browser,
    /// Direct links to trading pairs on DEXs or CEXs (e.g., Uniswap Pool, Binance Spot).
    Exchange,
    /// Interfaces allowing users to bridge this asset across chains.
    Bridge,
}

/// Auction Information
#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct TokenProfile {
    pub id: u64,
    pub controllers: Vec<Principal>,
    pub status: TokenStatus,
    pub created_at: u64, // Unix timestamp in milliseconds
    pub updated_at: u64,
    pub metadata: TokenMetadata,
    pub tags: Vec<String>, // e.g., ["DeFi", "Meme", "AI", "DAO"]
    pub verification: VerificationBadge,
    pub announcements: Vec<Announcement>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenStatus {
    Active,
    Locked,
    Deprecated,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct VerificationBadge {
    pub is_verified: bool,
    pub methods: Vec<String>, // ["dns_txt_record", "admin_wallet_signature", "dao_vote"]
    pub verified_at: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct Announcement {
    pub id: u64,
    pub r#type: String, // listing, governance, upgrade, ...
    pub title: String,
    pub content: String, // Markdown content
    pub url: Option<String>,
    pub published_at: u64,
}
