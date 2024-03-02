use crate::{
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AuthInfo, ROLE_SNS_GOVERNANCE,
    },
    AnyhowResult,
};
use anyhow::{anyhow, Error};
use candid::Encode;
use clap::Parser;

use ic_sns_governance::pb::v1::{
    manage_neuron,
    manage_neuron::{
        configure::Operation, Configure, IncreaseDissolveDelay, StartDissolving, StopDissolving,
    },
    ManageNeuron,
};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Signs a ManageNeuron message to configure the dissolve delay of a neuron. With this command
/// neuron holders can start dissolving, stop dissolving, or increase dissolve delay. The
/// dissolve delay of a neuron determines its voting power, its ability to vote, its ability
/// to make proposals, and other actions it can take (such as disbursing).
#[derive(Parser)]
pub struct ConfigureDissolveDelayOpts {
    /// The id of the neuron to configure as a hex encoded string.
    neuron_id: ParsedSnsNeuron,

    /// Additional number of seconds to add to the dissolve delay of a neuron. If the neuron is
    /// already dissolving and this argument is specified, the neuron will stop dissolving
    /// and begin aging
    #[clap(short, long)]
    additional_dissolve_delay_seconds: Option<u32>,

    /// When this argument is specified, the neuron will go into the dissolving state and a
    /// countdown timer will begin. When the timer is exhausted (i.e. dissolve_delay_seconds
    /// amount of time has elapsed), the neuron can be disbursed
    #[clap(long)]
    start_dissolving: bool,

    /// When this argument is specified, the neuron will exit the dissolving state and whatever
    /// amount of dissolve delay seconds is left in the countdown timer is stored. A neuron's
    /// dissolve delay can be extended (for instance to increase voting power) by using the
    /// additional_dissolve_delay_seconds flag
    #[clap(long, conflicts_with = "start-dissolving")]
    stop_dissolving: bool,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: ConfigureDissolveDelayOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    require_mutually_exclusive(
        opts.start_dissolving,
        opts.stop_dissolving,
        opts.additional_dissolve_delay_seconds,
    )?;

    let neuron_subaccount = opts.neuron_id.0.subaccount().map_err(Error::msg)?;
    let governance_canister_id = sns_canister_ids.governance_canister_id;

    let mut args = Vec::new();

    if opts.stop_dissolving {
        args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::StopDissolving(StopDissolving {}))
            })),
        })?;
    }

    if opts.start_dissolving {
        args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::StartDissolving(StartDissolving {}))
            })),
        })?;
    }

    if let Some(additional_dissolve_delay_seconds) = opts.additional_dissolve_delay_seconds {
        args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                    additional_dissolve_delay_seconds,
                }))
            })),
        })?;
    };

    let msg = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id,
        ROLE_SNS_GOVERNANCE,
        "manage_neuron",
        args,
    )?;
    Ok(vec![msg])
}

fn require_mutually_exclusive(
    stop_dissolving: bool,
    start_dissolving: bool,
    additional_dissolve_delay_seconds: Option<u32>,
) -> AnyhowResult {
    match (stop_dissolving, start_dissolving, additional_dissolve_delay_seconds) {
        (true, false, None)
        | (false, true, None)
        | (false, false, Some(_)) => Ok(()),
        _ => Err(anyhow!("--stop-dissolving, --start-dissolving, --additional-dissolve-delay-seconds are mutually exclusive arguments"))
    }
}
