use candid::Encode;
use clap::Parser;
use ic_sns_root::GetSnsCanistersSummaryRequest;

use crate::{
    commands::{send::submit_unsigned_ingress, SendingOpts},
    lib::{AnyhowResult, ROLE_SNS_ROOT},
};

use super::SnsCanisterIds;

/// Fetches the status of the canisters in the SNS. This includes their controller, running status, canister settings,
/// cycle balance, memory size, daily cycle burn rate, and module hash, along with their principals.
#[derive(Parser)]
pub struct StatusOpts {
    #[clap(flatten)]
    sending_opts: SendingOpts,
}

#[tokio::main]
pub async fn exec(ids: &SnsCanisterIds, opts: StatusOpts, fetch_root_key: bool) -> AnyhowResult {
    let root_canister_id = ids.root_canister_id;
    let arg = Encode!(&GetSnsCanistersSummaryRequest {
        update_canister_list: None,
    })?;
    submit_unsigned_ingress(
        root_canister_id,
        ROLE_SNS_ROOT,
        "get_sns_canisters_summary",
        arg,
        opts.sending_opts,
        fetch_root_key,
    )
    .await?;
    Ok(())
}
