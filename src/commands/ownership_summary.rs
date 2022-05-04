use crate::lib::{AnyhowResult, TargetCanister};
use clap::Parser;
use candid::Encode;
use ic_base_types::PrincipalId;
//use ic_ic00_types::{CanisterIdRecord, IC_00};
use crate::SnsCanisterIds;
use crate::commands::send::sign_send_and_check_status;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct OwnershipSummaryOps {

    /// Canister id of the dapp.
    #[clap(long)]
    canister_id: String,

    /// Will display the query, but not send it
    #[clap(long)]
    dry_run: bool,
}



pub async fn exec(private_key_pem: &str, sns_canister_ids: &SnsCanisterIds, opts: OwnershipSummaryOps) -> AnyhowResult {
    let canister_id = PrincipalId::from_str(&opts.canister_id)?;
    let root_canister_id = PrincipalId::from(sns_canister_ids.root_canister_id).0;

    let args = Encode!(&vec![canister_id])?;

    sign_send_and_check_status(
        private_key_pem,
        "get_sns_canisters_summary",
        args.clone(),
        opts.dry_run,
        TargetCanister::Root(root_canister_id),
    ).await?;

    Ok(())
}