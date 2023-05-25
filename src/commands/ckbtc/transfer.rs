use candid::{Encode, Nat};
use clap::Parser;
use icrc_ledger_types::icrc1::transfer::{Memo, TransferArg};

use crate::{
    commands::get_account,
    lib::{
        ckbtc_canister_id, now_nanos,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, AuthInfo, ParsedAccount, ParsedSubaccount, ROLE_ICRC1_LEDGER,
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
    let to = get_account(None, Some(opts.to), opts.to_subaccount)?;
    let amount = opts.satoshis.unwrap_or_else(|| opts.amount.unwrap().0);
    let args = TransferArg {
        amount,
        created_at_time: Some(now_nanos()),
        fee: opts.fee,
        from_subaccount: opts.from_subaccount.map(|x| x.0 .0),
        to,
        memo: opts.memo.map(Memo::from),
    };
    let message = sign_ingress_with_request_status_query(
        auth,
        ckbtc_canister_id(opts.testnet),
        ROLE_ICRC1_LEDGER,
        "icrc1_transfer",
        Encode!(&args)?,
    )?;
    Ok(vec![message])
}
