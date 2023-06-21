use candid::Encode;
use clap::Parser;
use ic_nns_governance::pb::v1::{
    manage_neuron::{Command, Merge},
    ManageNeuron,
};

use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ROLE_NNS_GOVERNANCE,
};

/// Signs a neuron management message to merge another neuron into this one.
#[derive(Parser)]
pub struct MergeOpts {
    /// The ID of the neuron to merge into.
    neuron_id: ParsedNeuron,

    /// The ID of the neuron to merge from.
    #[clap(long)]
    from: ParsedNeuron,
}

pub fn exec(auth: &AuthInfo, opts: MergeOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let arg = ManageNeuron {
        id: Some(opts.neuron_id.0),
        command: Some(Command::Merge(Merge {
            source_neuron_id: Some(opts.from.0),
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
