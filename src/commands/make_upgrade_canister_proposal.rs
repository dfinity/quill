use crate::{
    lib::{
        parse_neuron_id,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        TargetCanister,
    },
    AnyhowResult, SnsCanisterIds,
};
use anyhow::{Context, Error};
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_sns_governance::pb::v1::{
    manage_neuron, proposal, ManageNeuron, Proposal, UpgradeSnsControlledCanister,
};
use ic_types::Principal;
use sha2::{Digest, Sha256};

/// Signs a ManageNeuron message to submit a UpgradeSnsControlled canister
/// proposal.
#[derive(Parser)]
pub struct MakeUpgradeCanisterProposalOpts {
    /// The id of the neuron making the proposal. A hex encoded string. For example:
    /// 83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
    proposer_neuron_id: String,

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
    target_canister_id: String,

    /// Path to the WASM file to be installed onto the target canister.
    #[clap(long)]
    wasm_path: String,
}

pub fn exec(
    private_key_pem: &str,
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
    } = opts;

    let wasm = std::fs::read(wasm_path).context("Unable to read --wasm-path.")?;

    // (Dynamically) come up with a summary if one wasn't provided.
    let summary = if !summary.is_empty() {
        summary
    } else {
        let mut hasher = Sha256::new();
        hasher.update(&wasm);
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
    };

    let proposal = Proposal {
        title,
        url,
        summary,
        action: Some(proposal::Action::UpgradeSnsControlledCanister(
            UpgradeSnsControlledCanister {
                canister_id: Some(PrincipalId(Principal::from_text(target_canister_id)?)),
                new_canister_wasm: wasm,
            },
        )),
    };

    let neuron_id = parse_neuron_id(proposer_neuron_id)?;
    let neuron_subaccount = neuron_id.subaccount().map_err(Error::msg)?;
    let governance_canister_id = sns_canister_ids.governance_canister_id.get().0;

    let args = Encode!(&ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(manage_neuron::Command::MakeProposal(proposal))
    })?;

    let msg = sign_ingress_with_request_status_query(
        private_key_pem,
        "manage_neuron",
        args,
        TargetCanister::Governance(governance_canister_id),
    )?;

    Ok(vec![msg])
}
