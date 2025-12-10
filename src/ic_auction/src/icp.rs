use candid::{Nat, Principal};
use icrc_ledger_types::{
    icrc::generic_value::{ICRC3Value, Value},
    icrc1::{account::Account, transfer::TransferArg},
    icrc2::approve::{ApproveArgs, ApproveError},
    icrc2::transfer_from::TransferFromError,
    icrc3::blocks::{GetBlocksRequest, GetBlocksResult},
};
use num_traits::ToPrimitive;

pub mod kong;

use crate::{helper::call, types::TransferChecked};

pub async fn balance_of(ledger: Principal, account: Account) -> Result<u128, String> {
    let res: Nat = call(ledger, "icrc1_balance_of", (account,), 0).await?;
    res.0.to_u128().ok_or("Balance too large".to_string())
}

// return block index
pub async fn transfer(ledger: Principal, to: Account, amount: Nat) -> Result<String, String> {
    let res: Result<Nat, TransferFromError> = call(
        ledger,
        "icrc1_transfer",
        (TransferArg {
            from_subaccount: None,
            to,
            fee: None,
            created_at_time: None,
            memo: None,
            amount,
        },),
        0,
    )
    .await?;
    let res = res.map_err(|err| format!("ICP: failed to transfer token, error: {:?}", err))?;
    Ok(res.0.to_string())
}

pub async fn approve(ledger: Principal, spender: Account, amount: Nat) -> Result<String, String> {
    let res: Result<Nat, ApproveError> = call(
        ledger,
        "icrc2_approve",
        (ApproveArgs {
            from_subaccount: None,
            spender,
            amount,
            expected_allowance: Some(0u64.into()),
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        },),
        0,
    )
    .await?;
    let res = res.map_err(|err| format!("ICP: failed to approve token, error: {:?}", err))?;
    Ok(res.0.to_string())
}

pub async fn verify_transfer_token(
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
