use std::convert::TryInto;

use anyhow::Context;
use candid::{Encode, Nat};
use clap::Parser;
use ic_ckbtc_minter::updates::retrieve_btc::RetrieveBtcArgs;
use icrc_ledger_types::icrc1::transfer::{Memo, TransferArg};

use crate::{
    commands::get_principal,
    lib::{
        ckbtc_canister_id, ckbtc_minter_canister_id, now_nanos,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, AuthInfo, ParsedSubaccount, ROLE_CKBTC_MINTER, ROLE_ICRC1_LEDGER,
    },
};

use super::{ckbtc_withdrawal_address, Btc};

/// Signs messages to retrieve BTC in exchange for ckBTC.
///
/// This command generates two messages by default; a transfer of ckBTC to the minting canister, and a request for BTC.
/// However, if you have already made this transfer (the address can be viewed with `quill ckbtc withdrawal-address`),
/// you can use the `--already-transferred` flag to skip the first message.
///
/// Bitcoin transactions take a while, so the response to the second message will not be a success state, but rather a
/// block index. Use the `quill ckbtc retrieve-btc-status` command to check the status of this transfer.
#[derive(Parser)]
pub struct RetrieveBtcOpts {
    /// The Bitcoin address to send the BTC to. Note that Quill does not validate this address.
    to: String,
    /// The quantity, in decimal BTC, to convert.
    #[clap(long)]
    amount: Option<Btc>,
    /// The quantity, in integer satoshis, to convert.
    #[clap(long, conflicts_with = "amount", required_unless_present = "amount")]
    satoshis: Option<Nat>,
    /// The subaccount to transfer the ckBTC from.
    #[clap(long)]
    from_subaccount: Option<ParsedSubaccount>,
    /// An integer memo for the ckBTC transfer.
    #[clap(long)]
    memo: Option<u64>,
    /// The expected fee for the ckBTC transfer.
    #[clap(long)]
    fee: Option<Nat>,
    /// Skips signing the transfer of ckBTC, signing only the request for BTC.
    #[clap(
        long,
        conflicts_with = "memo",
        conflicts_with = "from-subaccount",
        conflicts_with = "fee"
    )]
    already_transferred: bool,
    /// Uses ckTESTBTC instead of ckBTC.
    #[clap(long)]
    testnet: bool,
}

pub fn exec(auth: &AuthInfo, opts: RetrieveBtcOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let principal = get_principal(auth)?;
    let mut messages = vec![];
    let amount = opts.satoshis.unwrap_or_else(|| opts.amount.unwrap().0);
    if !opts.already_transferred {
        let transfer_args = TransferArg {
            amount: amount.clone(),
            created_at_time: Some(now_nanos()),
            fee: opts.fee,
            from_subaccount: opts.from_subaccount.map(|x| x.0 .0),
            memo: opts.memo.map(Memo::from),
            to: ckbtc_withdrawal_address(&principal, opts.testnet),
        };
        messages.push(sign_ingress_with_request_status_query(
            auth,
            ckbtc_canister_id(opts.testnet),
            ROLE_ICRC1_LEDGER,
            "icrc1_transfer",
            Encode!(&transfer_args)?,
        )?);
    }
    let retrieve_args = RetrieveBtcArgs {
        address: opts.to,
        amount: amount.0.try_into().context("Amount too large (max 184B)")?,
    };
    messages.push(sign_ingress_with_request_status_query(
        auth,
        ckbtc_minter_canister_id(opts.testnet),
        ROLE_CKBTC_MINTER,
        "retrieve_btc",
        Encode!(&retrieve_args)?,
    )?);
    Ok(messages)
}
