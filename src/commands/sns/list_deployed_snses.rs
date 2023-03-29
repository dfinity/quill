use candid::Encode;
use clap::Parser;
use ic_sns_wasm::pb::v1::ListDeployedSnsesRequest;

use crate::{
    commands::send::submit_unsigned_ingress,
    lib::{sns_wasm_canister_id, AnyhowResult, ROLE_SNS_WASM},
};

/// Lists all SNSes that have been deployed by the NNS.
#[derive(Parser)]
pub struct ListDeployedSnsesOpts {
    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,

    /// Skips confirmation and sends the message immediately.
    #[clap(long, short)]
    yes: bool,
}

#[tokio::main]
pub async fn exec(opts: ListDeployedSnsesOpts, fetch_root_key: bool) -> AnyhowResult {
    let arg = Encode!(&ListDeployedSnsesRequest {})?;
    submit_unsigned_ingress(
        sns_wasm_canister_id(),
        ROLE_SNS_WASM,
        "list_deployed_snses",
        arg,
        opts.yes,
        opts.dry_run,
        fetch_root_key,
    )
    .await?;
    Ok(())
}
