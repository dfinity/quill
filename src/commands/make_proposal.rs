use crate::lib::signing::{sign_ingress_with_request_status_query, IngressWithRequestId};
use crate::lib::{parse_neuron_id, TargetCanister};
use crate::{AnyhowResult, CanisterIds};
use anyhow::Error;
use candid::{Decode, Encode, IDLArgs};
use clap::Parser;
use ic_sns_governance::pb::v1::{manage_neuron, ManageNeuron, Proposal};

/// Signs a ManageNeuron::MakeProposal message to submit a proposal for voting.
#[derive(Parser)]
pub struct MakeProposalOpts {
    /// The id of the neuron to manage as a hex encoded string.
    neuron_id: String,

    #[clap(long)]
    /// The proposal to be submitted in string wrapped candid record.
    proposal: String,
}

pub fn exec(
    pem: &str,
    canister_ids: &CanisterIds,
    opts: MakeProposalOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let neuron_id = parse_neuron_id(opts.neuron_id)?;
    let neuron_subaccount = neuron_id.subaccount().map_err(Error::msg)?;
    let prop = parse_proposal_from_candid_string(opts.proposal)?;
    let governance_canister_id = canister_ids.governance_canister_id.get().0;

    let args = Encode!(&ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(manage_neuron::Command::MakeProposal(prop))
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

fn parse_proposal_from_candid_string(proposal_candid: String) -> AnyhowResult<Proposal> {
    let args: IDLArgs = proposal_candid.parse()?;
    let args: Vec<u8> = args.to_bytes()?;
    Decode!(args.as_slice(), Proposal).map_err(Error::msg)
}
