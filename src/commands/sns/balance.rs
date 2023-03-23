use crate::{
    commands::{get_ids, send::submit_unsigned_ingress},
    lib::{AuthInfo, ParsedAccount, ParsedSubaccount, ROLE_ICRC1_LEDGER},
    AnyhowResult,
};
use candid::Encode;
use clap::Parser;
use icrc_ledger_types::Account;

use super::SnsCanisterIds;

/// Sends a ledger account-balance query call.
///
/// The `--of` parameter is required if a signing key is not provided.
#[derive(Parser)]
pub struct BalanceOpts {
    /// The account to query. Optional if a key is used.
    #[clap(long, required_unless_present = "auth")]
    of: Option<ParsedAccount>,

    /// The subaccount of the account to query.
    #[clap(long)]
    subaccount: Option<ParsedSubaccount>,

    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,

    /// Skips confirmation and sends the message immediately.
    #[clap(long, short)]
    yes: bool,
}

#[tokio::main]
pub async fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: BalanceOpts,
    fetch_root_key: bool,
) -> AnyhowResult {
    let ledger_canister_id = sns_canister_ids.ledger_canister_id;
    let mut account = if let Some(acct) = opts.of {
        acct.0
    } else {
        let (principal, _) = get_ids(auth)?;
        Account {
            owner: principal,
            subaccount: None,
        }
    };
    if let Some(subaccount) = opts.subaccount {
        account.subaccount = Some(subaccount.0 .0);
    }

    submit_unsigned_ingress(
        ledger_canister_id,
        ROLE_ICRC1_LEDGER,
        "icrc1_balance_of",
        Encode!(&account)?,
        opts.yes,
        opts.dry_run,
        fetch_root_key,
    )
    .await?;

    Ok(())
}
