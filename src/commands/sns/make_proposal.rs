use std::{fs, path::PathBuf};

use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AuthInfo, ROLE_SNS_GOVERNANCE,
    },
    AnyhowResult,
};
use anyhow::Error;
use candid::{CandidType, Decode, Encode, TypeEnv};
use candid_parser::parse_idl_args;
use clap::Parser;
use ic_sns_governance::pb::v1::{manage_neuron, ManageNeuron, Proposal};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Signs a ManageNeuron message to submit a proposal. With this command, neuron holders
/// can submit proposals (such as a Motion Proposal) to be voted on by other neuron
/// holders.
#[derive(Parser)]
pub struct MakeProposalOpts {
    /// The id of the neuron making the proposal as a hex encoded string.
    proposer_neuron_id: ParsedSnsNeuron,

    /// The proposal to be submitted. The proposal must be formatted as a string
    /// wrapped candid record.
    ///
    /// For example:
    /// '(
    ///     record {
    ///         title="SNS Launch";
    ///         url="https://dfinity.org";
    ///         summary="A motion to start the SNS";
    ///         action=opt variant {
    ///             Motion=record {
    ///                 motion_text="I hereby raise the motion that the use of the SNS shall commence";
    ///             }
    ///         };
    ///     }
    /// )'
    #[clap(long)]
    proposal: Option<String>,

    /// Path to a file containing the proposal. The proposal must be the binary encoding of
    /// the proposal candid record.
    #[clap(
        long,
        conflicts_with = "proposal",
        required_unless_present = "proposal"
    )]
    proposal_path: Option<PathBuf>,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: MakeProposalOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let neuron_id = opts.proposer_neuron_id.0;
    let neuron_subaccount = neuron_id.subaccount().map_err(Error::msg)?;
    let governance_canister_id = sns_canister_ids.governance_canister_id;

    let proposal = if let Some(proposal) = opts.proposal {
        parse_proposal_from_candid_string(proposal)?
    } else {
        Decode!(&fs::read(opts.proposal_path.unwrap())?, Proposal)?
    };

    let args = Encode!(&ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(manage_neuron::Command::MakeProposal(proposal))
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

fn parse_proposal_from_candid_string(proposal_candid: String) -> AnyhowResult<Proposal> {
    let args = parse_idl_args(&proposal_candid)?;
    let args: Vec<u8> = args.to_bytes_with_types(&TypeEnv::default(), &[Proposal::ty()])?;
    Decode!(args.as_slice(), Proposal).map_err(Error::msg)
}
