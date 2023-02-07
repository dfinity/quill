use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_NNS_GOVERNANCE,
};
use anyhow::{anyhow, bail, Context};
use candid::{CandidType, Encode, Principal};
use clap::{ArgEnum, Parser};
use ic_base_types::PrincipalId;
use ic_nns_common::pb::v1::{NeuronId, ProposalId};
use ic_nns_governance::pb::v1::{
    manage_neuron::{
        configure::Operation, AddHotKey, ChangeAutoStakeMaturity, Command, Configure, Disburse,
        Follow, IncreaseDissolveDelay, JoinCommunityFund, LeaveCommunityFund, Merge, RegisterVote,
        RemoveHotKey, Split, StakeMaturity, StartDissolving, StopDissolving,
    },
    ManageNeuron,
};

// These constants are copied from src/governance.rs
pub const ONE_DAY_SECONDS: u32 = 24 * 60 * 60;
pub const ONE_YEAR_SECONDS: u32 = (4 * 365 + 1) * ONE_DAY_SECONDS / 4;
pub const ONE_MONTH_SECONDS: u32 = ONE_YEAR_SECONDS / 12;

#[derive(CandidType)]
pub struct AccountIdentifier {
    hash: Vec<u8>,
}

#[derive(Debug, Clone, Copy, ArgEnum)]
enum EnableState {
    Enabled,
    Disabled,
}

/// Signs a neuron configuration change.
#[derive(Parser)]
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

    /// Remove all followees for the NeuronManagement topic
    #[clap(long)]
    clear_manage_neuron_followees: bool,

    /// Merge stake, maturity and age from the neuron specified by this option into the neuron being managed.
    #[clap(long)]
    merge_from_neuron: Option<String>,

    /// Merge the percentage (between 1 and 100) of the maturity of a neuron into the current stake.
    #[clap(hide(true), long)]
    merge_maturity: Option<u32>,

    /// Stake a percentage (between 1 and 100) of the maturity of a neuron.
    #[clap(long)]
    stake_maturity: Option<u32>,

    /// Join the Internet Computer's community fund with this neuron's entire stake.
    #[clap(long)]
    join_community_fund: bool,

    /// Leave the Internet Computer's community fund.
    #[clap(long, conflicts_with("join-community-fund"))]
    leave_community_fund: bool,

    /// Defines the topic of a follow rule.
    #[clap(long)]
    follow_topic: Option<i32>,

    /// Defines the neuron ids of a follow rule.
    #[clap(long, multiple_values(true))]
    follow_neurons: Option<Vec<u64>>,

    /// Vote on proposal(s) (approve by default).
    #[clap(long, multiple_values(true))]
    register_vote: Option<Vec<u64>>,

    /// Reject proposal(s).
    #[clap(long)]
    reject: bool,

    /// Set whether new maturity should be automatically staked.
    #[clap(long, arg_enum)]
    auto_stake_maturity: Option<EnableState>,
}

pub fn exec(auth: &AuthInfo, opts: ManageOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let mut msgs = Vec::new();

    let id = Some(NeuronId {
        id: parse_neuron_id(opts.neuron_id)?,
    });
    if opts.add_hot_key.is_some() {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::AddHotKey(AddHotKey {
                    new_hot_key: opts.add_hot_key.map(PrincipalId)
                }))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.remove_hot_key.is_some() {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::RemoveHotKey(RemoveHotKey {
                    hot_key_to_remove: opts.remove_hot_key.map(PrincipalId)
                }))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.stop_dissolving {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::StopDissolving(StopDissolving {}))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    }

    if opts.start_dissolving {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::StartDissolving(StartDissolving {}))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    }

    if let Some(additional_dissolve_delay_seconds) = opts.additional_dissolve_delay_seconds {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
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
                            .context("Failed to parse the dissolve delay")?,
                    }
                }))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.disburse {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Disburse(Disburse {
                to_account: None,
                amount: None
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.spawn {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Spawn(Default::default())),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if let Some(amount) = opts.split {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Split(Split {
                amount_e8s: amount * 100_000_000
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.clear_manage_neuron_followees {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Follow(Follow {
                topic: 1, // Topic::NeuronManagement as i32,
                followees: Vec::new()
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    }

    if let Some(neuron_id) = opts.merge_from_neuron {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Merge(Merge {
                source_neuron_id: Some(NeuronId {
                    id: parse_neuron_id(neuron_id)?
                }),
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.merge_maturity.is_some() {
        bail!("Merging maturity is no longer a supported option. See --stake-maturity. https://wiki.internetcomputer.org/wiki/NNS_neuron_operations_related_to_maturity");
    };

    if let Some(percentage) = opts.stake_maturity {
        if !(1..=100).contains(&percentage) {
            bail!("Percentage to merge must be a number from 1 to 100");
        }
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::StakeMaturity(StakeMaturity {
                percentage_to_stake: Some(percentage),
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    }

    if opts.join_community_fund {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::JoinCommunityFund(JoinCommunityFund {}))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.leave_community_fund {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::LeaveCommunityFund(LeaveCommunityFund {}))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    }

    if let Some(proposals) = opts.register_vote {
        for proposal in proposals {
            let args = Encode!(&ManageNeuron {
                id: id.clone(),
                command: Some(Command::RegisterVote(RegisterVote {
                    vote: if opts.reject { 2 } else { 1 },
                    proposal: Some(ProposalId { id: proposal }),
                })),
                neuron_id_or_subaccount: None,
            })?;
            msgs.push(args);
        }
    };

    if let (Some(topic), Some(neuron_ids)) = (opts.follow_topic, opts.follow_neurons.as_ref()) {
        let followees = neuron_ids.iter().map(|x| NeuronId { id: *x }).collect();
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Follow(Follow {
                topic, // Topic::NeuronManagement as i32,
                followees,
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    } else if opts.follow_topic.is_some() {
        return Err(anyhow!("Follow topic specified without followees."));
    } else if opts.follow_neurons.is_some() {
        return Err(anyhow!("Followees specified without topic."));
    }

    if let Some(enable) = opts.auto_stake_maturity {
        let requested_setting_for_auto_stake_maturity = matches!(enable, EnableState::Enabled);
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::ChangeAutoStakeMaturity(
                    ChangeAutoStakeMaturity {
                        requested_setting_for_auto_stake_maturity,
                    }
                ))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    }

    if msgs.is_empty() {
        return Err(anyhow!("No instructions provided"));
    }

    let mut generated = Vec::new();
    for args in msgs {
        generated.push(sign_ingress_with_request_status_query(
            auth,
            governance_canister_id(),
            ROLE_NNS_GOVERNANCE,
            "manage_neuron",
            args,
        )?);
    }
    Ok(generated)
}

fn parse_neuron_id(id: String) -> AnyhowResult<u64> {
    id.replace('_', "")
        .parse()
        .context("Failed to parse the neuron id")
}
