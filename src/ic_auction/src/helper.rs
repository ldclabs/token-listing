use candid::{
    CandidType, IDLValue, Principal, pretty::candid::value::pp_value, utils::ArgumentEncoder,
};
use icrc_ledger_types::{
    icrc::generic_value::{ICRC3Value, Value},
    icrc1::account::Account,
    icrc3::blocks::{GetBlocksRequest, GetBlocksResult},
};
use num_traits::ToPrimitive;
use std::collections::BTreeSet;

use crate::types::TransferChecked;

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

pub fn convert_amount(
    src_amount: u128,
    src_decimals: u8,
    target_decimals: u8,
) -> Result<u128, String> {
    if src_decimals == target_decimals {
        Ok(src_amount)
    } else if src_decimals < target_decimals {
        let factor = 10u128
            .checked_pow((target_decimals - src_decimals) as u32)
            .ok_or_else(|| "exponent too large".to_string())?;
        src_amount
            .checked_mul(factor)
            .ok_or_else(|| "multiplication overflow".to_string())
    } else {
        let factor = 10u128
            .checked_pow((src_decimals - target_decimals) as u32)
            .ok_or_else(|| "exponent too large".to_string())?;
        Ok(src_amount / factor)
    }
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

pub fn pretty_format<T>(data: &T) -> Result<String, String>
where
    T: CandidType,
{
    let val = IDLValue::try_from_candid_type(data).map_err(|err| format!("{err:?}"))?;
    let doc = pp_value(7, &val);

    Ok(format!("{}", doc.pretty(120)))
}

pub async fn get_icrc_transfer(
    ledger: Principal,
    block_index: u64,
) -> Result<TransferChecked, String> {
    let req = GetBlocksRequest {
        start: block_index.into(),
        length: 1u64.into(),
    };

    let res: GetBlocksResult = call(ledger, "icrc3_get_blocks", (vec![req],), 0).await?;

    let block = res.blocks.first().ok_or("Block not found")?;

    let tx_map = match &block.block {
        ICRC3Value::Map(map) => match map.get("tx") {
            Some(ICRC3Value::Map(tx)) => tx,
            _ => return Err("Block missing tx field or invalid format".to_string()),
        },
        _ => return Err("Invalid block format".to_string()),
    };

    // Check op
    match tx_map.get("op") {
        Some(ICRC3Value::Text(op)) if op == "xfer" => {}
        Some(ICRC3Value::Text(op)) => return Err(format!("Invalid operation: {}", op)),
        _ => return Err("Missing or invalid op field".to_string()),
    }

    // Check from
    let from_val = tx_map.get("from").ok_or("Missing from field")?;
    let from_acc = Account::try_from(Value::from(from_val.to_owned()))?;

    // Check to
    let to_val = tx_map.get("to").ok_or("Missing to field")?;
    let to_acc = Account::try_from(Value::from(to_val.to_owned()))?;
    // Check amount
    let amt_val = tx_map.get("amt").ok_or("Missing amt field")?;
    let amount = match amt_val {
        ICRC3Value::Nat(n) => n.0.to_u128().ok_or("Amount too large")?,
        ICRC3Value::Int(i) => i.0.to_u128().ok_or("Amount too large")?,
        _ => return Err("Invalid amount format".to_string()),
    };

    Ok(TransferChecked {
        token: ledger.to_string(),
        from: from_acc.to_string(),
        to: to_acc.to_string(),
        amount,
    })
}
