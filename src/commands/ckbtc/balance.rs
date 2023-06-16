use candid::Encode;
use clap::Parser;

use crate::{
    commands::{get_account, send::submit_unsigned_ingress},
    lib::{
        ckbtc_canister_id, AnyhowResult, AuthInfo, ParsedAccount, ParsedSubaccount,
        ROLE_ICRC1_LEDGER,
    },
};

/// Sends a message to check the provided user's ckBTC balance.
///
/// The `--of` parameter is required if a signing key is not provided.
#[derive(Parser)]
pub struct BalanceOpts {
    /// The account to check. Optional if a key is used.
    #[clap(long, required_unless_present = "auth")]
    of: Option<ParsedAccount>,

    /// The subaccount of the account to check.
    #[clap(long, alias = "of-subaccount")]
    subaccount: Option<ParsedSubaccount>,

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
    let account = get_account(Some(auth), opts.of, opts.subaccount)?;
    submit_unsigned_ingress(
        ckbtc_canister_id(opts.testnet),
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
