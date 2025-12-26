use candid::Principal;

use crate::{helper::pretty_format, store};

#[ic_cdk::update(guard = "is_controller")]
fn admin_set_tokens_canister(tokens_canister: Principal) -> Result<(), String> {
    store::state::with_mut(|s| {
        s.tokens_canister = tokens_canister;
        Ok(())
    })
}

#[ic_cdk::update]
fn validate_admin_set_tokens_canister(tokens_canister: Principal) -> Result<String, String> {
    pretty_format(&(tokens_canister,))
}

fn is_controller() -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    if ic_cdk::api::is_controller(&caller)
        || store::state::with(|s| s.governance_canister == Some(caller))
    {
        Ok(())
    } else {
        Err("user is not a controller".to_string())
    }
}
