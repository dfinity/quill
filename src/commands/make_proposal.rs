use std::{fs, path::PathBuf};

use crate::{
    lib::{
        get_local_candid, governance_canister_id,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AuthInfo, ROLE_NNS_GOVERNANCE,
    },
    AnyhowResult,
};
use anyhow::Error;
use candid::{CandidType, Decode, Encode, TypeEnv};
use candid_parser::{check_prog, parse_idl_args};
use clap::Parser;
use ic_nns_common::pb::v1::NeuronId;
use ic_nns_governance::pb::v1::{
    manage_neuron::{self, NeuronIdOrSubaccount},
    ManageNeuron, Proposal,
};

use super::neuron_manage::parse_neuron_id;

/// Creates an NNS proposal for others to vote on.
#[derive(Parser)]
pub struct MakeProposalOpts {
    /// The id of the neuron making the proposal.
    #[arg(value_parser = parse_neuron_id)]
    proposer_neuron_id: u64,

    /// The proposal to be submitted. The proposal must be formatted as a string
    /// wrapped candid record.
    ///
    /// For example:
    /// '(record {
    ///     title=opt "Known Neuron Proposal";
    ///     url="http://example.com";
    ///     summary="A proposal to become a named neuron";
    ///     action=opt variant {
    ///         RegisterKnownNeuron = record {
    ///             id=opt record { id=773; };
    ///             known_neuron_data=opt record { name="Me!" };
    ///         }
    ///     };
    /// })'
    #[arg(long)]
    proposal: Option<String>,

    /// Path to a file containing the proposal. The proposal must be the binary encoding of
    /// the proposal candid record.
    #[arg(
        long,
        conflicts_with = "proposal",
        required_unless_present = "proposal"
    )]
    proposal_path: Option<PathBuf>,
}

pub fn exec(auth: &AuthInfo, opts: MakeProposalOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let neuron_id = opts.proposer_neuron_id;

    let proposal = if let Some(proposal) = opts.proposal {
        parse_nns_proposal_from_candid_string(proposal)?
    } else {
        Decode!(&fs::read(opts.proposal_path.unwrap())?, Proposal)?
    };

    let args = Encode!(&ManageNeuron {
        id: None,
        neuron_id_or_subaccount: Some(NeuronIdOrSubaccount::NeuronId(NeuronId { id: neuron_id })),
        command: Some(manage_neuron::Command::MakeProposal(Box::new(proposal)))
    })?;

    let msg = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "manage_neuron",
        args,
    )?;

    Ok(vec![msg])
}

fn parse_nns_proposal_from_candid_string(proposal_candid: String) -> AnyhowResult<Proposal> {
    let args = parse_idl_args(&proposal_candid)?;
    let mut env = TypeEnv::default();
    check_prog(
        &mut env,
        &get_local_candid(governance_canister_id(), ROLE_NNS_GOVERNANCE)?.parse()?,
    )?;
    let args: Vec<u8> = args.to_bytes_with_types(&env, &[Proposal::ty()])?;
    Decode!(args.as_slice(), Proposal).map_err(Error::msg)
}
