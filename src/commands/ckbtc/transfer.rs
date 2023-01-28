use candid::{Encode, Nat};
use clap::Parser;
use ic_icrc1::{endpoints::Transfer, Account, Memo};

use crate::{
    commands::get_ids,
    lib::{
        ckbtc_canister_id,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, AuthInfo, ParsedAccount, ParsedSubaccount,
    },
};

use super::Btc;

/// Signs a message to transfer ckBTC from one account to another.
#[derive(Parser)]
pub struct TransferOpts {
    /// The account to transfer ckBTC to.
    to: ParsedAccount,
    /// The subaccount to transfer ckBTC to.
    #[clap(long)]
    to_subaccount: Option<ParsedSubaccount>,
    /// The subaccount to transfer ckBTC from.
    #[clap(long)]
    from_subaccount: Option<ParsedSubaccount>,
    /// The amount, in decimal ckBTC, to transfer.
    #[clap(long)]
    amount: Option<Btc>,
    /// The amount, in integer satoshis, to transfer.
    #[clap(long, conflicts_with = "amount", required_unless_present = "amount")]
    satoshis: Option<Nat>,
    /// An integer memo for this transaction.
    #[clap(long)]
    memo: Option<u64>,
    /// The expected fee for this transaction.
    #[clap(long)]
    fee: Option<Nat>,
    /// Uses ckTESTBTC instead of ckBTC.
    #[clap(long)]
    testnet: bool,
}

pub fn exec(auth: &AuthInfo, opts: TransferOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let (principal, _) = get_ids(auth)?;
    let from = Account {
        owner: principal.into(),
        subaccount: opts.from_subaccount.map(|x| x.0 .0),
    };
    let mut to = opts.to.0;
    if let Some(subaccount) = opts.to_subaccount {
        to.subaccount = Some(subaccount.0 .0);
    }
    let amount = opts.satoshis.unwrap_or_else(|| opts.amount.unwrap().0);
    let args = Transfer {
        amount,
        created_at_time: None,
        fee: opts.fee,
        from,
        to,
        memo: opts.memo.map(Memo::from),
    };
    let message = sign_ingress_with_request_status_query(
        auth,
        ckbtc_canister_id(opts.testnet),
        "icrc1_transfer",
        Encode!(&args)?,
    )?;
    Ok(vec![message])
}
