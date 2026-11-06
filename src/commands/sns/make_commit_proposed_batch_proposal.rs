use core::panic;
use std::path::PathBuf;

use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AuthInfo, ROLE_SNS_GOVERNANCE,
    },
    AnyhowResult,
};
use anyhow::{Context, Error};
use candid::Encode;
use clap::Parser;
use ic_sns_governance::pb::v1::{
    manage_neuron, proposal, ExecuteGenericNervousSystemFunction, ManageNeuron, Proposal
};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Signs a ManageNeuron message to submit a ExecuteGenericNervousSystemFunction
/// proposal to commit proposed batch of assset canister.
#[derive(Parser)]
pub struct MakeCommitProposedBatchProposalOpts {
    /// The id of the neuron making the proposal. A hex encoded string.
    proposer_neuron_id: ParsedSnsNeuron,

    /// Title of the proposal.
    #[arg(long)]
    title: String,

    /// URL of the proposal.
    #[arg(long, default_value_t = String::new())]
    url: String,

    /// Summary of the proposal.
    /// Either summary or summary-path need to be provided.
    #[arg(long)]
    summary: Option<String>,

    /// Path to a file containing the summary of the proposal.
    /// If neither summary nor summary-path is provided, a somewhat generic summary will be
    /// constructed dynamically.
    #[arg(long, conflicts_with = "summary")]
    summary_path: Option<PathBuf>,

    /// The function id where the function calling commit_proposed_batch of asset canister is registered
    #[arg(long)]
    function_id: u64,

    /// The evidence (in hex format) displayed by dfx deploy --by-proposal.
    /// e.g. `Proposed commit of batch 2 with evidence e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855.`
    #[arg(long)]
    evidence: String,

    /// The batch id displayed by dfx deploy --by-proposal.
    #[arg(long)]
    batch_id: u128,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct CommitProposedBatchArguments {
  pub batch_id: u128,
  pub evidence: Vec<u8>,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: MakeCommitProposedBatchProposalOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let MakeCommitProposedBatchProposalOpts {
        proposer_neuron_id,
        title,
        url,
        summary,
        summary_path,
        function_id,
        evidence,
        batch_id
    } = opts;

    let commit_proposed_batch_arguments = CommitProposedBatchArguments {
        batch_id,
        evidence: hex::decode(evidence)?,
    };
    let payload: Vec<u8> = candid::Encode!(&commit_proposed_batch_arguments)?;

    let summary = match (summary, summary_path) {
        (Some(arg), _) => arg,
        (_, Some(path)) => {
            String::from_utf8(std::fs::read(path).context("Unable to read --summary-path.")?)
                .context("Summary must be valid UTF-8.")?
        }
        (None, None) => panic!("Summary must be provided"),
    };

    let proposal = Proposal {
        title,
        url,
        summary,
        action: Some(proposal::Action::ExecuteGenericNervousSystemFunction(
            ExecuteGenericNervousSystemFunction {
                function_id,
                payload,
            }
        )),
    };

    let neuron_id = proposer_neuron_id.0;
    let neuron_subaccount = neuron_id.subaccount().map_err(Error::msg)?;
    let governance_canister_id = sns_canister_ids.governance_canister_id;

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