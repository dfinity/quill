use crate::{
    commands::sign::sign_ingress_with_request_status_query,
    lib::{governance_canister_id, sign::signed_message::IngressWithRequestId, AnyhowResult},
};
use anyhow::anyhow;
use candid::{CandidType, Encode};
use clap::Clap;
use ic_types::Principal;
use ledger_canister::{AccountIdentifier, ICPTs};

// These constants are copied from src/governance.rs
pub const ONE_DAY_SECONDS: u32 = 24 * 60 * 60;
pub const ONE_YEAR_SECONDS: u32 = (4 * 365 + 1) * ONE_DAY_SECONDS / 4;
pub const ONE_MONTH_SECONDS: u32 = ONE_YEAR_SECONDS / 12;

#[derive(CandidType)]
pub struct IncreaseDissolveDelay {
    pub additional_dissolve_delay_seconds: u32,
}

#[derive(CandidType, Copy, Clone)]
pub struct NeuronId {
    pub id: u64,
}

#[derive(CandidType)]
pub struct StartDissolving {}

#[derive(CandidType)]
pub struct StopDissolving {}

#[derive(CandidType)]
pub struct RemoveHotKey {
    pub hot_key_to_remove: Option<Principal>,
}

#[derive(CandidType)]
pub struct AddHotKey {
    pub new_hot_key: Option<Principal>,
}

#[derive(CandidType)]
pub enum Operation {
    RemoveHotKey(RemoveHotKey),
    StartDissolving(StartDissolving),
    StopDissolving(StopDissolving),
    AddHotKey(AddHotKey),
    IncreaseDissolveDelay(IncreaseDissolveDelay),
}

#[derive(CandidType)]
pub struct Configure {
    pub operation: Option<Operation>,
}

#[derive(CandidType)]
pub struct Disburse {
    pub to_account: Option<AccountIdentifier>,
    pub amount: Option<ICPTs>,
}

#[derive(CandidType, Default)]
pub struct Spawn {
    pub new_controller: Option<Principal>,
}

#[derive(CandidType)]
pub struct Split {
    pub amount_e8s: u64,
}

#[derive(candid::CandidType)]
pub struct MergeMaturity {
    pub percentage_to_merge: u32,
}

#[derive(CandidType)]
pub enum Command {
    Configure(Configure),
    Disburse(Disburse),
    Spawn(Spawn),
    Split(Split),
    MergeMaturity(MergeMaturity),
}

#[derive(CandidType)]
struct ManageNeuron {
    id: Option<NeuronId>,
    command: Option<Command>,
}

/// Signs a neuron configuration change.
#[derive(Clap)]
pub struct ManageOpts {
    /// The id of the neuron to manage.
    neuron_id: String,

    /// Principal to be used as a hot key.
    #[clap(long)]
    add_hot_key: Option<Principal>,

    /// Principal hot key to be removed.
    #[clap(long)]
    remove_hot_key: Option<Principal>,

    /// Number of dissolve seconds to add.
    #[clap(short, long)]
    additional_dissolve_delay_seconds: Option<String>,

    /// Start dissolving.
    #[clap(long)]
    start_dissolving: bool,

    /// Stop dissolving.
    #[clap(long)]
    stop_dissolving: bool,

    /// Disburse the entire staked amount to the controller's account.
    #[clap(long)]
    disburse: bool,

    /// Spawn rewards to a new neuron under the controller's account.
    #[clap(long)]
    spawn: bool,

    /// Split off the given number of ICP from a neuron.
    #[clap(long)]
    split: Option<u64>,

    /// Merge the percentage (between 1 and 100) of the maturity of a neuron into the current stake.
    #[clap(long)]
    merge_maturity: Option<u32>,
}

pub async fn exec(
    pem: &Option<String>,
    opts: ManageOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let mut msgs = Vec::new();

    let id = Some(NeuronId {
        id: parse_neuron_id(opts.neuron_id),
    });
    if opts.add_hot_key.is_some() {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::AddHotKey(AddHotKey {
                    new_hot_key: opts.add_hot_key
                }))
            }))
        })?;
        msgs.push(args);
    };

    if opts.remove_hot_key.is_some() {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::RemoveHotKey(RemoveHotKey {
                    hot_key_to_remove: opts.remove_hot_key
                }))
            }))
        })?;
        msgs.push(args);
    };

    if opts.stop_dissolving {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::StopDissolving(StopDissolving {}))
            }))
        })?;
        msgs.push(args);
    }

    if opts.start_dissolving {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::StartDissolving(StartDissolving {}))
            }))
        })?;
        msgs.push(args);
    }

    if let Some(additional_dissolve_delay_seconds) = opts.additional_dissolve_delay_seconds {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Configure(Configure {
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

                        "ONE_YEAR" => ONE_YEAR_SECONDS * 12,
                        "TWO_YEARS" => ONE_YEAR_SECONDS * 12 * 2,
                        "THREE_YEARS" => ONE_YEAR_SECONDS * 12 * 3,
                        "FOUR_YEARS" => ONE_YEAR_SECONDS * 12 * 4,
                        "FIVE_YEARS" => ONE_YEAR_SECONDS * 12 * 5,
                        "SIX_YEARS" => ONE_YEAR_SECONDS * 12 * 6,
                        "SEVEN_YEARS" => ONE_YEAR_SECONDS * 12 * 7,
                        "EIGHT_YEARS" => ONE_YEAR_SECONDS * 12 * 8,

                        s => s.parse::<u32>().expect("Couldn't parse the dissolve delay"),
                    }
                }))
            }))
        })?;
        msgs.push(args);
    };

    if opts.disburse {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Disburse(Disburse {
                to_account: None,
                amount: None
            }))
        })?;
        msgs.push(args);
    };

    if opts.spawn {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Spawn(Default::default()))
        })?;
        msgs.push(args);
    };

    if let Some(amount) = opts.split {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Split(Split {
                amount_e8s: amount * 100_000_000
            }))
        })?;
        msgs.push(args);
    };

    if let Some(percentage_to_merge) = opts.merge_maturity {
        if percentage_to_merge == 0 || percentage_to_merge > 100 {
            return Err(anyhow!(
                "Percentage to merge must be a number from 1 to 100"
            ));
        }
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::MergeMaturity(MergeMaturity {
                percentage_to_merge
            }))
        })?;
        msgs.push(args);
    };

    if msgs.is_empty() {
        return Err(anyhow!("No instructions provided"));
    }

    let mut generated = Vec::new();
    for args in msgs {
        generated.push(
            sign_ingress_with_request_status_query(
                pem,
                governance_canister_id(),
                "manage_neuron",
                args,
            )
            .await?,
        );
    }
    Ok(generated)
}

fn parse_neuron_id(id: String) -> u64 {
    id.replace("_", "")
        .parse()
        .expect("Couldn't parse the neuron id")
}
