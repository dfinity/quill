use askama::Template;
use candid::{Decode, Nat, Principal};
use icp_ledger::{Tokens, TransferError};

use crate::lib::{format::filters, ledger_canister_id, AnyhowResult};

pub fn display_icp_transfer(blob: &[u8]) -> AnyhowResult<String> {
    let result = Decode!(blob, Result<u64, TransferError>)?;
    match result {
        Ok(index) => Ok(Transfer {
            index: index.into(),
        }
        .render()?),
        Err(error) => Ok(TransferErr { error }.render()?),
    }
}

use TransferError::*;
#[derive(Template)]
#[template(path = "ledger/transfer_err.txt")]
struct TransferErr {
    error: TransferError,
}

#[derive(Template)]
#[template(path = "ledger/transfer.txt")]
struct Transfer {
    index: Nat,
}

#[derive(Template)]
#[template(path = "ledger/balance.txt")]
struct Balance {
    balance: Nat,
    canister: Principal,
}

pub fn display_send_dfx(blob: &[u8]) -> AnyhowResult<String> {
    let index = Decode!(blob, u64)?;
    Ok(Transfer {
        index: index.into(),
    }
    .render()?)
}

pub fn display_account_balance_or_dfx(blob: &[u8]) -> AnyhowResult<String> {
    let tokens = Decode!(blob, Tokens)?;
    Ok(Balance {
        canister: ledger_canister_id(),
        balance: tokens.get_e8s().into(),
    }
    .render()?)
}

pub fn display_transfer(blob: &[u8]) -> AnyhowResult<String> {
    let result = Decode!(blob, Result<Nat, TransferError>)?;
    match result {
        Ok(index) => Ok(Transfer { index }.render()?),
        Err(error) => Ok(TransferErr { error }.render()?),
    }
}

pub fn display_balance(blob: &[u8], canister: Principal) -> AnyhowResult<String> {
    let balance = Decode!(blob, Nat)?;
    Ok(Balance { canister, balance }.render()?) // we do not use any ICRC1 calls to ledgers with digits other than 8
}
