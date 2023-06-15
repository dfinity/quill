use candid::Encode;
use clap::{ArgGroup, Parser};
use ic_sns_governance::pb::v1::{
    manage_neuron::{configure::Operation, Command, Configure, StartDissolving, StopDissolving},
    ManageNeuron,
};

use crate::lib::{
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_SNS_GOVERNANCE,
};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Signs a neuron configuration message to start or stop dissolving.
#[derive(Parser)]
#[clap(group = ArgGroup::new("state").required(true))]
pub struct DissolveOpts {
    /// The neuron being dissolved.
    neuron_id: ParsedSnsNeuron,

    /// The neuron will go into the dissolving state and a
    /// countdown timer will begin. When the timer is exhausted (i.e. the dissolve delay
    /// has elapsed), the neuron can be disbursed.
    #[clap(long, group = "state")]
    start: bool,
    /// The neuron will exit the dissolving state and whatever
    /// amount of time is left in the countdown timer is stored. A neuron's
    /// dissolve delay can be extended (for instance to increase voting power) by using the
    /// `quill sns dissolve-delay` command.
    #[clap(long, group = "state")]
    stop: bool,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: DissolveOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
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
        command: Some(command),
        subaccount: opts.neuron_id.0.subaccount().unwrap().to_vec(),
    })?;
    let msg = sign_ingress_with_request_status_query(
        auth,
        sns_canister_ids.governance_canister_id,
        ROLE_SNS_GOVERNANCE,
        "manage_neuron",
        arg,
    )?;
    Ok(vec![msg])
}
