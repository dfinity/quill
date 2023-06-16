use candid::Encode;
use clap::{ArgGroup, Parser};
use ic_nns_governance::pb::v1::{
    manage_neuron::{
        configure::Operation, ChangeAutoStakeMaturity, Command, Configure, StakeMaturity,
    },
    ManageNeuron,
};

use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ROLE_NNS_GOVERNANCE,
};

/// Signs a neuron management message to add maturity to a neuron's stake, or configure auto-staking.
#[derive(Parser)]
#[clap(group(ArgGroup::new("operation").required(true)))]
pub struct StakeMaturityOpts {
    /// The ID of the neuron to manage.
    neuron_id: ParsedNeuron,
    /// The percentage of the neuron's accrued maturity to stake.
    #[clap(long, value_parser = 1..=100, group = "operation")]
    percentage: Option<i64>,
    /// Enable automatic maturity staking.
    #[clap(long, group = "operation")]
    automatic: bool,
    /// Disable automatic maturity staking.
    #[clap(long, group = "operation")]
    disable_automatic: bool,
}

pub fn exec(auth: &AuthInfo, opts: StakeMaturityOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
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
            percentage_to_stake: Some(opts.percentage.unwrap() as u32),
        })
    };
    let arg = ManageNeuron {
        id: Some(opts.neuron_id.0),
        command: Some(command),
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
