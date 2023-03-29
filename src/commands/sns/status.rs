use candid::Encode;
use clap::Parser;
use ic_sns_root::GetSnsCanistersSummaryRequest;

use crate::{
    commands::send::submit_unsigned_ingress,
    lib::{AnyhowResult, ROLE_SNS_ROOT},
};

use super::SnsCanisterIds;

/// Fetches the status of the canisters in the SNS. This includes their controller, running status, canister settings,
/// cycle balance, memory size, daily cycle burn rate, and module hash, along with their principals.
#[derive(Parser)]
pub struct StatusOpts {
    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,

    /// Skips confirmation and sends the message immediately.
    #[clap(long, short)]
    yes: bool,
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
        opts.yes,
        opts.dry_run,
        fetch_root_key,
    )
    .await?;
    Ok(())
}
