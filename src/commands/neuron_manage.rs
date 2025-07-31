use crate::commands::transfer::parse_tokens;
use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNnsAccount, ROLE_NNS_GOVERNANCE,
};
use anyhow::{anyhow, bail, ensure, Context};
use candid::{CandidType, Encode, Principal};
use clap::{Parser, ValueEnum};
use ic_base_types::PrincipalId;
use ic_nns_common::pb::v1::{NeuronId, ProposalId};
use ic_nns_governance::pb::v1::manage_neuron::{DisburseMaturity, RefreshVotingPower};
use ic_nns_governance::pb::v1::{
    manage_neuron::{
        configure::Operation, disburse::Amount, AddHotKey, ChangeAutoStakeMaturity, Command,
        Configure, Disburse, Follow, IncreaseDissolveDelay, JoinCommunityFund, LeaveCommunityFund,
        Merge, NeuronIdOrSubaccount, RegisterVote, RemoveHotKey, SetVisibility, Split,
        StakeMaturity, StartDissolving, StopDissolving,
    },
    ManageNeuron,
};
use icp_ledger::Tokens;

mod pb {
    pub use ic_nns_governance::pb::v1::Visibility;
}

// These constants are copied from src/governance.rs
pub const ONE_DAY_SECONDS: u32 = 24 * 60 * 60;
pub const ONE_YEAR_SECONDS: u32 = (4 * 365 + 1) * ONE_DAY_SECONDS / 4;
pub const ONE_MONTH_SECONDS: u32 = ONE_YEAR_SECONDS / 12;

#[derive(CandidType)]
pub struct AccountIdentifier {
    hash: Vec<u8>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
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
    #[arg(long)]
    add_hot_key: Option<Principal>,

    /// Principal hot key to be removed.
    #[arg(long)]
    remove_hot_key: Option<Principal>,

    /// Number of dissolve seconds to add.
    #[arg(short, long)]
    additional_dissolve_delay_seconds: Option<String>,

    /// Start dissolving.
    #[arg(long)]
    start_dissolving: bool,

    /// Stop dissolving.
    #[arg(long)]
    stop_dissolving: bool,

    /// Disburse the entire staked amount to the controller's account.
    #[arg(long)]
    disburse: bool,

    /// Disburse only the selected amount, instead of the entire amount, to the controller's account.
    #[arg(long, value_parser = parse_tokens)]
    disburse_amount: Option<Tokens>,

    /// Disburse to the selected NNS account instead of the controller.
    #[arg(long)]
    disburse_to: Option<ParsedNnsAccount>,

    /// Spawn rewards to a new neuron under the controller's account.
    #[arg(long)]
    spawn: bool,

    /// Split off the given number of ICP from a neuron.
    #[arg(long)]
    split: Option<u64>,

    /// Remove all followees for the NeuronManagement topic
    #[arg(long)]
    clear_manage_neuron_followees: bool,

    /// Merge stake, maturity and age from the neuron specified by this option into the neuron being managed.
    #[arg(long)]
    merge_from_neuron: Option<String>,

    /// Merge the percentage (between 1 and 100) of the maturity of a neuron into the current stake.
    #[arg(hide(true), long)]
    merge_maturity: Option<u32>,

    /// Stake a percentage (between 1 and 100) of the maturity of a neuron.
    #[arg(long)]
    stake_maturity: Option<u32>,

    /// Join the Internet Computer's community fund with this neuron's entire stake.
    #[arg(long)]
    join_community_fund: bool,

    /// Leave the Internet Computer's community fund.
    #[arg(long, conflicts_with = "join_community_fund")]
    leave_community_fund: bool,

    /// Defines the topic of a follow rule.
    #[arg(long, requires = "follow_neurons")]
    follow_topic: Option<i32>,

    /// Defines the neuron ids of a follow rule.
    #[arg(long, num_args = .., requires = "follow_topic")]
    follow_neurons: Option<Vec<u64>>,

    /// Vote on proposal(s) (approve by default, or use --reject).
    #[arg(long, num_args = ..)]
    register_vote: Option<Vec<u64>>,

    /// Reject the proposal(s) specified with --register-vote.
    #[arg(long, requires = "register_vote")]
    reject: bool,

    /// Set whether new maturity should be automatically staked.
    #[arg(long, value_enum)]
    auto_stake_maturity: Option<EnableState>,

    #[arg(from_global)]
    ledger: bool,

    /// Set whether the neuron is public or private. This controls whether an
    /// arbitrary principal can view all fields of the neuron (Public), or just
    /// a limited subset (Private).
    #[arg(long)]
    set_visibility: Option<NativeVisibility>,

    /// Refresh the neuron's voting power by reaffirming the current list of followed neurons.
    /// This must be done every so often to avoid neurons diminishing in voting power.
    #[arg(long, alias = "refresh-followers")]
    refresh_following: bool,

    /// Disburse the neuron's maturity to its controller's account.
    #[arg(long)]
    disburse_maturity: bool,

    /// Set the percentage of the neuron's maturity to disburse.
    #[arg(long, value_parser = 1..=100)]
    disburse_maturity_percentage: Option<i64>,

