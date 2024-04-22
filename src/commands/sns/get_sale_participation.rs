use candid::{Encode, Principal};
use clap::Parser;
use ic_sns_swap::pb::v1::GetBuyerStateRequest;

use crate::{
    commands::{get_principal, send::submit_unsigned_ingress, SendingOpts},
    lib::{AnyhowResult, AuthInfo, ROLE_SNS_SWAP},
};

use super::SnsCanisterIds;

/// Queries for how much ICP a user has contributed to a token sale.
#[derive(Parser)]
pub struct GetSaleParticipationOpts {
    /// The principal to query. If unspecified, the caller will be used.
    #[clap(long, required_unless_present = "auth")]
    principal: Option<Principal>,

    #[clap(flatten)]
    sending_opts: SendingOpts,
}

#[tokio::main]
pub async fn exec(
    auth: &AuthInfo,
    canister_ids: &SnsCanisterIds,
    opts: GetSaleParticipationOpts,
    fetch_root_key: bool,
) -> AnyhowResult {
    let principal = if let Some(principal) = opts.principal {
        principal
    } else {
        get_principal(auth)?
    };
    let message = GetBuyerStateRequest {
        principal_id: Some(principal.into()),
    };
    submit_unsigned_ingress(
        canister_ids.swap_canister_id,
        ROLE_SNS_SWAP,
        "get_buyer_state",
        Encode!(&message)?,
        opts.sending_opts,
        fetch_root_key,
    )
    .await?;
    Ok(())
}
