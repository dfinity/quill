use askama::Template;
use candid::Decode;
use ic_ckbtc_minter::{
    state::{ReimbursementReason, RetrieveBtcStatus, RetrieveBtcStatusV2},
    updates::{
        retrieve_btc::{RetrieveBtcError, RetrieveBtcOk},
        update_balance::{UpdateBalanceError, UtxoStatus},
    },
};

use crate::lib::{format::filters, AnyhowResult};

pub fn display_update_balance(blob: &[u8]) -> AnyhowResult<String> {
    let result = Decode!(blob, Result<Vec<UtxoStatus>, UpdateBalanceError>)?;
    let fmt = match result {
        Ok(statuses) => {
            use UtxoStatus::*;
            #[derive(Template)]
            #[template(path = "ckbtc/update_balance.txt")]
            struct UpdateBalance {
                statuses: Vec<UtxoStatus>,
            }
            UpdateBalance { statuses }.render()?
        }
        Err(error) => {
            use Err::*;
            #[derive(Template)]
            #[template(path = "ckbtc/update_balance_err.txt")]
            struct UpdateBalanceErr {
                error: Err,
            }
            enum Err {
                GenericError {
                    error_message: String,
                },
                AlreadyProcessing,
                NoNewUtxos {
                    current: u32,
                    required: u32,
                    pending: Option<usize>,
                    suspended: usize,
                },
                TemporarilyUnavailable(String),
            }
            UpdateBalanceErr {
                error: match error {
                    UpdateBalanceError::AlreadyProcessing => Err::AlreadyProcessing,
                    UpdateBalanceError::GenericError { error_message, .. } => {
                        Err::GenericError { error_message }
                    }
                    UpdateBalanceError::NoNewUtxos {
                        current_confirmations,
                        required_confirmations,
                        pending_utxos,
                        suspended_utxos,
                    } => Err::NoNewUtxos {
                        current: current_confirmations.unwrap_or_default(),
                        required: required_confirmations,
                        pending: pending_utxos.map(|v| v.len()),
                        suspended: suspended_utxos.unwrap_or_default().len(),
                    },
                    UpdateBalanceError::TemporarilyUnavailable(msg) => {
                        Err::TemporarilyUnavailable(msg)
                    }
                },
            }
            .render()?
        }
    };
    Ok(fmt)
}

pub fn display_retrieve_btc(blob: &[u8]) -> AnyhowResult<String> {
    let result = Decode!(blob, Result<RetrieveBtcOk, RetrieveBtcError>)?;
    let fmt = match result {
        Ok(status) => {
            #[derive(Template)]
            #[template(path = "ckbtc/retrieve_btc.txt")]
            struct RetrieveBtc {
                status: RetrieveBtcOk,
            }
            RetrieveBtc { status }.render()?
        }
        Err(error) => {
            use RetrieveBtcError::*;
            #[derive(Template)]
            #[template(path = "ckbtc/retrieve_btc_err.txt")]
            struct RetrieveBtcErr {
                error: RetrieveBtcError,
            }
            RetrieveBtcErr { error }.render()?
        }
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
    use ReimbursementReason::*;
    use RetrieveBtcStatusV2::*;
    #[derive(Template)]
    #[template(path = "ckbtc/retrieve_btc_status.txt")]
    struct RetrieveBtcStatus {
        status: RetrieveBtcStatusV2,
    }
    RetrieveBtcStatus { status }.render().unwrap()
}
