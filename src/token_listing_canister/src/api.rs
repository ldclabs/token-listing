use crate::{store, types};

#[ic_cdk::query]
fn info() -> Result<types::StateInfo, String> {
    Ok(store::state::info())
}

#[ic_cdk::query]
fn get_auction(id: Option<types::AuctionId>) -> Option<types::AuctionInfo> {
    let id = match id {
        Some(id) => store::AuctionAddress::try_from(&id).ok(),
        None => store::state::with(|s| s.auctions.last().cloned()),
    };

    match id {
        Some(id) => store::state::get_auction(id),
        None => None,
    }
}

#[ic_cdk::query]
fn list_auctions(take: usize, prev_id: Option<types::AuctionId>) -> Vec<types::AuctionInfo> {
    store::state::list_auctions(take.min(1000), prev_id)
}
