use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_sns_swap::pb::v1::RefreshBuyerTokensRequest;
use icp_ledger::{AccountIdentifier, Memo, SendArgs, Subaccount, Tokens};

use crate::commands::transfer::parse_tokens;
use crate::lib::{
    ledger_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_NNS_LEDGER, ROLE_SNS_SWAP,
};

use super::SnsCanisterIds;

/// Signs messages needed to participate in the initial token swap. This operation consists of two messages:
/// First, `amount` ICP is transferred to the swap canister on the NNS ledger, under the subaccount for your principal.
/// Second, the swap canister is notified that the transfer has been made.
/// Once the swap has been finalized, if it was successful, you will receive your neurons automatically.
#[derive(Parser)]
pub struct SwapOpts {
    /// The amount of ICP to transfer. Your neuron's share of the governance tokens at sale finalization will be proportional to your share of the contributed ICP.
    #[clap(long, requires("memo"), required_unless_present("notify-only"), value_parser = parse_tokens)]
    amount: Option<Tokens>,

    /// An arbitrary number used to identify the NNS block this transfer was made in.
    #[clap(long)]
    memo: Option<u64>,

    /// If this flag is specified, then no transfer will be made, and only the notification message will be generated.
    /// This is useful if there was an error previously submitting the notification which you have since rectified, or if you have made the transfer with another tool.
    #[clap(long)]
    notify_only: bool,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: SwapOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let (controller, _) = crate::commands::public::get_ids(auth)?;
    let mut messages = vec![];
    if !opts.notify_only {
        let subaccount = Subaccount::from(&PrincipalId(controller));
        let account_id =
            AccountIdentifier::new(sns_canister_ids.swap_canister_id.into(), Some(subaccount));
        let request = SendArgs {
            amount: opts.amount.unwrap(),
            created_at_time: None,
            from_subaccount: None,
            fee: Tokens::from_e8s(10_000),
            memo: Memo(opts.memo.unwrap()),
            to: account_id,
        };
        messages.push(sign_ingress_with_request_status_query(
            auth,
            ledger_canister_id(),
            ROLE_NNS_LEDGER,
            "send_dfx",
            Encode!(&request)?,
        )?)
    }
    let refresh = RefreshBuyerTokensRequest {
        buyer: controller.to_text(),
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
