use anyhow::Error;
use candid::Encode;
use clap::{ArgGroup, Parser};
use ic_sns_governance::pb::v1::{
    manage_neuron::{
        configure::Operation, ChangeAutoStakeMaturity, Command, Configure, StakeMaturity,
    },
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
#[clap(group(ArgGroup::new("operation").required(true)))]
pub struct StakeMaturityOpts {
    /// The percentage of the current maturity to stake (1-100).
    #[clap(long, value_parser = 1..=100, group = "operation")]
    percentage: Option<i64>,

    /// Enable automatic maturity staking.
    #[clap(long, group = "operation")]
    automatic: bool,
    /// Disable automatic maturity staking.
    #[clap(long, group = "operation")]
    disable_automatic: bool,

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

    let command = if opts.automatic {
        Command::Configure(Configure {
            operation: Some(Operation::ChangeAutoStakeMaturity(
                ChangeAutoStakeMaturity {
                    requested_setting_for_auto_stake_maturity: true,
                },
            )),
        })
    } else if opts.disable_automatic {
        Command::Configure(Configure {
            operation: Some(Operation::ChangeAutoStakeMaturity(
                ChangeAutoStakeMaturity {
                    requested_setting_for_auto_stake_maturity: false,
                },
            )),
        })
    } else {
        Command::StakeMaturity(StakeMaturity {
            percentage_to_stake: Some(opts.percentage.unwrap_or(100) as u32),
        })
    };
    let arg = ManageNeuron {
        command: Some(command),
        subaccount: neuron_subaccount.to_vec(),
    };

    let message = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id,
        ROLE_SNS_GOVERNANCE,
        "manage_neuron",
        Encode!(&arg)?,
    )?;
    Ok(vec![message])
}
