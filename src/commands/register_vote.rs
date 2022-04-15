use crate::lib::signing::{sign_ingress_with_request_status_query, IngressWithRequestId};
use crate::lib::{parse_neuron_id, TargetCanister};
use crate::{AnyhowResult, CanisterIds};
use anyhow::{anyhow, Error};
use candid::Encode;
use clap::Parser;
use ic_sns_governance::pb::v1::manage_neuron::RegisterVote;
use ic_sns_governance::pb::v1::{manage_neuron, ManageNeuron, ProposalId};

/// Signs a ManageNeuron::Configure message to register a vote for a proposal.
#[derive(Parser)]
pub struct RegisterVoteOpts {
    /// The id of the neuron to manage as a hex encoded string.
    neuron_id: String,

    #[clap(long)]
    /// The id of the proposal to vote on
    proposal_id: u64,

    #[clap(long)]
    /// The vote cast on the proposal (y/n)
    vote: String,
}

#[repr(i32)]
enum Vote {
    Yes = 1,
    No = 2,
}

pub fn exec(
    pem: &str,
    canister_ids: &CanisterIds,
    opts: RegisterVoteOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let id = parse_neuron_id(opts.neuron_id)?;
    let neuron_subaccount = id.subaccount().map_err(Error::msg)?;
    let governance_canister_id = canister_ids.governance_canister_id.get().0;

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
        pem,
        governance_canister_id,
        "manage_neuron",
        args,
        TargetCanister::Governance,
    )?;

    Ok(vec![msg])
}
