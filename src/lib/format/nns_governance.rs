use std::{collections::HashMap, fmt::Write};

use anyhow::{anyhow, bail, Context};
use askama::Template;
use candid::{Decode, Nat, Principal};
use ic_nns_governance::{
    pb::v1::{
        add_or_remove_node_provider::Change,
        claim_or_refresh_neuron_from_account_response::Result as ClaimResult,
        manage_neuron::{configure::Operation, Command as ProposalCommand, NeuronIdOrSubaccount},
        manage_neuron_response::Command,
        neuron::DissolveState,
        proposal::Action,
        reward_node_provider::RewardMode,
        ClaimOrRefreshNeuronFromAccountResponse, GovernanceError, KnownNeuronData,
        ListNeuronsResponse, ListProposalInfoResponse, ManageNeuronResponse, NeuronInfo,
        NeuronState, NeuronType, ProposalInfo, Topic, Visibility,
    },
    proposals::call_canister::CallCanister,
};
use itertools::Itertools;
use sha2::{Digest, Sha256};

use crate::lib::{format::filters, AnyhowResult};

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
            ClaimResult::Error(e) => display_governance_error(e),
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

fn get_topic(topic: &i32) -> Topic {
    Topic::try_from(*topic).unwrap_or_default()
}

fn sns_unsupported() -> AnyhowResult<String> {
    bail!("SNS proposals currently unsupported")
}

fn nested_proposals_not_supported() -> AnyhowResult<String> {
    bail!("Nested proposals not supported")
}
