use std::fmt::Write;

use anyhow::{anyhow, bail, Context};
use bigdecimal::BigDecimal;
use candid::Decode;
use chrono::Utc;
use ic_base_types::CanisterId;
use ic_nns_constants::canister_id_to_nns_canister_name;
use ic_nns_governance::pb::v1::{
    add_or_remove_node_provider::Change,
    claim_or_refresh_neuron_from_account_response::Result as ClaimResult,
    install_code::CanisterInstallMode,
    manage_neuron::{
        configure::Operation, Command as ProposalCommand, NeuronIdOrSubaccount, SetVisibility,
    },
    manage_neuron_response::Command,
    neuron::DissolveState,
    proposal::Action,
    reward_node_provider::{RewardMode, RewardToAccount},
    stop_or_start_canister,
    update_canister_settings::CanisterSettings,
    ClaimOrRefreshNeuronFromAccountResponse, GovernanceError, InstallCode, ListNeuronsResponse,
    ListProposalInfoResponse, ManageNeuronResponse, NeuronInfo, ProposalInfo, StopOrStartCanister,
    Topic, UpdateCanisterSettings, Visibility,
};
use itertools::Itertools;
use sha2::{Digest, Sha256};
use std::fmt::Display;

use crate::lib::{
    e8s_to_tokens,
    format::{format_duration_seconds, format_timestamp_seconds},
    get_default_role, get_idl_string, AnyhowResult,
};

fn hash_sha256(blob: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(blob);
    <[u8; 32]>::from(hasher.finalize())
}

pub fn display_get_neuron_info(blob: &[u8]) -> AnyhowResult<String> {
    let info = Decode!(blob, Result<NeuronInfo, GovernanceError>)?;
    let fmt = match info {
        Ok(info) => {
            let mut fmt = format!(
                "\
Age: {age}
Total stake: {icp} ICP
Voting power: {power}
State: {state:?}
Dissolve delay: {delay}
Created {creation}
",
                age = format_duration_seconds(info.age_seconds),
                icp = e8s_to_tokens(info.stake_e8s.into()),
                power = e8s_to_tokens(info.voting_power.into()),
                state = info.state(),
                delay = format_duration_seconds(info.dissolve_delay_seconds),
                creation = format_timestamp_seconds(info.created_timestamp_seconds)
            );
            if let Some(cf) = info.joined_community_fund_timestamp_seconds {
                writeln!(
                    fmt,
                    "Member of the community fund since {}",
                    format_timestamp_seconds(cf)
                )?;
            }
            if let Some(known) = info.known_neuron_data {
                writeln!(fmt, "Known neuron: \"{}\"", known.name)?;
                if let Some(desc) = known.description {
                    writeln!(fmt, "Description: \"{desc}\"")?;
                }
            }
            write!(
                fmt,
                "Accurate as of {}",
                format_timestamp_seconds(info.retrieved_at_timestamp_seconds)
            )?;
            fmt
        }
        Err(e) => display_governance_error(e),
    };
    Ok(fmt)
}

