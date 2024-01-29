use std::path::PathBuf;

use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AuthInfo, ROLE_SNS_GOVERNANCE,
    },
    AnyhowResult,
};
use anyhow::{Context, Error};
use candid::Principal;
use candid::{Encode, IDLArgs};
use candid_parser::parse_idl_args;
use clap::Parser;
use ic_sns_governance::pb::v1::{
    manage_neuron, proposal, ManageNeuron, Proposal, UpgradeSnsControlledCanister,
};
use k256::sha2::{Digest, Sha256};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Signs a ManageNeuron message to submit a UpgradeSnsControlledCanister
/// proposal.
#[derive(Parser)]
pub struct MakeUpgradeCanisterProposalOpts {
    /// The id of the neuron making the proposal. A hex encoded string.
    proposer_neuron_id: ParsedSnsNeuron,

    /// Title of the proposal.
    #[clap(long, default_value_t = String::from("Upgrade Canister"))]
    title: String,

    /// URL of the proposal.
    #[clap(long, default_value_t = String::new())]
    url: String,

    /// Summary of the proposal.
    /// If neither summary nor summary-path is provided, a somewhat generic summary will be
    /// constructed dynamically.
    #[clap(long)]
    summary: Option<String>,

    /// Path to a file containing the summary of the proposal.
    /// If neither summary nor summary-path is provided, a somewhat generic summary will be
    /// constructed dynamically.
    #[clap(long, conflicts_with("summary"))]
    summary_path: Option<PathBuf>,

    /// Canister to be upgraded.
    #[clap(long)]
    target_canister_id: Principal,

    /// Path to the WASM file to be installed onto the target canister.
    #[clap(long)]
    wasm_path: PathBuf,

    /// Argument to post-upgrade method of the new canister WASM. The argument must be formatted as a string
    /// wrapped candid record.
    #[clap(long)]
    canister_upgrade_arg: Option<String>,

    /// Path to the binary file containing argument to post-upgrade method of the new canister WASM.
    #[clap(long, conflicts_with("canister-upgrade-arg"))]
    canister_upgrade_arg_path: Option<String>,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: MakeUpgradeCanisterProposalOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let MakeUpgradeCanisterProposalOpts {
        proposer_neuron_id,
        title,
        url,
        summary,
        summary_path,
        target_canister_id,
        wasm_path,
        canister_upgrade_arg,
        canister_upgrade_arg_path,
    } = opts;

    let wasm = std::fs::read(wasm_path).context("Unable to read --wasm-path.")?;
    let canister_upgrade_arg = match (canister_upgrade_arg, canister_upgrade_arg_path) {
        (Some(arg), _) => {
            let parsed_arg: IDLArgs = parse_idl_args(&arg)?;
            Some(parsed_arg.to_bytes()?)
        }
        (_, Some(path)) => {
            Some(std::fs::read(path).context("Unable to read --canister-upgrade-arg-path.")?)
        }
        (None, None) => None,
    };

    // (Dynamically) come up with a summary if one wasn't provided.
    let summary = match (summary, summary_path) {
        (Some(arg), _) => arg,
        (_, Some(path)) => {
            String::from_utf8(std::fs::read(path).context("Unable to read --summary-path.")?)
                .context("Summary must be valid UTF-8.")?
        }
        (None, None) => summarize(target_canister_id, &wasm),
    };

    let proposal = Proposal {
        title,
        url,
        summary,
        action: Some(proposal::Action::UpgradeSnsControlledCanister(
            UpgradeSnsControlledCanister {
                canister_id: Some(target_canister_id.into()),
                new_canister_wasm: wasm,
                canister_upgrade_arg,
                mode: None,
            },
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

fn summarize(target_canister_id: Principal, wasm: &Vec<u8>) -> String {
    // Fingerprint wasm.
    let mut hasher = Sha256::new();
    hasher.update(wasm);
    let wasm_fingerprint = hex::encode(hasher.finalize());

    format!(
        "Upgrade canister:

  ID: {}

  WASM:
    length: {}
    fingerprint: {}",
        target_canister_id,
        wasm.len(),
        wasm_fingerprint
    )
}
