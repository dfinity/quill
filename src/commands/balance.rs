use crate::{
    commands::send::submit_unsigned_ingress,
    lib::{
        get_account_id, ledger_canister_id, AnyhowResult, AuthInfo, ParsedNnsAccount,
        ParsedSubaccount, ROLE_ICRC1_LEDGER, ROLE_NNS_LEDGER,
    },
};
use anyhow::ensure;
use candid::{CandidType, Encode};
use clap::Parser;

use super::get_principal;

#[derive(CandidType)]
pub struct AccountBalanceArgs {
    pub account: String,
}

/// Queries a ledger account balance.
#[derive(Parser)]
#[clap(alias = "account-balance")]
pub struct BalanceOpts {
    #[clap(hidden = true)]
    account_id: Option<ParsedNnsAccount>,

    /// The id of the account to query. Optional if a key is used.
    #[clap(long, required_unless_present_any = ["auth", "account-id"])]
    of: Option<ParsedNnsAccount>,

    /// The subaccount of the account to query.
    #[clap(long)]
    subaccount: Option<ParsedSubaccount>,

    /// Skips confirmation and sends the message directly.
    #[clap(long, short)]
    yes: bool,

    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,
}

#[tokio::main]
pub async fn exec(auth: &AuthInfo, opts: BalanceOpts, fetch_root_key: bool) -> AnyhowResult {
    let account_id = if let Some(id) = opts.of.or(opts.account_id) {
        id
    } else {
        let id = get_account_id(get_principal(auth)?, None)?;
        ParsedNnsAccount::Original(id)
    };
    match account_id {
        ParsedNnsAccount::Original(id) => {
            ensure!(
                opts.subaccount.is_none(),
                "Cannot specify both --subaccount and a legacy account ID"
            );
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
        ParsedNnsAccount::Icrc1(mut id) => {
            if let Some(sub) = opts.subaccount {
                id.subaccount = Some(sub.0 .0);
            }
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