pub fn display_list_neurons(blob: &[u8]) -> AnyhowResult<String> {
    let now_seconds = u64::try_from(Utc::now().timestamp()).unwrap();
    let neurons = Decode!(blob, ListNeuronsResponse)?;
    let mut fmt = String::new();
    for neuron in neurons.full_neurons {
        let neuron_type = neuron.neuron_type();
        if let Some(id) = neuron.id {
            writeln!(fmt, "Neuron {}", id.id)?;
        } else {
            writeln!(fmt, "Neuron (unknown id)")?;
        }
        if neuron.aging_since_timestamp_seconds != u64::MAX {
            writeln!(
                fmt,
                "Aging since: {}",
                format_duration_seconds(neuron.aging_since_timestamp_seconds)
            )?;
        }

        writeln!(
            fmt,
            "Staked ICP: {} ICP",
            e8s_to_tokens(neuron.cached_neuron_stake_e8s.into())
        )?;
        if let Some(staked_maturity) = neuron.staked_maturity_e8s_equivalent {
            writeln!(
                fmt,
                "Staked maturity: {}",
                e8s_to_tokens(staked_maturity.into())
            )?;
        }
        if neuron.auto_stake_maturity() {
            writeln!(fmt, "Auto staking maturity: Yes")?;
        }
        if let Some(timestamp) = neuron.spawn_at_timestamp_seconds {
            writeln!(
                fmt,
                "Spawning maturity as ICP at: {}",
                format_timestamp_seconds(timestamp)
            )?;
        }
        writeln!(fmt, "State: {:?}", neuron.state(now_seconds))?;
        if let Some(state) = neuron.dissolve_state {
            match state {
                DissolveState::DissolveDelaySeconds(s) => {
                    writeln!(fmt, "Dissolve delay: {}", format_duration_seconds(s))?
                }
                DissolveState::WhenDissolvedTimestampSeconds(s) => {
                    writeln!(fmt, "Dissolve timestamp: {}", format_timestamp_seconds(s))?
                }
            }
        }
        writeln!(
            fmt,
            "Created {}",
            format_timestamp_seconds(neuron.created_timestamp_seconds)
        )?;
        if let Some(cf) = neuron.joined_community_fund_timestamp_seconds {
            writeln!(
                fmt,
                "Member of the community fund since {}",
                format_timestamp_seconds(cf)
            )?;
        }
        if let Some(known) = neuron.known_neuron_data {
            writeln!(fmt, "Known neuron: \"{}\"", known.name)?;
            if let Some(desc) = known.description {
                writeln!(fmt, "Description: \"{desc}\"")?;
            }
        }
        if let Some(controller) = neuron.controller {
            writeln!(fmt, "Controller: {controller}")?;
        }
        if !neuron.hot_keys.is_empty() {
            writeln!(
                fmt,
                "Hot keys: {}",
                neuron.hot_keys.into_iter().format(", ")
            )?;
        }
        if neuron.neuron_type.is_some() {
            writeln!(fmt, "Neuron type: {:?}", neuron_type)?;
        }
        if neuron.kyc_verified {
            writeln!(fmt, "KYC verified: Yes")?;
        }
        if neuron.not_for_profit {
            writeln!(fmt, "Not-for-profit: Yes")?;
        }
        if !neuron.recent_ballots.is_empty() {
            writeln!(fmt, "Recent votes: {}", neuron.recent_ballots.len())?;
        }
        if !neuron.followees.is_empty() {
            if neuron.followees.len() < 4 {
                writeln!(
                    fmt,
                    "Followees: {}",
                    neuron
                        .followees
                        .into_iter()
                        .format_with(", ", |(topic, followees), f| {
                            let topic = Topic::try_from(topic).unwrap_or(Topic::Unspecified);
                            if followees.followees.len() < 4 {
                                f(&format_args!(
                                    "neurons {} ({topic:?})",
                                    followees.followees.into_iter().map(|id| id.id).format(", ")
                                ))
                            } else {
                                f(&format_args!(
                                    "{} followees ({topic:?})",
                                    followees.followees.len(),
                                ))
                            }
                        })
                )?;
            } else {
                writeln!(
                    fmt,
                    "Followees: {}",
                    neuron
                        .followees
                        .into_iter()
                        .map(|followees| followees.1.followees.len())
                        .sum::<usize>()
                )?;
            }
            fmt.push('\n');
        }
    }
    Ok(fmt)
}

pub fn display_manage_neuron(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, ManageNeuronResponse)?;
    let cmd = response.command.context("command was null")?;
    let fmt = match cmd {
        Command::Error(e) => display_governance_error(e),
        Command::Configure(_) => "Neuron successfully configured".to_string(),
        Command::RegisterVote(_) => "Successfully voted".to_string(),
        Command::Follow(_) => "Successfully set following relationship".to_string(),
        Command::Spawn(c) => {
            if let Some(id) = c.created_neuron_id {
                format!("Maturity successfully spawned to new neuron {}", id.id)
            } else {
                "Maturity successfully spawned to unknown new neuron".to_string()
            }
        }
        Command::Split(c) => {
            if let Some(id) = c.created_neuron_id {
                format!("Neuron successfully split off to new neuron {}", id.id)
            } else {
                "Neuron successfully split off to unknown new neuron".to_string()
            }
        }
        Command::ClaimOrRefresh(c) => {
            if let Some(id) = c.refreshed_neuron_id {
                format!("Successfully staked ICP in neuron {}", id.id)
            } else {
                "Successfully staked ICP in unknown neuron".to_string()
            }
        }
        Command::Merge(c) => {
            let mut fmt = "Successfully merged ".to_string();
            if let Some(source) = c.source_neuron {
                if let Some(id) = source.id {
                    write!(fmt, "neuron {}", id.id)?;
                } else {
                    write!(fmt, "neuron with account {}", hex::encode(source.account))?;
                }
            } else {
                write!(fmt, "unknown neuron")?;
            }
            write!(fmt, " into ")?;
            if let Some(target) = c.target_neuron {
                if let Some(id) = target.id {
                    write!(fmt, "neuron {}", id.id)?;
                } else {
                    write!(fmt, "neuron with account {}", hex::encode(target.account))?;
                }
            } else {
                write!(fmt, "unknown neuron")?;
            }
            fmt
        }
        Command::DisburseToNeuron(c) => {
            if let Some(id) = c.created_neuron_id {
                format!("Successfully disbursed into new neuron {}", id.id)
            } else {
                "Successfully disbursed into unknown new neuron".to_string()
            }
        }
        Command::MakeProposal(c) => {
            if let Some(id) = c.proposal_id {
                format!("Successfully created new proposal with ID {id}\nhttps://dashboard.internetcomputer.org/proposal/{id}", id = id.id)
            } else {
                "Successfully created new proposal (unknown ID)".to_string()
            }
        }
        Command::StakeMaturity(c) => format!(
            "Successfully staked maturity ({staked} staked maturity total, {remaining} unstaked)",
            staked = e8s_to_tokens(c.staked_maturity_e8s.into()),
            remaining = e8s_to_tokens(c.maturity_e8s.into()),
        ),
        Command::MergeMaturity(c) => format!(
            "Successfully merged {merged} maturity (total stake now {total})",
            merged = e8s_to_tokens(c.merged_maturity_e8s.into()),
            total = e8s_to_tokens(c.new_stake_e8s.into())
        ),
        Command::Disburse(c) => format!(
            "Successfully disbursed ICP at block index {}",
            c.transfer_block_height
        ),
    };
    Ok(fmt)
}

