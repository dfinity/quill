use crate::lib::signing::{sign_ingress_with_request_status_query, IngressWithRequestId};
use crate::lib::{parse_neuron_id, TargetCanister};
use crate::{AnyhowResult, CanisterIds};
use anyhow::Error;
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;

use ic_sns_governance::pb::v1::manage_neuron;
use ic_sns_governance::pb::v1::manage_neuron::configure::Operation;
use ic_sns_governance::pb::v1::manage_neuron::{
    Configure, IncreaseDissolveDelay, StartDissolving, StopDissolving,
};
use ic_sns_governance::pb::v1::ManageNeuron;

// These constants are copied from src/governance.rs
pub const ONE_DAY_SECONDS: u32 = 24 * 60 * 60;
pub const ONE_YEAR_SECONDS: u32 = (4 * 365 + 1) * ONE_DAY_SECONDS / 4;
pub const ONE_MONTH_SECONDS: u32 = ONE_YEAR_SECONDS / 12;

/// Signs a ManageNeuron::Configure message to configure the dissolve delay of a neuron.
#[derive(Parser)]
pub struct ConfigureDissolveDelayOpts {
    /// The id of the neuron to manage as a hex encoded string.
    neuron_id: String,

    /// Number of dissolve seconds to add.
    #[clap(short, long)]
    additional_dissolve_delay_seconds: Option<String>,

    /// Start dissolving.
    #[clap(long)]
    start_dissolving: bool,

    /// Stop dissolving.
    #[clap(long)]
    stop_dissolving: bool,
}

pub fn exec(
    pem: &str,
    canister_ids: &CanisterIds,
    opts: ConfigureDissolveDelayOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let mut msgs = Vec::new();

    let neuron_id = parse_neuron_id(opts.neuron_id)?;
    let neuron_subaccount = neuron_id.subaccount().map_err(Error::msg)?;

    let governance_canister_id = PrincipalId::from(canister_ids.governance_canister_id).0;

    if opts.stop_dissolving {
        let args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::StopDissolving(StopDissolving {}))
            })),
        })?;
        msgs.push(args);
    }

    if opts.start_dissolving {
        let args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::StartDissolving(StartDissolving {}))
            })),
        })?;
        msgs.push(args);
    }

    if let Some(additional_dissolve_delay_seconds) = opts.additional_dissolve_delay_seconds {
        let args = Encode!(&ManageNeuron {
            subaccount: neuron_subaccount.to_vec(),
            command: Some(manage_neuron::Command::Configure(Configure {
                operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                    additional_dissolve_delay_seconds: match additional_dissolve_delay_seconds
                        .as_ref()
                    {
                        "ONE_DAY" => ONE_DAY_SECONDS,

                        "ONE_WEEK" => ONE_DAY_SECONDS * 7,
                        "TWO_WEEKS" => ONE_DAY_SECONDS * 7 * 2,
                        "THREE_WEEKS" => ONE_DAY_SECONDS * 7 * 3,
                        "FOUR_WEEKS" => ONE_DAY_SECONDS * 7 * 4,

                        "ONE_MONTH" => ONE_MONTH_SECONDS,
                        "TWO_MONTHS" => ONE_MONTH_SECONDS * 2,
                        "THREE_MONTHS" => ONE_MONTH_SECONDS * 3,
                        "FOUR_MONTHS" => ONE_MONTH_SECONDS * 4,
                        "FIVE_MONTHS" => ONE_MONTH_SECONDS * 5,
                        "SIX_MONTHS" => ONE_MONTH_SECONDS * 6,
                        "SEVEN_MONTHS" => ONE_MONTH_SECONDS * 7,
                        "EIGHT_MONTHS" => ONE_MONTH_SECONDS * 8,
                        "NINE_MONTHS" => ONE_MONTH_SECONDS * 9,
                        "TEN_MONTHS" => ONE_MONTH_SECONDS * 10,
                        "ELEVEN_MONTHS" => ONE_MONTH_SECONDS * 11,

                        "ONE_YEAR" => ONE_YEAR_SECONDS,
                        "TWO_YEARS" => ONE_YEAR_SECONDS * 2,
                        "THREE_YEARS" => ONE_YEAR_SECONDS * 3,
                        "FOUR_YEARS" => ONE_YEAR_SECONDS * 4,
                        "FIVE_YEARS" => ONE_YEAR_SECONDS * 5,
                        "SIX_YEARS" => ONE_YEAR_SECONDS * 6,
                        "SEVEN_YEARS" => ONE_YEAR_SECONDS * 7,
                        "EIGHT_YEARS" => ONE_YEAR_SECONDS * 8,

                        s => s
                            .parse::<u32>()
                            .expect("Failed to parse the dissolve delay"),
                    }
                }))
            })),
        })?;
        msgs.push(args);
    };

    let mut generated = Vec::new();
    for args in msgs {
        generated.push(sign_ingress_with_request_status_query(
            pem,
            governance_canister_id,
            "manage_neuron",
            args,
            TargetCanister::Governance,
        )?);
    }
    Ok(generated)
}
