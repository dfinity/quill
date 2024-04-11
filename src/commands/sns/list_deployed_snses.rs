use candid::Encode;
use clap::Parser;
use ic_sns_wasm::pb::v1::ListDeployedSnsesRequest;

use crate::{
    commands::{send::submit_unsigned_ingress, QueryOpts},
    lib::{sns_wasm_canister_id, AnyhowResult, ROLE_SNS_WASM},
};

/// Lists all SNSes that have been deployed by the NNS.
#[derive(Parser)]
pub struct ListDeployedSnsesOpts {
    #[clap(flatten)]
    query_opts: QueryOpts,
}

#[tokio::main]
pub async fn exec(opts: ListDeployedSnsesOpts, fetch_root_key: bool) -> AnyhowResult {
    let arg = Encode!(&ListDeployedSnsesRequest {})?;
    submit_unsigned_ingress(
        sns_wasm_canister_id(),
        ROLE_SNS_WASM,
        "list_deployed_snses",
        arg,
        opts.query_opts,
        fetch_root_key,
    )
    .await?;
    Ok(())
}
