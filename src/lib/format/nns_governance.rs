use std::{collections::HashMap, fmt::Write};

use anyhow::{anyhow, bail, Context};
use askama::Template;
use candid::{Decode, Nat, Principal};
use ic_nns_governance::{
    pb::v1::{
        add_or_remove_node_provider::Change,
        install_code::CanisterInstallMode,
        manage_neuron::{configure::Operation, Command as ProposalCommand, NeuronIdOrSubaccount},
        proposal::Action,
        reward_node_provider::RewardMode,
        stop_or_start_canister::CanisterAction,
        Account, GovernanceError, KnownNeuronData, NeuronState, NeuronType, ProposalRewardStatus,
        Topic, Visibility,
    },
    proposals::call_canister::CallCanister,
};
use ic_nns_governance_api::{
    claim_or_refresh_neuron_from_account_response::Result as ClaimResult,
    manage_neuron_response::Command, neuron::DissolveState,
    ClaimOrRefreshNeuronFromAccountResponse, ListNeuronsResponse, ListProposalInfoResponse,
    ManageNeuronResponse, NeuronInfo, ProposalInfo, ProposalStatus,
};
use itertools::Itertools;
use sha2::{Digest, Sha256};

use crate::lib::{
    format::{filters, icrc1_account},
    AnyhowResult, ParsedAccount,
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
            deciding: info.deciding_voting_power.unwrap_or_default().into(),
            potential: info.potential_voting_power.unwrap_or_default().into(),
            last_refreshed_seconds: info
                .voting_power_refreshed_timestamp_seconds
                .context("voting power refreshed timestamp was null")?,
            state: NeuronState::try_from(info.state).unwrap_or_default(),
            visibility: info
                .visibility
                .map(|vis| Visibility::try_from(vis).unwrap_or_default()),
            dissolve_delay_seconds: info.dissolve_delay_seconds,
            created_seconds: info.created_timestamp_seconds,
            community_fund_seconds: info.joined_community_fund_timestamp_seconds,
            known_neuron_data: info.known_neuron_data.map(Into::into),
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
            .map(|neuron| {
                Ok(FullNeuron {
                    aging_seconds: neuron.aging_since_timestamp_seconds,
                    auto_stake_maturity: neuron.auto_stake_maturity.unwrap_or_default(),
                    community_fund_seconds: neuron.joined_community_fund_timestamp_seconds,
                    controller: neuron.controller.map(|p| p.0),
                    created_seconds: neuron.created_timestamp_seconds,
                    deciding: neuron.deciding_voting_power.unwrap_or(0).into(),
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
                    last_refreshed_seconds: neuron
                        .voting_power_refreshed_timestamp_seconds
                        .context("voting power refreshed timestamp was null")?,
                    neuron_type: neuron
                        .neuron_type
                        .map(|t| NeuronType::try_from(t).unwrap_or_default()),
                    potential: neuron.potential_voting_power.unwrap_or_default().into(),
                    visibility: neuron
                        .visibility
                        .map(|vis| Visibility::try_from(vis).unwrap_or_default()),
                    id: neuron.id.unwrap().id,
                    known_neuron_data: neuron.known_neuron_data.map(Into::into),
                    kyc_verified: neuron.kyc_verified,
                    not_for_profit: neuron.not_for_profit,
                    recent_votes: (!neuron.recent_ballots.is_empty())
                        .then_some(neuron.recent_ballots.len()),
                    spawn_at_seconds: neuron.spawn_at_timestamp_seconds,
                    staked_icp_e8s: neuron.cached_neuron_stake_e8s.into(),
                    staked_maturity: neuron.staked_maturity_e8s_equivalent.map(|n| n.into()),
                    total_followees: neuron.followees.values().map(|f| f.followees.len()).sum(),
                })
            })
            .collect::<AnyhowResult<Vec<_>>>()?,
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
    use Action::*;
    use Change::*;
    use NeuronIdOrSubaccount::*;
    use Operation::*;
    use ProposalCommand::*;
    use RewardMode::*;
    #[derive(Template)]
    #[template(path = "nns/proposal_info.txt")]
    struct GetProposalInfo {
        proposal_info: ProposalInfo,
    }
    let fmt = GetProposalInfo { proposal_info }.render()?;
    Ok(fmt)
}

pub fn display_neuron_ids(blob: &[u8]) -> AnyhowResult<String> {
    let ids = Decode!(blob, Vec<u64>)?;
    #[derive(Template)]
    #[template(path = "nns/neuron_ids.txt")]
    struct NeuronIds {
        ids: Vec<u64>,
    }
    let fmt = NeuronIds { ids }.render()?;
    Ok(fmt)
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
    #[derive(Template)]
    #[template(path = "nns/claim_or_refresh_neuron_from_account.txt")]
    struct ClaimOrRefreshNeuronFromAccount {
        id: u64,
    }
    let fmt = if let Some(res) = res.result {
        match res {
            ClaimResult::NeuronId(id) => ClaimOrRefreshNeuronFromAccount { id: id.id }.render()?,
            ClaimResult::Error(e) => display_governance_error(e.into()),
        }
    } else {
        "Unknown result of call".to_string()
    };
    Ok(fmt)
}

pub fn display_governance_error(err: GovernanceError) -> String {
    format!("NNS error: {}", err.error_message)
}

fn map_governance_error<T>(res: Result<T, GovernanceError>) -> AnyhowResult<T> {
    res.map_err(|e| anyhow!(e.error_message))
}

fn get_topic(topic: &i32) -> AnyhowResult<Topic> {
    Topic::try_from(*topic).context("Unknown topic")
}

fn get_status(status: &i32) -> AnyhowResult<ProposalStatus> {
    ProposalStatus::from_repr(*status).context("Unknown proposal status")
}

fn get_reward_status(status: &i32) -> AnyhowResult<ProposalRewardStatus> {
    ProposalRewardStatus::try_from(*status).context("Unknown proposal reward status")
}

fn sns_unsupported() -> AnyhowResult<String> {
    bail!("SNS proposals currently unsupported")
}

fn nested_proposals_not_supported() -> AnyhowResult<String> {
    bail!("Nested proposals not supported")
}

fn icrc1_helper(account: &Account) -> AnyhowResult<ParsedAccount> {
    Ok(icrc1_account(
        account.owner.unwrap().0,
        Some(account.subaccount.as_ref().map_or(Ok([0; 32]), |s| {
            s.subaccount[..]
                .try_into()
                .map_err(|_| anyhow!("subaccount had wrong length"))
        })?),
    ))
}
