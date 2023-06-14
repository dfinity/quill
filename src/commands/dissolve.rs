use std::time::Duration;

use anyhow::ensure;
use candid::Encode;
use clap::{ArgGroup, Parser};
use humantime::Duration as HumanDuration;
use ic_nns_governance::pb::v1::{
    manage_neuron::{
        configure::Operation, Command, Configure, IncreaseDissolveDelay, SetDissolveTimestamp,
        StartDissolving, StopDissolving,
    },
    ManageNeuron,
};
use time::OffsetDateTime;

use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ROLE_NNS_GOVERNANCE,
};

/// Signs a neuron configuration message to change a neuron's dissolve delay.
#[derive(Parser)]
#[clap(group = ArgGroup::new("operation").required(true))]
pub struct DissolveOpts {
    /// ID of the neuron to manage.
    neuron_id: ParsedNeuron,

    /// Additional time to add to the neuron's dissolve delay, e.g. '1y'
    #[clap(long, group = "operation")]
    increase_delay_by: Option<HumanDuration>,

    /// Total time to set the neuron's dissolve delay to, e.g. '4y'
    #[clap(long, group = "operation")]
    increase_delay_to: Option<HumanDuration>,

    /// Start dissolving the neuron
    #[clap(long, group = "operation")]
    start: bool,

    /// Stop dissolving the neuron
    #[clap(long, group = "operation")]
    stop: bool,
}

const DELAY_MAX: Duration = Duration::from_secs(252_460_800);

pub fn exec(auth: &AuthInfo, opts: DissolveOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let command = if let Some(by) = opts.increase_delay_by {
        ensure!(*by <= DELAY_MAX, "Cannot increase by more than eight years");
        Command::Configure(Configure {
            operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                additional_dissolve_delay_seconds: by.as_secs() as u32,
            })),
        })
    } else if let Some(to) = opts.increase_delay_to {
        ensure!(*to <= DELAY_MAX, "Cannot increase to more than eight years");
        // pad by a couple minutes to account for clock drift; user is likely trying to hit a milestone
        let to = (*to + Duration::from_secs(120)).max(DELAY_MAX);
        let now = OffsetDateTime::now_utc();
        let future = now + to;
        Command::Configure(Configure {
            operation: Some(Operation::SetDissolveTimestamp(SetDissolveTimestamp {
                dissolve_timestamp_seconds: future.unix_timestamp() as u64,
            })),
        })
    } else if opts.start {
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
