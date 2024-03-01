use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_sns_swap::pb::v1::RefreshBuyerTokensRequest;
use icp_ledger::{AccountIdentifier, Memo, SendArgs, Subaccount, TimeStamp, Tokens};

use crate::lib::ParsedSubaccount;
use crate::lib::{
    ledger_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_NNS_LEDGER, ROLE_SNS_SWAP,
};

use super::SnsCanisterIds;

/// Signs messages to pay for an open sale ticket that you can create using `quill sns new-sale-ticket`.
/// This operation consists of two messages:
/// First, transfer ICP to the sale canister on the NNS ledger, under the subaccount for your principal.
/// Second, the sale canister is notified that the transfer has been made.
#[derive(Parser)]
pub struct PayOpts {
    /// The amount of ICP to transfer. This should be the same as the "amount_icp_e8s" of the sale ticket.
    /// Please note that a 10000 e8s transaction fee will be charged on top of this amount.
    #[clap(long, required_unless_present("notify-only"))]
    amount_icp_e8s: Option<u64>,

    /// Pay from this subaccount. For example: e000d80101. This should be aligned with the "account" of the sale ticket.
    #[clap(long)]
    subaccount: Option<ParsedSubaccount>,

    /// The creation_time of the sale ticket. This should be the same as the "creation_time" of the sale ticket.
    #[clap(long, required_unless_present("notify-only"))]
    ticket_creation_time: Option<u64>,

    /// The tocket_id of the sale ticket. This should be the same as the "ticket_id" of the sale ticket.
    #[clap(long, required_unless_present("notify-only"))]
    ticket_id: Option<u64>,

    /// If this flag is specified, then no transfer will be made, and only the notification message will be generated.
    /// This is useful if there was an error previously submitting the notification which you have since rectified, or if you have made the transfer with another tool.
    #[clap(long)]
    notify_only: bool,

    /// If a particular SNS requires confirmation text to participate in a sale, enter it using this flag.
    #[clap(long)]
    confirmation_text: Option<String>,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: PayOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let controller = crate::lib::get_principal(auth)?;
    let mut messages = vec![];
    if !opts.notify_only {
        let subaccount = Subaccount::from(&PrincipalId(controller));
        let account_id =
            AccountIdentifier::new(sns_canister_ids.swap_canister_id.into(), Some(subaccount));
        let request = SendArgs {
            amount: Tokens::from_e8s(opts.amount_icp_e8s.unwrap()),
            created_at_time: Some(TimeStamp::from_nanos_since_unix_epoch(
                opts.ticket_creation_time.unwrap(),
            )),
            from_subaccount: opts.subaccount.map(|x| x.0),
            fee: Tokens::from_e8s(10_000),
            memo: Memo(opts.ticket_id.unwrap()),
            to: account_id,
        };
        messages.push(sign_ingress_with_request_status_query(
            auth,
            ledger_canister_id(),
            ROLE_NNS_LEDGER,
            "send_dfx",
            Encode!(&request)?,
        )?);
    }
    let refresh = RefreshBuyerTokensRequest {
        buyer: controller.to_text(),
        confirmation_text: opts.confirmation_text,
    };
    messages.push(sign_ingress_with_request_status_query(
        auth,
        sns_canister_ids.swap_canister_id,
        ROLE_SNS_SWAP,
        "refresh_buyer_tokens",
        Encode!(&refresh)?,
    )?);
    Ok(messages)
}
