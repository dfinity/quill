use std::{collections::HashMap, fmt::Write};

use anyhow::{anyhow, bail, Context};
use askama::Template;
use bigdecimal::BigDecimal;
use candid::{Decode, Nat, Principal};
use ic_base_types::CanisterId;
use ic_nns_constants::canister_id_to_nns_canister_name;
use ic_nns_governance::{
    pb::v1::{
        add_or_remove_node_provider::Change,
        claim_or_refresh_neuron_from_account_response::Result as ClaimResult,
        install_code::CanisterInstallMode,
        manage_neuron::{configure::Operation, Command as ProposalCommand, NeuronIdOrSubaccount},
        manage_neuron_response::Command,
        neuron::DissolveState,
        proposal::Action,
        reward_node_provider::{RewardMode, RewardToAccount},
        stop_or_start_canister::CanisterAction,
        update_canister_settings::CanisterSettings,
        ClaimOrRefreshNeuronFromAccountResponse, GovernanceError, KnownNeuronData,
        ListNeuronsResponse, ListProposalInfoResponse, ManageNeuronResponse, NeuronInfo,
        NeuronState, NeuronType, ProposalInfo, Topic, Visibility,
    },
    proposals::call_canister::CallCanister,
};
use indicatif::HumanBytes;
use itertools::Itertools;
use sha2::{Digest, Sha256};

use crate::lib::{
    e8s_to_tokens,
    format::{filters, format_duration_seconds, format_t_cycles, format_timestamp_seconds},
    get_default_role, get_idl_string, AnyhowResult,
};

pub fn display_get_neuron_info(blob: &[u8]) -> AnyhowResult<String> {
    let info = Decode!(blob, Result<NeuronInfo, GovernanceError>)?;
    #[derive(Template)]
    #[template(path = "nns/min_neuron_info.txt")]
    struct GetNeuronInfo {
        age_seconds: u64,
        stake: Nat,
        deciding: Nat,
        potential: Nat,
        last_refreshed_seconds: u64,
        state: NeuronState,
        dissolve_delay_seconds: u64,
        created_seconds: u64,
        community_fund_seconds: Option<u64>,
        known_neuron_data: Option<KnownNeuronData>,
        visibility: Option<Visibility>,
        retrieved_seconds: u64,
    }
    let fmt = match info {
        Ok(info) => GetNeuronInfo {
            age_seconds: info.age_seconds,
            stake: info.stake_e8s.into(),
            deciding: info.deciding_voting_power().into(),
            potential: info.potential_voting_power().into(),
            last_refreshed_seconds: info.voting_power_refreshed_timestamp_seconds(),
            state: info.state(),
            visibility: info.visibility.map(|_| info.visibility()),
            dissolve_delay_seconds: info.dissolve_delay_seconds,
            created_seconds: info.created_timestamp_seconds,
            community_fund_seconds: info.joined_community_fund_timestamp_seconds,
            known_neuron_data: info.known_neuron_data,
            retrieved_seconds: info.retrieved_at_timestamp_seconds,
        }
        .render()?,

        Err(e) => display_governance_error(e),
    };
    Ok(fmt)
}

pub fn display_list_neurons(blob: &[u8]) -> AnyhowResult<String> {
    use DissolveState::*;
    let neurons = Decode!(blob, ListNeuronsResponse)?;
    #[derive(Template)]
    #[template(path = "nns/full_neuron_info.txt")]
    struct FullNeuron {
        id: u64,
        aging_seconds: u64,
        staked_icp_e8s: Nat,
        staked_maturity: Option<Nat>,
        auto_stake_maturity: bool,
        deciding: Nat,
        potential: Nat,
        last_refreshed_seconds: u64,
        spawn_at_seconds: Option<u64>,
        dissolve_delay: Option<DissolveState>,
        created_seconds: u64,
        community_fund_seconds: Option<u64>,
        known_neuron_data: Option<KnownNeuronData>,
        controller: Option<Principal>,
        hotkeys: Vec<Principal>,
        neuron_type: Option<NeuronType>,
        kyc_verified: bool,
        not_for_profit: bool,
        recent_votes: Option<usize>,
        followees: HashMap<Topic, Vec<u64>>,
        total_followees: usize,
        visibility: Option<Visibility>,
    }
    #[derive(Template)]
    #[template(path = "nns/list_neurons.txt")]
    struct ListNeurons {
        neurons: Vec<FullNeuron>,
    }
    let fmt = ListNeurons {
        neurons: neurons
            .full_neurons
            .into_iter()
            .map(|neuron| FullNeuron {
                aging_seconds: neuron.aging_since_timestamp_seconds,
                auto_stake_maturity: neuron.auto_stake_maturity(),
                community_fund_seconds: neuron.joined_community_fund_timestamp_seconds,
                controller: neuron.controller.map(|p| p.0),
                created_seconds: neuron.created_timestamp_seconds,
                deciding: neuron.deciding_voting_power().into(),
                dissolve_delay: neuron.dissolve_state,
                followees: neuron
                    .followees
                    .iter()
                    .map(|(topic, followees)| {
                        (
                            Topic::try_from(*topic).unwrap(),
                            followees
                                .followees
                                .iter()
                                .map(|followee| followee.id)
                                .collect_vec(),
                        )
                    })
                    .collect(),
                hotkeys: neuron.hot_keys.iter().map(|p| p.0).collect(),
                last_refreshed_seconds: neuron.voting_power_refreshed_timestamp_seconds(),
                neuron_type: neuron.neuron_type.map(|_| neuron.neuron_type()),
                potential: neuron.potential_voting_power().into(),
                visibility: neuron.visibility.map(|_| neuron.visibility()),
                id: neuron.id.unwrap().id,
                known_neuron_data: neuron.known_neuron_data,
                kyc_verified: neuron.kyc_verified,
                not_for_profit: neuron.not_for_profit,
                recent_votes: (!neuron.recent_ballots.is_empty())
                    .then_some(neuron.recent_ballots.len()),
                spawn_at_seconds: neuron.spawn_at_timestamp_seconds,
                staked_icp_e8s: neuron.cached_neuron_stake_e8s.into(),
                staked_maturity: neuron.staked_maturity_e8s_equivalent.map(|n| n.into()),
                total_followees: neuron.followees.values().map(|f| f.followees.len()).sum(),
            })
            .collect(),
    }
    .render()?;
    Ok(fmt)
}

