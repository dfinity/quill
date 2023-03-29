use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AuthInfo, ROLE_SNS_GOVERNANCE,
    },
    AnyhowResult,
};
use anyhow::{anyhow, Error};
use candid::Encode;
use clap::Parser;
use ic_sns_governance::pb::v1::{
    manage_neuron, manage_neuron::RegisterVote, ManageNeuron, ProposalId, Vote,
};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Signs a ManageNeuron message to register a vote for a proposal. Registering a vote will
/// update the ballot of the given proposal and could trigger followees to vote. When
/// enough votes are cast or enough time passes, the proposal will either be rejected or
/// adopted and executed.
#[derive(Parser)]
pub struct RegisterVoteOpts {
    /// The id of the neuron to configure as a hex encoded string.
    neuron_id: ParsedSnsNeuron,

    /// The id of the proposal to voted on.
    #[clap(long)]
    proposal_id: u64,

    /// The vote to be cast on the proposal [y/n]
    #[clap(long)]
    vote: String,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: RegisterVoteOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let neuron_subaccount = opts.neuron_id.0.subaccount().map_err(Error::msg)?;
    let governance_canister_id = sns_canister_ids.governance_canister_id;

    let vote = match opts.vote.as_str() {
        "y" => Ok(Vote::Yes),
        "n" => Ok(Vote::No),
        _ => Err(anyhow!(
            "Unsupported vote supplied to --vote. Supported values: ['y', 'n']"
        )),
    }?;

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
