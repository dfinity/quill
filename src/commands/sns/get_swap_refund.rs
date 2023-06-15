use candid::{Encode, Principal};
use clap::Parser;
use ic_sns_swap::pb::v1::ErrorRefundIcpRequest;

use crate::{
    commands::get_principal,
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, AuthInfo, ROLE_SNS_SWAP,
    },
};

use super::SnsCanisterIds;

/// Signs a message to request a refund from the SNS swap canister.
/// If the swap was aborted or failed, or some of your contributed ICP never made it into a neuron,
/// this command can retrieve your unused ICP, minus transaction fees.
#[derive(Parser)]
pub struct GetSwapRefundOpts {
    /// The principal that made the ICP contribution and should be refunded. The ICP will be
    /// refunded to the main account of this Principal irrespective of the caller.
    #[clap(long)]
    principal: Option<Principal>,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: GetSwapRefundOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let principal = if let Some(principal) = opts.principal {
        principal
    } else {
        get_principal(auth)?
    };
    let message = ErrorRefundIcpRequest {
        source_principal_id: Some(principal.into()),
    };
    let req = sign_ingress_with_request_status_query(
        auth,
        sns_canister_ids.swap_canister_id,
        ROLE_SNS_SWAP,
        "error_refund_icp",
        Encode!(&message)?,
    )?;
    Ok(vec![req])
}
