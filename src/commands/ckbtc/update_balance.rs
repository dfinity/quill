use candid::Encode;
use clap::Parser;
use ic_ckbtc_minter::updates::update_balance::UpdateBalanceArgs;

use crate::{
    commands::get_account,
    lib::{
        ckbtc_minter_canister_id,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, AuthInfo, ParsedAccount, ParsedSubaccount, ROLE_CKBTC_MINTER,
    },
};

/// Signs a message to mint ckBTC from previously deposited BTC.
#[derive(Parser)]
pub struct UpdateBalanceOpts {
    /// The account to mint ckBTC to.
    #[clap(long, required_unless_present = "auth")]
    sender: Option<ParsedAccount>,
    /// The subaccount to mint ckBTC to.
    #[clap(long)]
    subaccount: Option<ParsedSubaccount>,
    /// Uses ckTESTBTC instead of ckBTC.
    #[clap(long)]
    testnet: bool,
}

pub fn exec(auth: &AuthInfo, opts: UpdateBalanceOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let account = get_account(Some(auth), opts.sender, opts.subaccount)?;
    let args = UpdateBalanceArgs {
        owner: Some(account.owner),
        subaccount: account.subaccount,
    };
    let message = sign_ingress_with_request_status_query(
        auth,
        ckbtc_minter_canister_id(opts.testnet),
        ROLE_CKBTC_MINTER,
        "update_balance",
        Encode!(&args)?,
    )?;
    Ok(vec![message])
}
