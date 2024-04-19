use candid::Decode;
use ic_ckbtc_minter::{
    state::{ReimbursementReason, RetrieveBtcStatus, RetrieveBtcStatusV2},
    updates::{
        retrieve_btc::{RetrieveBtcError, RetrieveBtcOk},
        update_balance::{UpdateBalanceError, UtxoStatus},
    },
};
use std::fmt::Write;

use crate::lib::{e8s_to_tokens, AnyhowResult};

pub fn display_update_balance(blob: &[u8]) -> AnyhowResult<String> {
    let result = Decode!(blob, Result<Vec<UtxoStatus>, UpdateBalanceError>)?;
    let fmt = match result {
        Ok(statuses) => {
            let mut fmt = String::new();
            for status in statuses {
                match status {
                    UtxoStatus::Minted { block_index, minted_amount, utxo } => writeln!(fmt, "{txid}({btc} BTC): Minted {ckbtc} ckBTC at block index {block_index}", txid = utxo.outpoint.txid, btc = e8s_to_tokens(utxo.value.into()), ckbtc = e8s_to_tokens(minted_amount.into()))?,
                    UtxoStatus::ValueTooSmall(utxo) => writeln!(fmt,"{txid}({btc} BTC): UTXO rejected: too small to cover KYT cost", txid = utxo.outpoint.txid, btc = e8s_to_tokens(utxo.value.into()))?,
                    UtxoStatus::Tainted(utxo) => writeln!(fmt, "{txid}({btc} BTC): UTXO rejected: the KYT process determined the BTC is tainted", txid = utxo.outpoint.txid, btc = e8s_to_tokens(utxo.value.into()))?,
                    UtxoStatus::Checked(utxo) => writeln!(fmt, "{txid}({btc} BTC): The deposted BTC cleared the KYT check, but minting ckBTC failed. Retry this command.", txid = utxo.outpoint.txid, btc = e8s_to_tokens(utxo.value.into()))?,
                }
            }
            fmt
        }
        Err(e) => match e {
            UpdateBalanceError::GenericError { error_message, .. } => {
                format!("ckBTC error: {error_message}")
            }
            UpdateBalanceError::AlreadyProcessing => {
                "ckBTC error: already processing another update_balance call for the same account"
                    .to_string()
            }
            UpdateBalanceError::NoNewUtxos {
                current_confirmations,
                required_confirmations,
                pending_utxos,
            } => {
                let mut fmt = "ckBTC error: no new confirmed UTXOs to process".to_string();
                if let Some(pending_utxos) = pending_utxos {
                    write!(
                        fmt,
                        " ({} unconfirmed, needing {} confirmations but having {})",
                        pending_utxos.len(),
                        required_confirmations,
                        current_confirmations.unwrap_or_default()
                    )?;
                }
                fmt
            }
            UpdateBalanceError::TemporarilyUnavailable(e) => {
                format!("ckBTC error: temporarily unavailable: {e}. Try again later.")
            }
        },
    };
    Ok(fmt)
}

pub fn display_retrieve_btc(blob: &[u8]) -> AnyhowResult<String> {
    let result = Decode!(blob, Result<RetrieveBtcOk, RetrieveBtcError>)?;
    let fmt = match result {
        Ok(ok) => format!("Begun retrieval process at block index {}", ok.block_index),
        Err(e) => match e {
            RetrieveBtcError::GenericError { error_message, .. } => {
                format!("ckBTC error: {error_message}")
            }
            RetrieveBtcError::AmountTooLow(min) => format!(
                "ckBTC error: amount too low to withdraw (min: {})",
                e8s_to_tokens(min.into())
            ),
            RetrieveBtcError::InsufficientFunds { balance } => format!(
                "ckBTC error: the withdrawal account does not have enough ckBTC (balance: {})",
                e8s_to_tokens(balance.into())
            ),
            RetrieveBtcError::MalformedAddress(msg) => {
                format!("ckBTC error: malformed address: {msg}")
            }
            RetrieveBtcError::AlreadyProcessing => {
                "ckBTC error: already processing another retrieve_btc call for the same account"
                    .to_string()
            }
            RetrieveBtcError::TemporarilyUnavailable(msg) => {
                format!("ckBTC error: temporarily unavailable: {msg}")
            }
        },
    };
    Ok(fmt)
}

pub fn display_retrieve_btc_status(blob: &[u8]) -> AnyhowResult<String> {
    let status = Decode!(blob, RetrieveBtcStatus)?;
    Ok(display_retrieve_btc_status_internal(status.into()))
}

pub fn display_retrieve_btc_status_v2(blob: &[u8]) -> AnyhowResult<String> {
    let status = Decode!(blob, RetrieveBtcStatusV2)?;
    Ok(display_retrieve_btc_status_internal(status))
}

fn display_retrieve_btc_status_internal(status: RetrieveBtcStatusV2) -> String {
    match status {
        RetrieveBtcStatusV2::AmountTooLow => "ckBTC error: amount too low to withdraw".to_string(),
        RetrieveBtcStatusV2::Unknown => "ckBTC error: request ID invalid or too old".to_string(),
        RetrieveBtcStatusV2::Pending => "The BTC transaction is pending in the queue".to_string(),
        RetrieveBtcStatusV2::Signing => "The BTC transaction is being signed".to_string(),
        RetrieveBtcStatusV2::Sending { txid } => format!("The BTC transaction is being sent (id {txid})"),
        RetrieveBtcStatusV2::Submitted { txid } => format!("The BTC transaction has been sent, awaiting confirmations (id {txid})"),
        RetrieveBtcStatusV2::Confirmed { txid } => format!("The BTC transaction has been completed (id {txid})"),
        RetrieveBtcStatusV2::WillReimburse(task) => match task.reason {
            ReimbursementReason::CallFailed => format!("The BTC transaction failed. {amount} ckBTC is being reimbursed to {account}", amount = e8s_to_tokens(task.amount.into()), account = task.account),
            ReimbursementReason::TaintedDestination { kyt_provider, kyt_fee } => format!("The KYT process determined that the BTC destination is tainted. {amount} ckBTC is being reimbursed to {account}\nKYT fee: {fee}, provider: {kyt_provider}", amount = e8s_to_tokens(task.amount.into()), fee = e8s_to_tokens(kyt_fee.into()), account = task.account)
        }
        RetrieveBtcStatusV2::Reimbursed(reimbursed) => match reimbursed.reason {
            ReimbursementReason::CallFailed => format!("The BTC transaction failed. {amount} ckBTC has been reimbursed to {account} at block index {index}", amount = reimbursed.amount, account = reimbursed.account, index = reimbursed.mint_block_index),
            ReimbursementReason::TaintedDestination { kyt_provider, kyt_fee } => format!("The KYT process determined that the BTC destination is tainted. {amount} ckBTC has been reimbursed to {account} at block index {index}\nKYT fee: {fee}, provider: {kyt_provider}", amount = e8s_to_tokens(reimbursed.amount.into()), account = reimbursed.account, index = reimbursed.mint_block_index, fee = e8s_to_tokens(kyt_fee.into()))
        }
    }
}
