use crate::{
    commands::send::submit_unsigned_ingress,
    lib::{governance_canister_id, AnyhowResult, ROLE_NNS_GOVERNANCE},
};
use candid::Encode;
use clap::Parser;
use ic_nns_governance::pb::v1::ListProposalInfo;

/// Queries for a list of pending proposals.
#[derive(Parser)]
pub struct ListProposalsOpts {
    /// Only displays <LIMIT> proposals.
    #[clap(long)]
    pub limit: Option<u32>,

    /// Skips confirmation and sends the message directly.
    #[clap(long, short)]
    yes: bool,

    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,
}

// We currently only support a subset of the functionality.
#[tokio::main]
pub async fn exec(opts: ListProposalsOpts, fetch_root_key: bool) -> AnyhowResult {
    let args = Encode!(&ListProposalInfo {
        limit: opts.limit.unwrap_or(100),
        before_proposal: None,
        exclude_topic: vec![2 /*TOPIC_EXCHANGE_RATE*/, 9 /*TOPIC_KYC*/],
        include_reward_status: Vec::new(),
        include_status: Vec::new(),
    })?;
    submit_unsigned_ingress(
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "list_proposals",
        args,
        opts.yes,
        opts.dry_run,
        fetch_root_key,
    )
    .await
}
