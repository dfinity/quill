use crate::commands::send::{Memo, SendArgs, TimeStamp};
use crate::lib::{
    ledger_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo,
};
use crate::lib::{
    now_nanos, ParsedNnsAccount, ParsedSubaccount, ROLE_ICRC1_LEDGER, ROLE_NNS_LEDGER,
};
use anyhow::{anyhow, bail, Context};
use candid::Encode;
use clap::Parser;
use icp_ledger::{Tokens, DEFAULT_TRANSFER_FEE};
use icrc_ledger_types::icrc1::transfer::TransferArg;

/// Signs an ICP transfer transaction.
#[derive(Parser)]
pub struct TransferOpts {
    /// Destination account.
    pub to: ParsedNnsAccount,

    /// Amount of ICPs to transfer (with up to 8 decimal digits after comma).
    #[clap(long, value_parser = parse_tokens)]
    pub amount: Tokens,

    /// Reference number, default is 0.
    #[clap(long)]
    pub memo: Option<u64>,

    /// Transaction fee, default is 0.0001 ICP.
    #[clap(long, value_parser = parse_tokens)]
    pub fee: Option<Tokens>,

    /// The subaccount to transfer from.
    #[clap(long)]
    pub from_subaccount: Option<ParsedSubaccount>,
}

pub fn exec(auth: &AuthInfo, opts: TransferOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let amount = opts.amount;
    let fee = opts.fee.unwrap_or(DEFAULT_TRANSFER_FEE);
    let memo = Memo(opts.memo.unwrap_or(0));
    let to = opts.to;
    match to {
        ParsedNnsAccount::Original(to) => {
            let args = Encode!(&SendArgs {
                memo,
                amount,
                fee,
                from_subaccount: opts.from_subaccount.map(|x| x.0),
                to: to.to_hex(),
                created_at_time: Some(TimeStamp {
                    timestamp_nanos: now_nanos()
                }),
            })?;

            let msg = sign_ingress_with_request_status_query(
                auth,
                ledger_canister_id(),
                ROLE_NNS_LEDGER,
                "send_dfx",
                args,
            )?;
            Ok(vec![msg])
        }
        ParsedNnsAccount::Icrc1(to) => {
            let args = Encode!(&TransferArg {
                memo: Some(memo.0.into()),
                amount: amount.get_e8s().into(),
                fee: Some(fee.get_e8s().into()),
                from_subaccount: opts.from_subaccount.map(|x| x.0 .0),
                to,
                created_at_time: Some(now_nanos()),
            })?;
            let msg = sign_ingress_with_request_status_query(
                auth,
                ledger_canister_id(),
                ROLE_ICRC1_LEDGER,
                "icrc1_transfer",
                args,
            )?;
            Ok(vec![msg])
        }
    }
}

fn new_tokens(tokens: u64, e8s: u64) -> AnyhowResult<Tokens> {
    Tokens::new(tokens, e8s)
        .map_err(|err| anyhow!(err))
        .context("Cannot create new tokens structure")
}

pub fn parse_tokens(amount: &str) -> AnyhowResult<Tokens> {
    let parse = |s: &str| {
        s.parse::<u64>()
            .context("Failed to parse tokens as unsigned integer")
    };
    match *amount.split('.').collect::<Vec<_>>().as_slice() {
        [tokens] => new_tokens(parse(tokens)?, 0),
        [tokens, e8s] => {
            let mut e8s = e8s.to_string();
            while e8s.len() < 8 {
                e8s.push('0');
            }
            let e8s = &e8s[..8];
            new_tokens(parse(tokens)?, parse(e8s)?)
        }
        _ => bail!("Cannot parse amount {}", amount),
    }
}
