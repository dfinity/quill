use crate::{
    commands::send::submit_unsigned_ingress,
    lib::{ledger_canister_id, AnyhowResult},
};
use candid::{CandidType, Encode};
use clap::Parser;

#[derive(CandidType)]
pub struct AccountBalanceArgs {
    pub account: String,
}

/// Signs a neuron configuration change.
#[derive(Parser)]
pub struct AccountBalanceOpts {
    /// The id of the account to query.
    account_id: String,

    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,
}

// We currently only support a subset of the functionality.
pub async fn exec(opts: AccountBalanceOpts) -> AnyhowResult {
    let args = Encode!(&AccountBalanceArgs {
        account: opts.account_id,
    })?;
    submit_unsigned_ingress(
        ledger_canister_id(),
        "account_balance_dfx",
        args,
        opts.dry_run,
    )
    .await
}
