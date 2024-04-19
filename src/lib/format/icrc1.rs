use candid::{Decode, Nat};
use icrc_ledger_types::icrc1::transfer::TransferError;

use crate::lib::{e8s_to_tokens, AnyhowResult};

pub fn display_transfer(blob: &[u8]) -> AnyhowResult<String> {
    let result = Decode!(blob, Result<Nat, TransferError>)?;
    match result {
        Ok(index) => Ok(format!("Transfer sent at block index {index}")),
        Err(e) => Ok(format!("Transfer error: {e}")),
    }
}

pub fn display_balance(blob: &[u8]) -> AnyhowResult<String> {
    let balance = Decode!(blob, Nat)?;
    Ok(format!("Balance: {}", e8s_to_tokens(balance))) // we do not use any ICRC1 calls to ledgers with digits other than 8
}
