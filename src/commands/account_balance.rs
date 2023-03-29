use crate::{
    commands::send::submit_unsigned_ingress,
    lib::{
        ledger_canister_id, AnyhowResult, AuthInfo, ParsedNnsAccount, ROLE_ICRC1_LEDGER,
        ROLE_NNS_LEDGER,
    },
};
use candid::{CandidType, Encode};
use clap::Parser;

use super::get_ids;

#[derive(CandidType)]
pub struct AccountBalanceArgs {
    pub account: String,
}

/// Queries a ledger account balance.
#[derive(Parser)]
pub struct AccountBalanceOpts {
    /// The id of the account to query. Optional if a key is used.
    #[clap(required_unless_present = "auth")]
    account_id: Option<ParsedNnsAccount>,

    /// Skips confirmation and sends the message directly.
    #[clap(long, short)]
    yes: bool,

    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,
}

// We currently only support a subset of the functionality.
#[tokio::main]
pub async fn exec(auth: &AuthInfo, opts: AccountBalanceOpts, fetch_root_key: bool) -> AnyhowResult {
    let account_id = if let Some(id) = opts.account_id {
        id
    } else {
        let (_, id) = get_ids(auth)?;
        ParsedNnsAccount::Original(id)
    };
    match account_id {
        ParsedNnsAccount::Original(id) => {
            let args = Encode!(&AccountBalanceArgs {
                account: id.to_hex()
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
        ParsedNnsAccount::Icrc1(id) => {
            let args = Encode!(&id)?;
            submit_unsigned_ingress(
                ledger_canister_id(),
                ROLE_ICRC1_LEDGER,
                "icrc1_balance_of",
                args,
                opts.yes,
                opts.dry_run,
                fetch_root_key,
            )
            .await
        }
    }
}
