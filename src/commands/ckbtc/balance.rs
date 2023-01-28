use candid::Encode;
use clap::Parser;
use ic_icrc1::Account;

use crate::{
    commands::{get_ids, send::submit_unsigned_ingress},
    lib::{ckbtc_canister_id, AnyhowResult, AuthInfo, ParsedAccount, ParsedSubaccount},
};

/// Sends a message to check the provided user's ckBTC balance.
///
/// The `--of` parameter is required if a signing key is not provided.
#[derive(Parser)]
pub struct BalanceOpts {
    /// The account to check.
    #[clap(long, required_unless_present = "auth")]
    of: Option<ParsedAccount>,

    /// The subaccount of the account to check.
    #[clap(long, conflicts_with = "of")]
    of_subaccount: Option<ParsedSubaccount>,

    /// Skips confirmation and sends the message immediately.
    #[clap(long, short)]
    yes: bool,

    /// Will display the signed message, but not send it.
    #[clap(long)]
    dry_run: bool,

    /// Uses ckTESTBTC instead of ckBTC.
    #[clap(long)]
    testnet: bool,
}

#[tokio::main]
pub async fn exec(auth: &AuthInfo, opts: BalanceOpts, fetch_root_key: bool) -> AnyhowResult {
    let mut account = if let Some(acct) = opts.of {
        acct.0
    } else {
        let (principal, _) = get_ids(auth)?;
        Account {
            owner: principal.into(),
            subaccount: None,
        }
    };
    if let Some(subaccount) = opts.of_subaccount {
        account.subaccount = Some(subaccount.0 .0);
    }
    submit_unsigned_ingress(
        ckbtc_canister_id(opts.testnet),
        "icrc1_balance_of",
        Encode!(&account)?,
        opts.yes,
        opts.dry_run,
        fetch_root_key,
    )
    .await?;
    Ok(())
}
