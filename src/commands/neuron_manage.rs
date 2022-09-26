use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo,
};
use anyhow::{anyhow, Context};
use candid::{CandidType, Encode};
use clap::Parser;
use ic_types::Principal;
use ledger_canister::Tokens;

// These constants are copied from src/governance.rs
pub const ONE_DAY_SECONDS: u32 = 24 * 60 * 60;
pub const ONE_YEAR_SECONDS: u32 = (4 * 365 + 1) * ONE_DAY_SECONDS / 4;
pub const ONE_MONTH_SECONDS: u32 = ONE_YEAR_SECONDS / 12;
pub const RANGE_LIMIT: usize = 100;

#[derive(CandidType)]
pub struct IncreaseDissolveDelay {
    pub additional_dissolve_delay_seconds: u32,
}

#[derive(CandidType, Copy, Clone)]
pub struct NeuronId {
    pub id: u64,
}
#[allow(dead_code)]
#[derive(CandidType)]
pub enum NeuronIdOrSubaccount {
    Subaccount(Vec<u8>),
    NeuronId(NeuronId),
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
pub struct JoinCommunityFund {}

#[derive(CandidType)]
pub struct LeaveCommunityFund {}

#[derive(CandidType)]
pub struct ProposalId {
    pub id: u64,
}

#[derive(CandidType)]
pub struct RegisterVote {
    pub vote: i32,
    pub proposal: Option<ProposalId>,
}

#[derive(CandidType)]
pub enum Operation {
    RemoveHotKey(RemoveHotKey),
    StartDissolving(StartDissolving),
    StopDissolving(StopDissolving),
    AddHotKey(AddHotKey),
    IncreaseDissolveDelay(IncreaseDissolveDelay),
    JoinCommunityFund(JoinCommunityFund),
    LeaveCommunityFund(LeaveCommunityFund),
}

#[derive(CandidType)]
pub struct Configure {
    pub operation: Option<Operation>,
}

#[derive(CandidType)]
pub struct AccountIdentifier {
    hash: Vec<u8>,
}
#[derive(CandidType)]
pub struct Disburse {
    pub to_account: Option<AccountIdentifier>,
    pub amount: Option<Tokens>,
}

#[derive(CandidType, Default)]
pub struct Spawn {
    pub new_controller: Option<Principal>,
}

#[derive(CandidType)]
pub struct Split {
    pub amount_e8s: u64,
}

#[derive(CandidType)]
pub struct Merge {
    pub source_neuron_id: NeuronId,
}

#[derive(CandidType)]
pub struct Follow {
    pub topic: i32,
    pub followees: Vec<NeuronId>,
}

#[derive(candid::CandidType)]
pub struct MergeMaturity {
    pub percentage_to_merge: u32,
}

#[derive(CandidType)]
pub enum Command {
    Configure(Configure),
    RegisterVote(RegisterVote),
    Disburse(Disburse),
    Spawn(Spawn),
    Split(Split),
    Follow(Follow),
    Merge(Merge),
    MergeMaturity(MergeMaturity),
}

#[derive(CandidType)]
struct ManageNeuron {
    id: Option<NeuronId>,
    command: Option<Command>,
    neuron_id_or_subaccount: Option<NeuronIdOrSubaccount>,
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
    #[clap(long)]
    merge_maturity: Option<u32>,

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
    register_vote: Option<Vec<String>>,

