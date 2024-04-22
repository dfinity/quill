use candid::Decode;
use icp_ledger::{Tokens, TransferError};

use crate::lib::{e8s_to_tokens, AnyhowResult};

pub fn display_transfer(blob: &[u8]) -> AnyhowResult<String> {
    let result = Decode!(blob, Result<u64, TransferError>)?;
    match result {
        Ok(index) => Ok(format!("Transfer sent at block index {index}")),
        Err(e) => Ok(format!("Transfer error: {e}")),
    }
}

pub fn display_send_dfx(blob: &[u8]) -> AnyhowResult<String> {
    let index = Decode!(blob, u64)?;
    Ok(format!("Transfer sent at block index {index}"))
}

pub fn display_account_balance_or_dfx(blob: &[u8]) -> AnyhowResult<String> {
    let tokens = Decode!(blob, Tokens)?;
    Ok(format!(
        "Balance: {} ICP",
        e8s_to_tokens(tokens.get_e8s().into())
    ))
}
