use candid::Encode;
use clap::{ArgGroup, Parser};
use ic_nns_governance::pb::v1::{
    manage_neuron::{configure::Operation, Command, Configure, StartDissolving, StopDissolving},
    ManageNeuron,
};

use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ROLE_NNS_GOVERNANCE,
};

/// Signs a neuron configuration message to start or stop a neuron dissolving.
#[derive(Parser)]
#[clap(group = ArgGroup::new("state").required(true))]
pub struct DissolveOpts {
    /// The ID of the neuron to dissolve.
    neuron_id: ParsedNeuron,

    /// Start dissolving the neuron.
    #[clap(long, group = "state")]
    start: bool,

    /// Stop dissolving the neuron.
    #[clap(long, group = "state")]
    stop: bool,
}

pub fn exec(auth: &AuthInfo, opts: DissolveOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let command = if opts.start {
        Command::Configure(Configure {
            operation: Some(Operation::StartDissolving(StartDissolving {})),
        })
    } else if opts.stop {
        Command::Configure(Configure {
            operation: Some(Operation::StopDissolving(StopDissolving {})),
        })
    } else {
        unreachable!()
    };
    let arg = Encode!(&ManageNeuron {
        id: Some(opts.neuron_id.0),
        command: Some(command),
        neuron_id_or_subaccount: None,
    })?;
    let msg = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "manage_neuron",
        arg,
    )?;
    Ok(vec![msg])
}