pub fn display_manage_neuron(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, ManageNeuronResponse)?;
    let cmd = response.command.context("command was null")?;
    use Command::*;
    #[derive(Template)]
    #[template(path = "nns/manage_neuron.txt")]
    struct ManageNeuron {
        cmd: Command,
    }
    let fmt = ManageNeuron { cmd }.render()?;
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
                Action::InstallCode(a) => {
                    let install_mode = match a.install_mode() {
                        CanisterInstallMode::Unspecified => "Install (unspecified mode)",
                        CanisterInstallMode::Install => "Install",
                        CanisterInstallMode::Reinstall => "Reinstall",
                        CanisterInstallMode::Upgrade => "Upgrade",
                    };
                    let (canister_id, _) = a
                        .canister_and_function()
                        .map_err(|e| anyhow!(display_governance_error(e)))?;
                    let canister_name = canister_id_to_nns_canister_name(canister_id);
                    writeln!(fmt, "{install_mode} canister {canister_name}")?;
                    writeln!(
                        fmt,
                        "WASM blob hash: {}",
                        hex::encode(Sha256::digest(a.wasm_module()))
                    )?;
                    if a.skip_stopping_before_installing() {
                        writeln!(
                            fmt,
                            "Canister will NOT be stopped before installing new WASM"
                        )?;
                    }
                    if let Some(arg) = &a.arg {
                        if let Ok(payload) = get_idl_string(
                            arg,
                            canister_id.into(),
                            get_default_role(canister_id.into()).unwrap_or_default(),
                            ".",
                            "args",
                        ) {
                            writeln!(fmt, "Init args: {payload}",)?;
                        } else {
                            writeln!(fmt, "Init args: {}", hex::encode(arg))?;
                        }
                    }
                }
                Action::StopOrStartCanister(a) => {
                    let action = match a.action() {
                        CanisterAction::Start => "Start",
                        CanisterAction::Stop => "Stop",
                        CanisterAction::Unspecified => "Start/stop (unspecified)",
                    };
                    let (canister_id, _) = a
                        .canister_and_function()
                        .map_err(|e| anyhow!(display_governance_error(e)))?;
                    let canister_name = canister_id_to_nns_canister_name(canister_id);

                    writeln!(fmt, "{action} canister {canister_name}")?;
                }
                Action::UpdateCanisterSettings(a) => {
                    let (canister_id, _) = a
                        .canister_and_function()
                        .map_err(|e| anyhow!(display_governance_error(e)))?;
                    let canister_name = canister_id_to_nns_canister_name(canister_id);
                    writeln!(fmt, "Update settings of canister {canister_name}")?;
                    fmt.push_str(&display_canister_settings(a.settings.unwrap())?);
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
                        ProposalCommand::RefreshVotingPower(_) => {
                            writeln!(fmt, "Refresh the voting power of neuron {neuron}")?
                        }
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
                                Operation::SetVisibility(set_visibility) => writeln!(
                                    fmt,
                                    "Set visibility of {neuron} to {visibility:?}",
                                    visibility = set_visibility.visibility()
                                )?,
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

fn display_canister_settings(settings: CanisterSettings) -> AnyhowResult<String> {
    let mut fmt = String::new();
    if let Some(controllers) = &settings.controllers {
        let controllers = controllers
            .controllers
            .iter()
            .map(|&c| {
                CanisterId::try_from_principal_id(c)
                    .map_or_else(|c| c.to_string(), canister_id_to_nns_canister_name)
            })
            .format(", ");
        writeln!(fmt, "Controllers: {}", controllers)?;
    }
    if let Some(freezing) = settings.freezing_threshold {
        writeln!(
            fmt,
            "Freezing threshold: {} cycles",
            format_t_cycles(freezing.into())
        )?;
    }
    if let Some(memory) = settings.memory_allocation {
        writeln!(fmt, "Memory allocation: {memory}%")?;
    }
    if let Some(compute) = settings.compute_allocation {
        writeln!(fmt, "Compute allocation: {compute}%")?;
    }
    if settings.log_visibility.is_some() {
        writeln!(fmt, "Log visibility: {:?}", settings.log_visibility())?;
    }
    if let Some(limit) = settings.wasm_memory_limit {
        writeln!(fmt, "WASM memory limit: {}", HumanBytes(limit))?;
    }
    if fmt.is_empty() {
        Ok("No changes to canister settings\n".into())
    } else {
        Ok(fmt)
    }
}
