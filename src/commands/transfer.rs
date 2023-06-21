use crate::commands::send::{Memo, SendArgs, TimeStamp};
use crate::lib::{
    ledger_canister_id, now_nanos, parse_tokens,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNnsAccount, ParsedSubaccount, ROLE_ICRC1_LEDGER, ROLE_NNS_LEDGER,
};
use anyhow::ensure;
use candid::Encode;
use clap::Parser;
use icp_ledger::{Tokens, DEFAULT_TRANSFER_FEE};
use icrc_ledger_types::icrc1::transfer::TransferArg;

/// Signs an ICP transfer transaction.
#[derive(Parser)]
pub struct TransferOpts {
    /// Destination account.
    pub to: ParsedNnsAccount,

    /// Destination subaccount.
    #[clap(long)]
    pub to_subaccount: Option<ParsedSubaccount>,

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
            ensure!(
                opts.to_subaccount.is_none(),
                "Cannot specify both --subaccount and a legacy account ID"
            );
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
        ParsedNnsAccount::Icrc1(mut to) => {
            if let Some(sub) = opts.to_subaccount {
                to.subaccount = Some(sub.0 .0);
            }
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
