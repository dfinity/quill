use candid::{Encode, Principal};
use clap::Parser;
use ic_nns_governance::pb::v1::{
    manage_neuron::{Command, Spawn},
    ManageNeuron,
};

use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ROLE_NNS_GOVERNANCE,
};

/// Signs a neuron management message to convert a neuron's maturity into a rapidly-dissolving neuron.
#[derive(Parser)]
#[clap(alias = "disburse-maturity")]
pub struct SpawnOpts {
    /// The ID of the neuron to spawn from.
    neuron_id: ParsedNeuron,
    /// The owner of the spawned neuron.
    #[clap(long)]
    to: Option<Principal>,
    /// The percentage of the maturity to spawn.
    #[clap(long, value_parser = 1..=100)]
    percentage: Option<i64>,
}

pub fn exec(auth: &AuthInfo, opts: SpawnOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let arg = ManageNeuron {
        id: Some(opts.neuron_id.0),
        command: Some(Command::Spawn(Spawn {
            new_controller: opts.to.map(|x| x.into()),
            nonce: None,
            percentage_to_spawn: opts.percentage.map(|x| x as u32),
        })),
        neuron_id_or_subaccount: None,
    };
    let msg = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "manage_neuron",
        Encode!(&arg)?,
    )?;
    Ok(vec![msg])
}
