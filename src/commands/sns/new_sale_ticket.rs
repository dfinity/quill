use candid::Encode;
use clap::Parser;
use ic_sns_swap::pb::v1::NewSaleTicketRequest;

use crate::lib::{
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedSubaccount, ROLE_SNS_SWAP,
};

use super::SnsCanisterIds;

/// Attempt to reate a new sale ticket. If there is already an open ticket, it will return the details of the existing ticket.
#[derive(Parser)]
pub struct NewSaleTicketOpts {
    /// The amount of ICP tokens to participate in this sns sale. You will need to make the transfer later.
    #[clap(long)]
    amount_icp_e8s: u64,

    /// The subaccount you will use to pay for this ticket. For example: e000d80101.
    #[clap(long)]
    subaccount: Option<ParsedSubaccount>,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: NewSaleTicketOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let request = NewSaleTicketRequest {
        amount_icp_e8s: opts.amount_icp_e8s,
        subaccount: opts.subaccount.map(|x| x.0 .0.to_vec()),
    };
    let message = sign_ingress_with_request_status_query(
        auth,
        sns_canister_ids.swap_canister_id,
        ROLE_SNS_SWAP,
        "new_sale_ticket",
        Encode!(&request)?,
    )?;
    Ok(vec![message])
}
