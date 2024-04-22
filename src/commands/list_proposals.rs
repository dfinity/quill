use crate::{
    commands::{send::submit_unsigned_ingress, SendingOpts},
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

    #[clap(flatten)]
    pub sending_opts: SendingOpts,
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
        include_all_manage_neuron_proposals: Some(false),
        omit_large_fields: Some(false),
    })?;
    submit_unsigned_ingress(
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "list_proposals",
        args,
        opts.sending_opts,
        fetch_root_key,
    )
    .await
}