    /// Reject proposal(s).
    #[clap(long)]
    reject: bool,
}

pub fn exec(auth: &AuthInfo, opts: ManageOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let mut msgs = Vec::new();

    let id = Some(NeuronId {
        id: parse_neuron_id(opts.neuron_id)?,
    });
    if opts.add_hot_key.is_some() {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::AddHotKey(AddHotKey {
                    new_hot_key: opts.add_hot_key
                }))
            })),
            neuron_id_or_subaccount: None,
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
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.stop_dissolving {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::StopDissolving(StopDissolving {}))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    }

    if opts.start_dissolving {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::StartDissolving(StartDissolving {}))
            })),
            neuron_id_or_subaccount: None,
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
            id,
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
            id,
            command: Some(Command::Spawn(Default::default())),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if let Some(amount) = opts.split {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Split(Split {
                amount_e8s: amount * 100_000_000
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.clear_manage_neuron_followees {
        let args = Encode!(&ManageNeuron {
            id,
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
            id,
            command: Some(Command::Merge(Merge {
                source_neuron_id: NeuronId {
                    id: parse_neuron_id(neuron_id)?
                },
            })),
            neuron_id_or_subaccount: None,
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
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.join_community_fund {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::JoinCommunityFund(JoinCommunityFund {}))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.leave_community_fund {
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::LeaveCommunityFund(LeaveCommunityFund {}))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    }

    if let Some(proposals) = opts.register_vote {
        for proposal in proposals {
            let mut proposals = Vec::new();
            if proposal.contains('-') {
                let (mut first, last) = get_range(&proposal)?;
                proposals = (first, last+1).collect();
            } else if let Ok(proposal) = proposal.parse::<u64>() {
                proposals.push(proposal);
            }
            for proposal in proposals {
                let args = Encode!(&ManageNeuron {
                    id,
                    command: Some(Command::RegisterVote(RegisterVote {
                        vote: if opts.reject { 2 } else { 1 },
                        proposal: Some(ProposalId { id: proposal }),
                    })),
                    neuron_id_or_subaccount: None,
                })?;
                msgs.push(args);
            }
        }
    };

    if let (Some(topic), Some(neuron_ids)) = (opts.follow_topic, opts.follow_neurons.as_ref()) {
        let followees = neuron_ids.iter().map(|x| NeuronId { id: *x }).collect();
        let args = Encode!(&ManageNeuron {
            id,
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

    if msgs.is_empty() {
        return Err(anyhow!("No instructions provided"));
    }

    let mut generated = Vec::new();
    for args in msgs {
        generated.push(sign_ingress_with_request_status_query(
            auth,
            governance_canister_id(),
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

// Get the range first..last from a string of the form X-Y
// of the form 1234-5 = 1234..1245, 1234-45 = 1234-1245, etc. where
// the string Y is a new suffix overwriting the end of X.
// Empty ranges are an error as are any points which do not parse as u64 as
// are any ranges not less than RANGE_LIMIT.
// See test_get_range() for additional examples.
fn get_range(range: &str) -> AnyhowResult<(u64, u64)> {
    let pieces: Vec<&str> = range.split('-').collect();
    if pieces.len() == 2 {
        if let Ok(first) = pieces[0].parse::<u64>() {
            let mut last = pieces[0].to_string();
            last.replace_range(
                pieces[0]
                    .chars()
                    .count()
                    .saturating_sub(pieces[1].chars().count())..,
                pieces[1],
            );
            if let Ok(last) = last.parse::<u64>() {
                if last >= first && ((last - first) as usize) < RANGE_LIMIT {
                    return Ok((first, last));
                }
            }
        }
    }
    return Err(anyhow!(
        "Proposal ranges must be of the form START-ENDSUFFIX (e.g. 123-31) and shorter than {}.",
        RANGE_LIMIT
    ));
}

#[test]
fn test_get_range() {
    assert!(get_range("1").is_err());
    assert!(get_range("ABCDE").is_err());
    assert!(get_range("1-A").is_err());
    assert!(get_range("2-1").is_err());
    assert!(get_range("12-10").is_err());
    assert!(get_range("12-1").is_err());
    assert!(get_range("10000-99999").is_err());

    assert_eq!(get_range("1-1").ok(), Some((1, 1)));
    assert_eq!(get_range("1-2").ok(), Some((1, 2)));
    assert_eq!(get_range("1-29").ok(), Some((1, 29)));

    assert_eq!(get_range("10-10").ok(), Some((10, 10)));
    assert_eq!(get_range("10-2").ok(), Some((10, 12)));
    assert_eq!(get_range("10-12").ok(), Some((10, 12)));
    assert_eq!(get_range("10-22").ok(), Some((10, 22)));

    assert_eq!(get_range("777-8").ok(), Some((777, 778)));
    assert_eq!(get_range("777-88").ok(), Some((777, 788)));
    assert_eq!(get_range("777-788").ok(), Some((777, 788)));
    assert_eq!(get_range("777-783").ok(), Some((777, 783)));
    assert_eq!(get_range("999-1001").ok(), Some((999, 1001)));
}
