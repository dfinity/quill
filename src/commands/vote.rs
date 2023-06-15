use anyhow::ensure;
use candid::Encode;
use clap::{ArgGroup, Parser};
use ic_nns_common::pb::v1::ProposalId;
use ic_nns_governance::pb::v1::{
    manage_neuron::{Command, RegisterVote},
    ManageNeuron, Vote,
};

use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ROLE_NNS_GOVERNANCE,
};

/// Signs a neuron management message to vote on a proposal.
#[derive(Parser)]
#[clap(group(ArgGroup::new("yn").required(true)))]
pub struct VoteOpts {
    /// The ID of the neuron to vote with.
    neuron_id: ParsedNeuron,

    /// Vote to approve the proposal.
    #[clap(long, group = "yn")]
    approve: bool,

    /// Vote to reject the proposal.
    #[clap(long, group = "yn")]
    reject: bool,

    /// The ID of the proposal to vote on.
    #[clap(long)]
    proposal_id: u64,

    #[clap(from_global)]
    ledger: bool,
}

pub fn exec(auth: &AuthInfo, opts: VoteOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    ensure!(!opts.ledger, "Cannot use `--ledger` with this command. This version of Quill does not support voting with a Ledger device.");
    let vote = if opts.approve {
        Vote::Yes
    } else if opts.reject {
        Vote::No
    } else {
        unreachable!();
    };
    let arg = ManageNeuron {
        id: Some(opts.neuron_id.0),
        command: Some(Command::RegisterVote(RegisterVote {
            proposal: Some(ProposalId {
                id: opts.proposal_id,
            }),
            vote: vote as i32,
        })),
        neuron_id_or_subaccount: None,
    };
    let msg = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "manage_neuron",
        Encode!(&arg)?,
    )?;
    Ok(vec![msg])
}
