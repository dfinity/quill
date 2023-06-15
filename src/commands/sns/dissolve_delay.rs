use std::time::Duration;

use anyhow::ensure;
use candid::Encode;
use clap::{ArgGroup, Parser};
use humantime::Duration as HumanDuration;
use ic_sns_governance::pb::v1::{
    manage_neuron::{
        configure::Operation, Command, Configure, IncreaseDissolveDelay, SetDissolveTimestamp,
    },
    ManageNeuron,
};

use crate::lib::{
    now,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_SNS_GOVERNANCE,
};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Signs a neuron configure message to increase the dissolve delay of a neuron.
/// The dissolve delay of a neuron determines its voting power, its ability to vote, its ability
/// to make proposals, and other actions it can take (such as disbursing).
#[derive(Parser)]
#[clap(group = ArgGroup::new("delay").required(true))]
pub struct DissolveDelayOpts {
    /// The ID of the neuron to configure.
    neuron_id: ParsedSnsNeuron,

    /// Additional time to add to the neuron's dissolve delay, e.g. '1y'
    #[clap(long, group = "delay", value_name = "DURATION")]
    increase_by: Option<HumanDuration>,
    /// Total time to set the neuron's dissolve delay to, e.g. '4y'
    #[clap(long, group = "delay", value_name = "DURATION")]
    increase_to: Option<HumanDuration>,
}

const DELAY_MAX: Duration = Duration::from_secs(252_460_800);

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: DissolveDelayOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
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
        command: Some(command),
        subaccount: opts.neuron_id.0.subaccount().unwrap().to_vec()
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
