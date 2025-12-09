use candid::{
    CandidType, IDLValue, Principal, pretty::candid::value::pp_value, utils::ArgumentEncoder,
};
use std::collections::BTreeSet;

const ANONYMOUS: Principal = Principal::anonymous();

pub static APP_AGENT: &str = concat!(
    "Mozilla/5.0 ICP canister ",
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

pub fn msg_caller() -> Result<Principal, String> {
    let caller = ic_cdk::api::msg_caller();
    check_auth(&caller)?;
    Ok(caller)
}

pub fn check_auth(user: &Principal) -> Result<(), String> {
    if user == &ANONYMOUS {
        Err("anonymous user is not allowed".to_string())
    } else {
        Ok(())
    }
}

pub fn validate_principals(principals: &BTreeSet<Principal>) -> Result<(), String> {
    if principals.is_empty() {
        return Err("principals cannot be empty".to_string());
    }
    if principals.contains(&ANONYMOUS) {
        return Err("anonymous user is not allowed".to_string());
    }
    Ok(())
}

pub fn format_error<T>(err: T) -> String
where
    T: std::fmt::Debug,
{
    format!("{:?}", err)
}

pub async fn call<In, Out>(
    id: Principal,
    method: &str,
    args: In,
    cycles: u128,
) -> Result<Out, String>
where
    In: ArgumentEncoder + Send,
    Out: candid::CandidType + for<'a> candid::Deserialize<'a>,
{
    let res = ic_cdk::call::Call::bounded_wait(id, method)
        .with_args(&args)
        .with_cycles(cycles)
        .await
        .map_err(|err| format!("failed to call {} on {:?}, error: {:?}", method, &id, err))?;
    res.candid().map_err(|err| {
        format!(
            "failed to decode response from {} on {:?}, error: {:?}",
            method, &id, err
        )
    })
}

pub async fn call_raw<In>(
    id: Principal,
    method: &str,
    args: In,
    cycles: u128,
) -> Result<ic_cdk::call::Response, String>
where
    In: ArgumentEncoder + Send,
{
    let res = ic_cdk::call::Call::bounded_wait(id, method)
        .with_args(&args)
        .with_cycles(cycles)
        .await
        .map_err(|err| format!("failed to call {} on {:?}, error: {:?}", method, &id, err))?;
    Ok(res)
}

pub fn pretty_format<T>(data: &T) -> Result<String, String>
where
    T: CandidType,
{
    let val = IDLValue::try_from_candid_type(data).map_err(|err| format!("{err:?}"))?;
    let doc = pp_value(7, &val);

    Ok(format!("{}", doc.pretty(120)))
}
