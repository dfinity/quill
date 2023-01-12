use candid::Encode;
use clap::Parser;
use ic_ckbtc_minter::updates::update_balance::UpdateBalanceArgs;

use crate::lib::{
    ckbtc_minter_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedAccount, ParsedSubaccount,
};

/// Signs a message to mint ckBTC from previously deposited BTC.
#[derive(Parser)]
pub struct UpdateBalanceOpts {
    /// The account to mint ckBTC to.
    #[clap(long)]
    sender: Option<ParsedAccount>,
    /// The subaccount to mint ckBTC to.
    #[clap(long)]
    subaccount: Option<ParsedSubaccount>,
}

pub fn exec(auth: &AuthInfo, opts: UpdateBalanceOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let (owner, mut subaccount) = opts
        .sender
        .map_or((None, None), |x| (Some(x.0.owner.into()), x.0.subaccount));
    if let Some(subacct) = opts.subaccount {
        subaccount = Some(subacct.0 .0);
    }
    let args = UpdateBalanceArgs { owner, subaccount };
    let message = sign_ingress_with_request_status_query(
        auth,
        ckbtc_minter_canister_id(),
        "update_balance",
        Encode!(&args)?,
    )?;
    Ok(vec![message])
}
