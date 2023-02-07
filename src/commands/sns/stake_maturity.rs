use anyhow::Error;
use candid::Encode;
use clap::Parser;
use ic_sns_governance::pb::v1::{
    manage_neuron::{Command, StakeMaturity},
    ManageNeuron,
};

use crate::lib::{
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_SNS_GOVERNANCE,
};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Signs a ManageNeuron message to stake a percentage of a neuron's maturity.
///
/// A neuron's total stake is the combination of its staked governance tokens and staked maturity.
#[derive(Parser)]
pub struct StakeMaturityOpts {
    /// The percentage of the current maturity to stake (1-100).
    #[clap(long, value_parser = 1..=100)]
    percentage: i64,

    /// The id of the neuron to configure as a hex encoded string.
    neuron_id: ParsedSnsNeuron,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: StakeMaturityOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let neuron_subaccount = opts.neuron_id.0.subaccount().map_err(Error::msg)?;

    let governance_canister_id = sns_canister_ids.governance_canister_id;

    let command = ManageNeuron {
        command: Some(Command::StakeMaturity(StakeMaturity {
            percentage_to_stake: Some(opts.percentage as u32),
        })),
        subaccount: neuron_subaccount.to_vec(),
    };

    let message = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id,
        ROLE_SNS_GOVERNANCE,
        "manage_neuron",
        Encode!(&command)?,
    )?;
    Ok(vec![message])
}
