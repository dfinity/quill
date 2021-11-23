use crate::{
    commands::send::submit_unsigned_ingress,
    lib::{governance_canister_id, AnyhowResult},
};
use candid::{CandidType, Encode};
use clap::Parser;
use ic_nns_common::types::ProposalId;

#[derive(CandidType)]
pub struct ListProposalInfo {
    pub limit: u32,
    pub before_proposal: Option<ProposalId>,
    pub exclude_topic: Vec<i32>,
    pub include_reward_status: Vec<i32>,
    pub include_status: Vec<i32>,
}

#[derive(Parser)]
pub struct ListProposalsOpts {
    #[clap(long)]
    pub limit: Option<u32>,

    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,
}

// We currently only support a subset of the functionality.
pub async fn exec(opts: ListProposalsOpts) -> AnyhowResult {
    let args = Encode!(&ListProposalInfo {
        limit: opts.limit.unwrap_or(100),
        before_proposal: None,
        exclude_topic: vec![2 /*TOPIC_EXCHANGE_RATE*/, 9 /*TOPIC_KYC*/],
        include_reward_status: Vec::new(),
        include_status: Vec::new(),
    })?;
    submit_unsigned_ingress(
        governance_canister_id(),
        "list_proposals",
        args,
        opts.dry_run,
    )
    .await
}