    /// Disburse the neuron's maturity to the specified NNS account.
    #[arg(long)]
    disburse_maturity_to: Option<ParsedNnsAccount>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum NativeVisibility {
    Public = pb::Visibility::Public as isize,
    Private = pb::Visibility::Private as isize,
}

pub fn exec(auth: &AuthInfo, opts: ManageOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    if opts.ledger {
        ensure!(
            !opts.disburse_maturity && opts.disburse_maturity_to.is_none()
            && opts.disburse_maturity_percentage.is_none(),
            "\
Cannot use --ledger with these flags. This version of quill does not support the --disburse-maturity, --disburse-maturity-to, \
or --disburse-maturity-percentage flags with a Ledger device"
        );
    }
    let mut msgs = Vec::new();

    let id = NeuronId {
        id: parse_neuron_id(opts.neuron_id)?,
    };
    let id = Some(NeuronIdOrSubaccount::NeuronId(id));
    if opts.add_hot_key.is_some() {
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::AddHotKey(AddHotKey {
                    new_hot_key: opts.add_hot_key.map(PrincipalId)
                }))
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    };

    if opts.remove_hot_key.is_some() {
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::RemoveHotKey(RemoveHotKey {
                    hot_key_to_remove: opts.remove_hot_key.map(PrincipalId)
                }))
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    };

    if opts.stop_dissolving {
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::StopDissolving(StopDissolving {}))
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    }

    if opts.start_dissolving {
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::StartDissolving(StartDissolving {}))
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    }

    if let Some(additional_dissolve_delay_seconds) = opts.additional_dissolve_delay_seconds {
        let args = Encode!(&ManageNeuron {
            id: None,
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
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    };

    if opts.disburse || opts.disburse_amount.is_some() || opts.disburse_to.is_some() {
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Disburse(Disburse {
                to_account: opts.disburse_to.map(|to| to.into_identifier().into()),
                amount: opts.disburse_amount.map(|amount| Amount {
                    e8s: amount.get_e8s()
                }),
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    };

    if opts.spawn {
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Spawn(Default::default())),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    };

    if let Some(amount) = opts.split {
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Split(Split {
                amount_e8s: amount * 100_000_000
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    };

    if opts.clear_manage_neuron_followees {
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Follow(Follow {
                topic: 1, // Topic::NeuronManagement as i32,
                followees: Vec::new()
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    }

    if let Some(neuron_id) = opts.merge_from_neuron {
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Merge(Merge {
                source_neuron_id: Some(NeuronId {
                    id: parse_neuron_id(neuron_id)?
                }),
            })),
            neuron_id_or_subaccount: id.clone(),
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
            id: None,
            command: Some(Command::StakeMaturity(StakeMaturity {
                percentage_to_stake: Some(percentage),
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    }

    if opts.join_community_fund {
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::JoinCommunityFund(JoinCommunityFund {}))
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    };

    if opts.leave_community_fund {
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::LeaveCommunityFund(LeaveCommunityFund {}))
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    }

    if let Some(proposals) = opts.register_vote {
        for proposal in proposals {
            let args = Encode!(&ManageNeuron {
                id: None,
                command: Some(Command::RegisterVote(RegisterVote {
                    vote: if opts.reject { 2 } else { 1 },
                    proposal: Some(ProposalId { id: proposal }),
                })),
                neuron_id_or_subaccount: id.clone(),
            })?;
            msgs.push(args);
        }
    };

    if let (Some(topic), Some(neuron_ids)) = (opts.follow_topic, opts.follow_neurons) {
        let followees = neuron_ids.into_iter().map(|x| NeuronId { id: x }).collect();
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Follow(Follow {
                topic, // Topic::NeuronManagement as i32,
                followees,
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    }

    if let Some(enable) = opts.auto_stake_maturity {
        let requested_setting_for_auto_stake_maturity = matches!(enable, EnableState::Enabled);
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::ChangeAutoStakeMaturity(
                    ChangeAutoStakeMaturity {
                        requested_setting_for_auto_stake_maturity,
                    }
                ))
            })),
            neuron_id_or_subaccount: id.clone(),
        })?;
        msgs.push(args);
    }

    if let Some(native_visibility) = opts.set_visibility {
        let visibility = Some(native_visibility as i32);
        let set_visibility = SetVisibility { visibility };
        let operation = Some(Operation::from(set_visibility));
        let command = Some(Command::from(Configure { operation }));

        let args = Encode!(&ManageNeuron {
            command,
            neuron_id_or_subaccount: id.clone(),
            id: None,
        })?;

        msgs.push(args);
    }

    if opts.refresh_following {
        let args = Encode!(&ManageNeuron {
            command: Some(Command::RefreshVotingPower(RefreshVotingPower {})),
            neuron_id_or_subaccount: id.clone(),
            id: None,
        })?;
        msgs.push(args);
    }

    if opts.disburse_maturity
        || opts.disburse_maturity_to.is_some()
        || opts.disburse_maturity_percentage.is_some()
    {
        let percentage_to_disburse = opts.disburse_maturity_percentage.unwrap_or(100) as u32;
        let disburse = match opts.disburse_maturity_to {
            Some(ParsedNnsAccount::Original(ident)) => DisburseMaturity {
                percentage_to_disburse,
                to_account: None,
                to_account_identifier: Some(ident.into()),
            },
            Some(ParsedNnsAccount::Icrc1(account)) => DisburseMaturity {
                percentage_to_disburse,
                to_account: Some(account.into()),
                to_account_identifier: None,
            },
            None => DisburseMaturity {
                percentage_to_disburse,
                to_account: None,
                to_account_identifier: None,
            },
        };
        let args = Encode!(&ManageNeuron {
            id: None,
            command: Some(Command::DisburseMaturity(disburse)),
            neuron_id_or_subaccount: id.clone(),
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
