use crate::{
    commands::sign::sign_ingress,
    lib::{ledger_canister_id, sign::signed_message::Ingress, AnyhowResult},
};
use candid::{CandidType, Encode};
use clap::Clap;

#[derive(CandidType)]
pub struct AccountBalanceArgs {
    pub account: String,
}

/// Signs a neuron configuration change.
#[derive(Clap)]
pub struct AccountBalanceOpts {
    /// The id of the account to query.
    account_id: String,
}

// We currently only support a subset of the functionality.
pub async fn exec(pem: &Option<String>, opts: AccountBalanceOpts) -> AnyhowResult<Vec<Ingress>> {
    let args = Encode!(&AccountBalanceArgs {
        account: opts.account_id,
    })?;
    Ok(vec![
        sign_ingress(pem, ledger_canister_id(), "account_balance_dfx", args).await?,
    ])
}
