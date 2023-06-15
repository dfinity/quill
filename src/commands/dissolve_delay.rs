use std::time::Duration;

use crate::lib::{
    governance_canister_id, now,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ROLE_NNS_GOVERNANCE,
};

use anyhow::ensure;
use candid::Encode;
use clap::{ArgGroup, Parser};
use humantime::Duration as HumanDuration;
use ic_nns_governance::pb::v1::{
    manage_neuron::{
        configure::Operation, Command, Configure, IncreaseDissolveDelay, SetDissolveTimestamp,
    },
    ManageNeuron,
};

/// Signs a neuron configuration change to increase a neuron's dissolve delay.
#[derive(Parser)]
#[clap(group(ArgGroup::new("delay").required(true)))]
pub struct DissolveDelayOpts {
    /// The ID of the neuron to configure.
    neuron_id: ParsedNeuron,

    /// Additional time to add to the neuron's dissolve delay, e.g. '1y'
    #[clap(long, group = "delay", value_name = "DURATION")]
    increase_by: Option<HumanDuration>,

    /// Total time to set the neuron's dissolve delay to, e.g. '4y'
    #[clap(long, group = "delay", value_name = "DURATION")]
    increase_to: Option<HumanDuration>,
}

const DELAY_MAX: Duration = Duration::from_secs(252_460_800);

pub fn exec(auth: &AuthInfo, opts: DissolveDelayOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let command = if let Some(by) = opts.increase_by {
        ensure!(*by <= DELAY_MAX, "Cannot increase by more than eight years");
        Command::Configure(Configure {
            operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                additional_dissolve_delay_seconds: by.as_secs() as u32,
            })),
        })
    } else if let Some(to) = opts.increase_to {
        ensure!(*to <= DELAY_MAX, "Cannot increase to more than eight years");
        // pad by a couple minutes to account for clock drift; user is likely trying to hit a milestone
        let to = (*to + Duration::from_secs(300)).max(DELAY_MAX);
        let now = now();
        let future = now + to;
        Command::Configure(Configure {
            operation: Some(Operation::SetDissolveTimestamp(SetDissolveTimestamp {
                dissolve_timestamp_seconds: future.unix_timestamp() as u64,
            })),
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
