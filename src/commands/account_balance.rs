use crate::{
    commands::send::submit_unsigned_ingress,
    lib::{ledger_canister_id, AnyhowResult, ROLE_NNS_LEDGER},
};
use candid::{CandidType, Encode};
use clap::Parser;

#[derive(CandidType)]
pub struct AccountBalanceArgs {
    pub account: String,
}

/// Queries a ledger account balance.
#[derive(Parser)]
pub struct AccountBalanceOpts {
    /// The id of the account to query.
    account_id: String,

    /// Skips confirmation and sends the message directly.
    #[clap(long)]
    yes: bool,

    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,
}

// We currently only support a subset of the functionality.
#[tokio::main]
pub async fn exec(opts: AccountBalanceOpts, fetch_root_key: bool) -> AnyhowResult {
    let args = Encode!(&AccountBalanceArgs {
        account: opts.account_id,
    })?;
    submit_unsigned_ingress(
        ledger_canister_id(),
        ROLE_NNS_LEDGER,
        "account_balance_dfx",
        args,
        opts.yes,
        opts.dry_run,
        fetch_root_key,
    )
    .await
}
