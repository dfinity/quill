use candid::{Decode, Nat};
use icrc_ledger_types::icrc1::transfer::TransferError;

use crate::lib::AnyhowResult;

pub fn display_transfer(blob: &[u8]) -> AnyhowResult<String> {
    let result = Decode!(blob, Result<Nat, TransferError>)?;
    match result {
        Ok(index) => Ok(format!("Transfer sent at block index {index}")),
        Err(e) => Ok(format!("Transfer error: {e}")),
    }
}

pub fn display_balance(blob: &[u8]) -> AnyhowResult<String> {
    let balance = Decode!(blob, Nat)?;
    Ok(format!("Balance: {balance}"))
}
