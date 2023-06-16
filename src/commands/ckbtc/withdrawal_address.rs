use candid::Principal;
use clap::Parser;

use crate::{
    commands::get_principal,
    lib::{AnyhowResult, AuthInfo, ParsedAccount},
};

use super::ckbtc_withdrawal_address;

/// Displays the address that you or a specified user can deposit ckBTC at to retrieve BTC.
///
/// The `--of` parameter is required if a signing key is not provided.
///
/// If you have made a transfer to this address, use the `--already-transferred` flag with
/// `quill ckbtc retrieve-btc` to register it.
#[derive(Parser)]
pub struct GetWithdrawalAddressOpts {
    /// The principal to get the withdrawal address for. Optional if a key is used.
    #[clap(long, required_unless_present = "auth")]
    of: Option<Principal>,

    /// Uses ckTESTBTC instead of ckBTC.
    #[clap(long)]
    testnet: bool,
}

pub fn exec(auth: &AuthInfo, opts: GetWithdrawalAddressOpts) -> AnyhowResult {
    let principal = if let Some(principal) = opts.of {
        principal
    } else {
        get_principal(auth)?
    };
    let address = ParsedAccount(ckbtc_withdrawal_address(&principal, opts.testnet));
    println!("{address}");
    eprintln!("Use the --already-transferred flag with `quill ckbtc retrieve-btc` to register any transfers.");
    Ok(())
}
