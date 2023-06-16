use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AuthInfo, ROLE_SNS_GOVERNANCE,
    },
    AnyhowResult,
};
use anyhow::Error;
use candid::Encode;
use clap::{ArgGroup, Parser};
use ic_sns_governance::pb::v1::{
    manage_neuron, manage_neuron::RegisterVote, ManageNeuron, ProposalId, Vote,
};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Signs a ManageNeuron message to register a vote for a proposal. Registering a vote will
/// update the ballot of the given proposal and could trigger followees to vote. When
/// enough votes are cast or enough time passes, the proposal will either be rejected or
/// adopted and executed.
#[derive(Parser)]
#[clap(alias = "register-vote")]
#[clap(group(ArgGroup::new("yn").required(true)))]
pub struct VoteOpts {
    /// The id of the neuron to configure as a hex encoded string.
    neuron_id: ParsedSnsNeuron,

    /// The id of the proposal to be voted on.
    #[clap(long)]
    proposal_id: u64,

    /// The vote to be cast on the proposal [y/n]
    #[clap(long, hide = true, group = "yn", value_parser = ["y", "n"])]
    vote: Option<String>,

    /// Vote to approve the proposal.
    #[clap(long, group = "yn")]
    approve: bool,
    /// Vote to reject the proposal.
    #[clap(long, group = "yn")]
    reject: bool,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: VoteOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let neuron_subaccount = opts.neuron_id.0.subaccount().map_err(Error::msg)?;
    let governance_canister_id = sns_canister_ids.governance_canister_id;

    let vote = if opts.approve {
        Vote::Yes
    } else if opts.reject {
        Vote::No
    } else {
        match opts.vote.unwrap().as_str() {
            "y" => Vote::Yes,
            "n" => Vote::No,
            _ => unreachable!(),
        }
    };

    let args = Encode!(&ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(manage_neuron::Command::RegisterVote(RegisterVote {
            proposal: Some(ProposalId {
                id: opts.proposal_id
            }),
            vote: vote as i32
        }))
    })?;

    let msg = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id,
        ROLE_SNS_GOVERNANCE,
        "manage_neuron",
        args,
    )?;

    Ok(vec![msg])
}
