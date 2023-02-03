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
use candid::Principal;
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

    /// Summary of the proposal. If empty, a somewhat generic summary will be
    /// constructed dynamically.
    #[clap(long, default_value_t = String::new())]
    summary: String,

    /// Canister to be upgraded.
    #[clap(long)]
    target_canister_id: Principal,

    /// Path to the WASM file to be installed onto the target canister.
    #[clap(long)]
    wasm_path: PathBuf,

    /// Path to the file containing argument to post-upgrade method of the new canister WASM.
    #[clap(long)]
    canister_upgrade_arg_path: Option<PathBuf>,
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
        target_canister_id,
        wasm_path,
        canister_upgrade_arg_path,
    } = opts;

    let wasm = std::fs::read(wasm_path).context("Unable to read --wasm-path.")?;
    let canister_upgrade_arg = match canister_upgrade_arg_path {
        Some(path) => {
            Some(std::fs::read(path).context("Unable to read --canister-upgrade-arg-path.")?)
        }
        None => None,
    };

    // (Dynamically) come up with a summary if one wasn't provided.
    let summary = if !summary.is_empty() {
        summary
    } else {
        summarize(target_canister_id, &wasm)
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