pub fn display_update_node_provider(blob: &[u8]) -> AnyhowResult<String> {
    let res = Decode!(blob, Result<(), GovernanceError>)?;
    let fmt = match res {
        Ok(()) => "Successfully updated node provider".to_string(),
        Err(e) => display_governance_error(e),
    };
    Ok(fmt)
}

pub fn display_list_proposals(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, ListProposalInfoResponse)?;
    let mut fmt = String::new();
    for proposal_info in response.proposal_info {
        write!(fmt, "{}\n\n", display_proposal_info(proposal_info)?)?;
    }
    Ok(fmt)
}

pub fn display_get_proposal(blob: &[u8]) -> AnyhowResult<String> {
    let opt = Decode!(blob, Option<ProposalInfo>)?;
    let fmt = match opt {
        Some(proposal) => display_proposal_info(proposal)?,
        None => "No proposal with that ID was found.".to_string(),
    };
    Ok(fmt)
}

fn display_proposal_info(proposal_info: ProposalInfo) -> AnyhowResult<String> {
    let mut fmt = String::new();
    let topic = proposal_info.topic();
    let status = proposal_info.status();
    let reward_status = proposal_info.reward_status();
    if let Some(proposal) = proposal_info.proposal {
        if let Some(title) = proposal.title {
            writeln!(fmt, "\"{}\" ({:?})", title, topic)?;
        } else {
            writeln!(fmt, "Untitled proposal ({:?})", topic)?;
        }
        if !proposal.summary.is_empty() {
            writeln!(fmt, "Summary: \"{}\"", proposal.summary)?;
        }
        if !proposal.url.is_empty() {
            writeln!(fmt, "URL: {}", proposal.url)?;
        }
        if let Some(action) = proposal.action {
            fmt.push_str("Proposed action: ");
            match action {
                Action::RegisterKnownNeuron(a) => {
                    fmt.push_str("Register known neuron");
                    if let Some(id) = a.id {
                        write!(fmt, " {}", id.id)?;
                    }
                    if let Some(data) = a.known_neuron_data {
                        write!(fmt, " as {}.", data.name)?;
                        if let Some(desc) = data.description {
                            write!(fmt, " \"{desc}\"")?;
                        }
                    }
                    fmt.push('\n');
                }
                Action::ApproveGenesisKyc(a) => writeln!(
                    fmt,
                    "Approve principals {} for Genesis KYC",
                    a.principals.iter().format(", ")
                )?,
                Action::AddOrRemoveNodeProvider(a) => {
                    let (change, provider) =
                        match a.change.context("node provider change was null")? {
                            Change::ToAdd(provider) => ("Add", provider),
                            Change::ToRemove(provider) => ("Remove", provider),
                        };
                    write!(fmt, "{change} node provider")?;
                    if let Some(id) = provider.id {
                        write!(fmt, " {id}")?;
                    }
                    if let Some(reward) = provider.reward_account {
                        write!(fmt, " with reward account {}", hex::encode(reward.hash))?;
                    }
                    fmt.push('\n');
                }
                Action::CreateServiceNervousSystem(_)
                | Action::OpenSnsTokenSwap(_)
                | Action::SetSnsTokenSwapOpenTimeWindow(_) => {
                    bail!("SNS proposals currently unsupported") //todo
                }
                Action::ExecuteNnsFunction(a) => {
                    let function = a.nns_function();
                    writeln!(fmt, "Execute NNS function {:?}", function)?;
                    if a.payload.starts_with(b"DIDL") {
                        let (canister_id, method) = function
                            .canister_and_function()
                            .map_err(|e| anyhow!(e.error_message))?;
                        if let Ok(idl) = get_idl_string(
                            &a.payload,
                            canister_id.into(),
                            get_default_role(canister_id.into()).unwrap_or_default(),
                            method,
                            "args",
                        ) {
                            writeln!(fmt, "Payload: {idl}")?;
                        } else {
                            writeln!(fmt, "Payload: {}", hex::encode(a.payload))?;
                        }
                    } else {
                        writeln!(fmt, "Payload: {}", hex::encode(a.payload))?;
                    }
                }
                Action::Motion(a) => writeln!(fmt, "\"{}\" (motion)", a.motion_text)?,
                Action::ManageNetworkEconomics(a) => {
                    writeln!(fmt, "Update network economics")?;
                    if a.max_proposals_to_keep_per_topic != 0 {
                        writeln!(
                            fmt,
                            "New maximum proposals to keep, per topic: {}",
                            a.max_proposals_to_keep_per_topic
                        )?;
                    }
                    if a.maximum_node_provider_rewards_e8s != 0 {
                        writeln!(
                            fmt,
                            "New maximum node provider reward: {} ICP",
                            e8s_to_tokens(a.maximum_node_provider_rewards_e8s.into())
                        )?;
                    }
                    if a.minimum_icp_xdr_rate != 0 {
                        writeln!(
                            fmt,
                            "New minimum ICP/SDR conversion rate: 1 ICP <> {} XDR",
                            BigDecimal::new(a.minimum_icp_xdr_rate.into(), 2)
                        )?;
                    }
                    if a.neuron_management_fee_per_proposal_e8s != 0 {
                        writeln!(
                            fmt,
                            "New cost for making \"manage neuron\" proposals: {} ICP",
                            e8s_to_tokens(a.neuron_management_fee_per_proposal_e8s.into())
                        )?;
                    }
                    if a.neuron_minimum_stake_e8s != 0 {
                        writeln!(
                            fmt,
                            "New minimum stake for neurons: {} ICP",
                            e8s_to_tokens(a.neuron_minimum_stake_e8s.into())
                        )?;
                    }
                    if a.neuron_spawn_dissolve_delay_seconds != 0 {
                        writeln!(
                            fmt,
                            "New dissolve delay for spawned-maturity neurons: {}",
                            format_duration_seconds(a.neuron_spawn_dissolve_delay_seconds)
                        )?;
                    }
                    if a.transaction_fee_e8s != 0 {
                        writeln!(
                            fmt,
                            "New ICP transaction fee: {} ICP",
                            e8s_to_tokens(a.transaction_fee_e8s.into())
                        )?;
                    }
                    if a.reject_cost_e8s != 0 {
                        writeln!(
                            fmt,
                            "New proposal rejection cost: {} ICP",
                            e8s_to_tokens(a.reject_cost_e8s.into())
                        )?;
                    }
                    if let Some(extra) = a.neurons_fund_economics {
                        if let Some(max) = extra.maximum_icp_xdr_rate {
                            writeln!(
                                fmt,
                                "New maximum ICP/SDR conversion rate for the community fund: {}%",
                                BigDecimal::new(max.basis_points().into(), 2)
                            )?;
                        }
                        if let Some(min) = extra.minimum_icp_xdr_rate {
                            writeln!(
                                fmt,
                                "New minimum ICP/SDR conversion rate for the community fund: {}%",
                                BigDecimal::new(min.basis_points().into(), 2)
                            )?;
                        }
                        if let Some(max) =
                            extra.max_theoretical_neurons_fund_participation_amount_xdr
                        {
                            writeln!(fmt, "New maximum theoretical community fund participation amount: {} XDR", max.human_readable())?;
                        }
                        if let Some(extra) = extra.neurons_fund_matched_funding_curve_coefficients {
                            if let Some(threshold) = extra.contribution_threshold_xdr {
                                writeln!(fmt, "New SNS participation threshold to receive any community fund contributions: {} XDR", threshold.human_readable())?;
                            }
                            if let Some(milestone) = extra.one_third_participation_milestone_xdr {
                                writeln!(fmt, "New SNS participation milestone to receive 1/3 community fund contribution: {} XDR", milestone.human_readable())?;
                            }
                            if let Some(milestone) = extra.full_participation_milestone_xdr {
                                writeln!(fmt, "New SNS participation milestone to receive full community fund contribution: {} XDR", milestone.human_readable())?;
                            }
                        }
                    }
                }
                Action::RewardNodeProvider(a) => {
                    fmt.push_str("Reward node provider");
                    if let Some(provider) = a.node_provider {
                        if let Some(id) = provider.id {
                            write!(fmt, " {id}")?;
                        }
                        if let Some(account) = provider.reward_account {
                            write!(fmt, " (reward account {})", hex::encode(account.hash))?;
                        }
                    }
                    write!(fmt, " with {} ICP", e8s_to_tokens(a.amount_e8s.into()))?;
                    match a.reward_mode {
                        Some(RewardMode::RewardToAccount(RewardToAccount {
                            to_account: Some(to),
                        })) => write!(fmt, " to account {}", hex::encode(to.hash))?,
                        Some(RewardMode::RewardToNeuron(n)) => write!(
                            fmt,
                            " to neuron with {} dissolve delay",
                            format_duration_seconds(n.dissolve_delay_seconds)
                        )?,
                        _ => {}
                    }
                    fmt.push('\n');
                }
                Action::RewardNodeProviders(a) => {
                    if a.use_registry_derived_rewards() {
                        writeln!(
                            fmt,
                            "Reward node providers {}",
                            a.rewards
                                .into_iter()
                                .filter_map(|r| r.node_provider.and_then(|p| p.id))
                                .format(", ")
                        )?;
                    } else {
                        fmt.push_str("Reward node providers\n");
                        for reward in a.rewards {
                            if let Some(provider) = reward.node_provider {
                                if let Some(id) = provider.id {
                                    write!(fmt, "{id}")?;
                                    if let Some(account) = provider.reward_account {
                                        write!(
                                            fmt,
                                            " (reward account {})",
                                            hex::encode(account.hash)
                                        )?;
                                    }
                                    write!(
                                        fmt,
                                        " with {} ICP",
                                        e8s_to_tokens(reward.amount_e8s.into())
                                    )?;
                                    match reward.reward_mode {
                                        Some(RewardMode::RewardToAccount(RewardToAccount {
                                            to_account: Some(to),
                                        })) => write!(fmt, " to account {}", hex::encode(to.hash))?,
                                        Some(RewardMode::RewardToNeuron(n)) => write!(
                                            fmt,
                                            " to neuron with {} dissolve delay",
                                            format_duration_seconds(n.dissolve_delay_seconds)
                                        )?,
                                        _ => {}
                                    }
                                    fmt.push('\n');
                                }
                            }
                        }
                    }
                }
                Action::SetDefaultFollowees(a) => {
                    if a.default_followees.len() == 1 {
                        let (topic, followees) = a.default_followees.into_iter().next().unwrap();
                        writeln!(
                            fmt,
                            "Set default followees for {topic:?} to {followees}",
                            topic = Topic::try_from(topic).unwrap_or_default(),
                            followees = followees.followees.iter().map(|id| id.id).format(", ")
                        )?;
                    } else {
                        writeln!(fmt, "Set default followees")?;
                        for (topic, followees) in a.default_followees {
                            writeln!(
                                fmt,
                                "For {topic:?}: {followees}",
                                topic = Topic::try_from(topic).unwrap_or_default(),
                                followees = followees.followees.iter().map(|id| id.id).format(", ")
                            )?;
                        }
                    }
                }
                Action::InstallCode(install_code) => {
                    let InstallCode {
                        canister_id,
                        install_mode,
                        wasm_module,
                        arg,
                        skip_stopping_before_installing,
                    } = install_code;

                    let canister_id = canister_id
                        .map(CanisterId::unchecked_from_principal)
                        .map(canister_id_to_nns_canister_name);

                    let install_mode = match install_mode {
                        None => None,
                        Some(ok) => Some(
                            CanisterInstallMode::try_from(ok)
                                .context("converting from i32 to CanisterInstallMode")?,
                        ),
                    };

                    let wasm_module =
                        wasm_module.map(|wasm_module| hex::encode(hash_sha256(&wasm_module)));

                    let skip_stopping_before_installing =
                        if skip_stopping_before_installing.unwrap_or_default() {
                            " (Warning: canister will not be stopped before installing new WASM)"
                        } else {
                            ""
                        };

                    let arg = match arg {
                        None => "None".to_string(),
                        Some(ok) => hex::encode(ok),
                    };
                    // TODO Do something like didc decode --defs $DEFS[canister_id] $arg

                    writeln!(
                        fmt,
                        "{:?} {:?} to WASM with SHA256 = {:?}{}. arg: {}",
                        install_mode,
                        canister_id,
                        wasm_module,
                        skip_stopping_before_installing,
                        arg,
                    )?;
                }
                Action::StopOrStartCanister(stop_or_start_canister) => {
                    let StopOrStartCanister {
                        canister_id,
                        action,
                    } = stop_or_start_canister;

                    let canister_id = canister_id
                        .map(CanisterId::unchecked_from_principal)
                        .map(canister_id_to_nns_canister_name);

                    let action = match action {
                        None => None,
                        Some(ok) => Some(
                            stop_or_start_canister::CanisterAction::try_from(ok)
                                .context("converting from i32 to CanisterAction")?,
                        ),
                    };

                    writeln!(fmt, "{:?} {:?}", action, canister_id,)?;
                }
                Action::UpdateCanisterSettings(update_canister_settings) => {
                    let UpdateCanisterSettings {
                        canister_id,
                        settings,
                    } = update_canister_settings;

                    let canister_id = canister_id.map(|canister_id| {
                        canister_id_to_nns_canister_name(CanisterId::unchecked_from_principal(
                            canister_id,
                        ))
                    });

                    let settings = settings.map(display_canister_settings);

                    writeln!(
                        fmt,
                        "Update settings of {:?} to {:?}",
                        canister_id, settings,
                    )?;
                }
                Action::ManageNeuron(a) => {
                    let neuron = a
                        .get_neuron_id_or_subaccount()
                        .map_err(|e| anyhow!(e.error_message))?
                        .context("neuron ID was null")?;
                    let neuron = display_neuron_id(neuron);
                    match a.command.context("command was null")? {
                        ProposalCommand::ClaimOrRefresh(_) => {
                            writeln!(fmt, "Refresh the stake of neuron {neuron}")?;
                        }
                        ProposalCommand::Disburse(c) => {
                            if let Some(amount) = c.amount {
                                write!(
                                    fmt,
                                    "Disburse {icp} ICP from neuron {neuron}",
                                    icp = e8s_to_tokens(amount.e8s.into())
                                )?;
                            } else {
                                write!(fmt, "Disburse neuron {neuron}")?;
                            }
                            if let Some(to) = c.to_account {
                                write!(fmt, " to account {}", hex::encode(to.hash))?;
                            }
                            fmt.push('\n');
                        }
                        ProposalCommand::DisburseToNeuron(c) => {
                            write!(
                                fmt,
                                "Disburse {icp} ICP from neuron {neuron} to a new{verified} neuron",
                                icp = e8s_to_tokens(c.amount_e8s.into()),
                                verified = if c.kyc_verified { " KYC verified" } else { "" },
                            )?;
                            if let Some(controller) = c.new_controller {
                                write!(fmt, " owned by {controller}")?;
                            }
                            writeln!(
                                fmt,
                                " with dissolve delay {}",
                                format_timestamp_seconds(c.dissolve_delay_seconds)
                            )?;
                        }
                        ProposalCommand::Follow(c) => {
                            writeln!(fmt, "Configure neuron {neuron} to follow {ids} for proposals of type {topic:?}", topic = c.topic(), ids = c.followees.iter().map(|id| id.id).format(", "))?;
                        }
                        ProposalCommand::MakeProposal(_) => {
                            bail!("nested proposals not supported")
                        }
                        ProposalCommand::Merge(c) => {
                            if let Some(source) = c.source_neuron_id {
                                writeln!(
                                    fmt,
                                    "Merge neuron {source} into neuron {neuron}",
                                    source = source.id
                                )?;
                            } else {
                                writeln!(fmt, "Merge neuron {neuron}")?;
                            }
                        }
                        ProposalCommand::MergeMaturity(c) => writeln!(
                            fmt,
                            "Merge {percentage}% of maturity into the stake of neuron {neuron}",
                            percentage = c.percentage_to_merge
                        )?,
                        ProposalCommand::RegisterVote(c) => {
                            if let Some(proposal) = c.proposal {
                                writeln!(
                                    fmt,
                                    "Vote {yn:?} on proposal {proposal} from neuron {neuron}",
                                    proposal = proposal.id,
                                    yn = c.vote()
                                )?
                            } else {
                                writeln!(fmt, "Vote {yn:?} from neuron {neuron}", yn = c.vote())?;
                            }
                        }
                        ProposalCommand::Spawn(c) => {
                            if let Some(controller) = c.new_controller {
                                writeln!(fmt, "Spawn {percentage}% of the maturity of neuron {neuron} to {controller}", percentage = c.percentage_to_spawn())?;
                            } else {
                                writeln!(fmt, "Spawn {percentage}% of the maturity of neuron {neuron} to its owner", percentage = c.percentage_to_spawn())?;
                            }
                        }
                        ProposalCommand::Split(c) => writeln!(
                            fmt,
                            "Split off {icp} ICP from neuron {neuron} as a new neuron",
                            icp = e8s_to_tokens(c.amount_e8s.into())
                        )?,
                        ProposalCommand::StakeMaturity(c) => writeln!(
                            fmt,
                            "Stake {percentage}% of the maturity of neuron {neuron}",
                            percentage = c.percentage_to_stake()
                        )?,
                        ProposalCommand::Configure(c) => {
                            match c.operation.context("operation was null")? {
                                Operation::AddHotKey(o) => {
                                    if let Some(key) = o.new_hot_key {
                                        writeln!(fmt, "Add hot key {key} to neuron {neuron}")?
                                    } else {
                                        writeln!(fmt, "Add hot key to neuron {neuron}")?
                                    }
                                }
                                Operation::RemoveHotKey(o) => {
                                    if let Some(key) = o.hot_key_to_remove {
                                        writeln!(fmt, "Remove hot key {key} from neuron {neuron}")?
                                    } else {
                                        writeln!(fmt, "Remove hot key from neuron {neuron}")?
                                    }
                                }
                                Operation::ChangeAutoStakeMaturity(o) => writeln!(
                                    fmt,
                                    "{op} auto-staking maturity for neuron {neuron}",
                                    op = if o.requested_setting_for_auto_stake_maturity {
                                        "Enable"
                                    } else {
                                        "Disable"
                                    }
                                )?,
                                Operation::IncreaseDissolveDelay(o) => writeln!(
                                    fmt,
                                    "Increase dissolve delay for neuron {neuron} by {dur}",
                                    dur = format_duration_seconds(
                                        o.additional_dissolve_delay_seconds.into()
                                    )
                                )?,
                                Operation::SetDissolveTimestamp(o) => writeln!(
                                    fmt,
                                    "Set dissolve timestamp for neuron {neuron} to {time}",
                                    time = format_timestamp_seconds(o.dissolve_timestamp_seconds)
                                )?,
                                Operation::JoinCommunityFund(_) => {
                                    writeln!(fmt, "Add neuron {neuron} to the community fund")?
                                }
                                Operation::LeaveCommunityFund(_) => {
                                    writeln!(fmt, "Remove neuron {neuron} from the community fund")?
                                }
                                Operation::StartDissolving(_) => {
                                    writeln!(fmt, "Start dissolving neuron {neuron}")?
                                }
                                Operation::StopDissolving(_) => {
                                    writeln!(fmt, "Stop dissolving neuron {neuron}")?
                                }
                                Operation::SetVisibility(set_visibility) => {
                                    let SetVisibility { visibility } = set_visibility;

                                    let visibility = visibility.map(|visibility| {
                                        match Visibility::try_from(visibility) {
                                            Ok(ok) => format!("{:?}", ok),
                                            Err(err) => format!(
                                                "unknown visibility {visibility} ({err})???"
                                            ),
                                        }
                                    });

                                    writeln!(fmt, "Set visibility of {neuron} to {visibility:?}")?
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        writeln!(fmt, "Unknown proposal ({:?})", proposal_info.topic())?;
    }
    if let Some(id) = proposal_info.id {
        writeln!(fmt, "Proposal ID: {}", id.id)?;
    }
    if let Some(proposer) = proposal_info.proposer {
        writeln!(
            fmt,
            "Created at {} by neuron {}",
            format_timestamp_seconds(proposal_info.proposal_timestamp_seconds),
            proposer.id,
        )?;
    } else {
        writeln!(
            fmt,
            "Created at {}",
            format_timestamp_seconds(proposal_info.proposal_timestamp_seconds)
        )?;
    }
    writeln!(fmt, "Status: {status:?}, reward status: {reward_status:?}")?;
    if let Some(reason) = proposal_info.failure_reason {
        writeln!(fmt, "Failure reason: {}", reason.error_message)?;
    }
    if proposal_info.decided_timestamp_seconds != 0 {
        writeln!(
            fmt,
            "Decided at {}",
            format_timestamp_seconds(proposal_info.decided_timestamp_seconds)
        )?;
    }
    if proposal_info.failed_timestamp_seconds != 0 {
        writeln!(
            fmt,
            "Failed at {}",
            format_timestamp_seconds(proposal_info.failed_timestamp_seconds)
        )?;
    }
    if proposal_info.executed_timestamp_seconds != 0 {
        writeln!(
            fmt,
            "Executed at {}",
            format_timestamp_seconds(proposal_info.executed_timestamp_seconds)
        )?;
    }
    if let Some(deadline) = proposal_info.deadline_timestamp_seconds {
        writeln!(fmt, "Deadline: {}", format_timestamp_seconds(deadline))?;
    }
    if proposal_info.reject_cost_e8s != 0 {
        writeln!(
            fmt,
            "Rejection cost: {} ICP",
            e8s_to_tokens(proposal_info.reject_cost_e8s.into())
        )?;
    }
    if let Some(tally) = proposal_info.latest_tally {
        let y = e8s_to_tokens(tally.yes.into());
        let n = e8s_to_tokens(tally.no.into());
        let total = e8s_to_tokens(tally.total.into());
        writeln!(
            fmt,
            "Current tally: Y {y} ({y_percent}%), N {n} ({n_percent}%) as of {timestamp}",
            y_percent = (y.clone() / total.clone() * 100_u8).round(2),
            n_percent = (n.clone() / total * 100_u8).round(2),
            timestamp = format_timestamp_seconds(tally.timestamp_seconds),
        )?;
    }
    if proposal_info.reward_event_round != 0 {
        writeln!(
            fmt,
            "Reward event round: {}",
            proposal_info.reward_event_round
        )?;
    }
    fmt.truncate(fmt.trim_end().len());
    Ok(fmt)
}

pub fn display_neuron_ids(blob: &[u8]) -> AnyhowResult<String> {
    let ids = Decode!(blob, Vec<u64>)?;
    let fmt = ids.into_iter().format(", ");
    Ok(format!("Neurons: {fmt}"))
}

pub fn display_claim_gtc_neurons(blob: &[u8]) -> AnyhowResult<String> {
    let res = Decode!(blob, Result<(), GovernanceError>)?;
    let fmt = match res {
        Ok(()) => "Successfully claimed Genesis neurons".to_string(),
        Err(e) => display_governance_error(e),
    };
    Ok(fmt)
}

pub fn display_claim_or_refresh_neuron_from_account(blob: &[u8]) -> AnyhowResult<String> {
    let res = Decode!(blob, ClaimOrRefreshNeuronFromAccountResponse)?;
    let fmt = if let Some(res) = res.result {
        match res {
            ClaimResult::NeuronId(id) => format!("Successfully staked ICP in neuron {}", id.id),
            ClaimResult::Error(e) => display_governance_error(e),
        }
    } else {
        "Unknown result of call".to_string()
    };
    Ok(fmt)
}

fn display_neuron_id(id: NeuronIdOrSubaccount) -> String {
    match id {
        NeuronIdOrSubaccount::NeuronId(i) => format!("{}", i.id),
        NeuronIdOrSubaccount::Subaccount(s) => {
            format!("with subaccount {}", hex::encode(s))
        }
    }
}

pub fn display_governance_error(err: GovernanceError) -> String {
    format!("NNS error: {}", err.error_message)
}

fn display_canister_settings(settings: CanisterSettings) -> String {
    let CanisterSettings {
        controllers,
        compute_allocation,
        memory_allocation,
        freezing_threshold,
        log_visibility,
        wasm_memory_limit,
    } = settings;

    let mut chunks = vec![];

    if let Some(controllers) = controllers {
        let controllers = controllers
            .controllers
            .into_iter()
            .map(CanisterId::unchecked_from_principal)
            .map(canister_id_to_nns_canister_name)
            .join(", ");
        chunks.push(format!("controllers = {}", controllers,));
    }

    fn display_set_field<T: Display>(name: &str, value: Option<T>, chunks: &mut Vec<String>) {
        let Some(value) = value else {
            return;
        };

        chunks.push(format!("{} = {}", name, value));
    }

    display_set_field("compute_allocation", compute_allocation, &mut chunks);
    display_set_field("memory_allocation", memory_allocation, &mut chunks);
    display_set_field("freezing_threshold", freezing_threshold, &mut chunks);
    display_set_field("log_visibility", log_visibility, &mut chunks);
    display_set_field("wasm_memory_limit", wasm_memory_limit, &mut chunks);

    chunks.join("; ")
}
