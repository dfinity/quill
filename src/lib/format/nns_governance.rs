use std::fmt::{self, Display, Formatter, Write};

use anyhow::{anyhow, bail, Context};
use askama::Template;
use bigdecimal::BigDecimal;
use candid::{Decode, Nat, Principal};
use chrono::Utc;
use ic_base_types::{CanisterId, PrincipalId};
use ic_nns_constants::{
    CYCLES_MINTING_CANISTER_ID, LIFELINE_CANISTER_ID, MIGRATION_CANISTER_ID, REGISTRY_CANISTER_ID,
    ROOT_CANISTER_ID, SNS_WASM_CANISTER_ID, SUBNET_RENTAL_CANISTER_ID,
};
use ic_nns_governance::{
    pb::v1::{
        add_or_remove_node_provider::Change,
        install_code::CanisterInstallMode,
        manage_neuron::{configure::Operation, Command as ProposalCommand, NeuronIdOrSubaccount},
        proposal::Action,
        reward_node_provider::RewardMode,
        stop_or_start_canister::CanisterAction,
        Account, CanisterSettings, GovernanceError, KnownNeuronData, NeuronState, NeuronType,
        NnsFunction, ProposalRewardStatus, RewardNodeProviders, Topic, Visibility,
    },
    proposals::call_canister::CallCanister,
};
use ic_nns_governance_api::{
    claim_or_refresh_neuron_from_account_response::Result as ClaimResult,
    manage_neuron_response::Command, neuron::DissolveState, proposal::Action as ApiAction,
    ClaimOrRefreshNeuronFromAccountResponse, ListNeuronsResponse, ListProposalInfoResponse,
    ManageNeuronResponse, NeuronInfo, ProposalInfo, ProposalStatus,
};
use itertools::Itertools;

use crate::lib::{
    format::{filters, icrc1_account},
    AnyhowResult,
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
        last_refreshed_seconds: Option<u64>,
        state: NeuronState,
        dissolve_delay_seconds: u64,
        created_seconds: u64,
        community_fund_seconds: Option<u64>,
        known_neuron_data: Option<KnownNeuronData>,
        visibility: Option<Visibility>,
        eight_year_gang_bonus: Option<Nat>,
        retrieved_seconds: u64,
    }
    let fmt = match info {
        Ok(info) => GetNeuronInfo {
            age_seconds: info.age_seconds,
            stake: info.stake_e8s.into(),
            deciding: info.deciding_voting_power.unwrap_or_default().into(),
            potential: info.potential_voting_power.unwrap_or_default().into(),
            last_refreshed_seconds: info.voting_power_refreshed_timestamp_seconds,
            state: NeuronState::try_from(info.state).unwrap_or_default(),
            visibility: info
                .visibility
                .map(|vis| Visibility::try_from(vis).unwrap_or_default()),
            dissolve_delay_seconds: info.dissolve_delay_seconds,
            created_seconds: info.created_timestamp_seconds,
            community_fund_seconds: info.joined_community_fund_timestamp_seconds,
            known_neuron_data: info.known_neuron_data.map(Into::into),
            eight_year_gang_bonus: eight_year_gang_bonus(info.eight_year_gang_bonus_base_e8s),
            retrieved_seconds: info.retrieved_at_timestamp_seconds,
        }
        .render()?,

        Err(e) => display_governance_error(e),
    };
    Ok(fmt)
}

pub fn display_list_neurons(blob: &[u8]) -> AnyhowResult<String> {
    use DissolveState::*;
    let now_seconds = u64::try_from(Utc::now().timestamp()).unwrap();
    let neurons = Decode!(blob, ListNeuronsResponse)?;
    #[derive(Template)]
    #[template(path = "nns/full_neuron_info.txt")]
    struct FullNeuron {
        id: Option<u64>,
        aging_seconds: Option<u64>,
        staked_icp_e8s: Nat,
        staked_maturity: Option<Nat>,
        auto_stake_maturity: bool,
        deciding: Nat,
        potential: Nat,
        last_refreshed_seconds: Option<u64>,
        spawn_at_seconds: Option<u64>,
        state: NeuronState,
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
        followees: Vec<(ProposalTopic, Vec<u64>)>,
        total_followees: usize,
        visibility: Option<Visibility>,
        eight_year_gang_bonus: Option<Nat>,
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
                let state = NeuronState::try_from(neuron.state(now_seconds) as i32)
                    .unwrap_or(NeuronState::Unspecified);
                Ok(FullNeuron {
                    aging_seconds: (neuron.aging_since_timestamp_seconds != u64::MAX)
                        .then_some(neuron.aging_since_timestamp_seconds),
                    auto_stake_maturity: neuron.auto_stake_maturity.unwrap_or_default(),
                    community_fund_seconds: neuron.joined_community_fund_timestamp_seconds,
                    controller: neuron.controller.map(|p| p.0),
                    created_seconds: neuron.created_timestamp_seconds,
                    deciding: neuron.deciding_voting_power.unwrap_or(0).into(),
                    dissolve_delay: neuron.dissolve_state,
                    eight_year_gang_bonus: eight_year_gang_bonus(
                        neuron.eight_year_gang_bonus_base_e8s,
                    ),
                    // Keyed by topic id rather than by `Topic` so that two ids this
                    // binary doesn't recognize don't collapse into a single entry,
                    // and sorted because the response's map has no stable order.
                    followees: neuron
                        .followees
                        .iter()
                        .sorted_by_key(|&(topic, _)| *topic)
                        .map(|(topic, followees)| {
                            (
                                ProposalTopic::from(*topic),
                                followees
                                    .followees
                                    .iter()
                                    .map(|followee| followee.id)
                                    .collect_vec(),
                            )
                        })
                        .collect(),
                    hotkeys: neuron.hot_keys.iter().map(|p| p.0).collect(),
                    last_refreshed_seconds: neuron.voting_power_refreshed_timestamp_seconds,
                    neuron_type: neuron
                        .neuron_type
                        .map(|t| NeuronType::try_from(t).unwrap_or_default()),
                    potential: neuron.potential_voting_power.unwrap_or_default().into(),
                    visibility: neuron
                        .visibility
                        .map(|vis| Visibility::try_from(vis).unwrap_or_default()),
                    id: neuron.id.map(|id| id.id),
                    state,
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
    Ok(fmt.trim_end().to_string())
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

/// The ids of the node providers being rewarded, skipping any reward whose provider
/// or provider id is missing (matching the pre-template behavior).
fn reward_node_provider_ids(rewards: &RewardNodeProviders) -> Vec<PrincipalId> {
    rewards
        .rewards
        .iter()
        .filter_map(|r| r.node_provider.as_ref().and_then(|p| p.id))
        .collect()
}

/// True when an `UpdateCanisterSettings` proposal does not actually change anything.
fn no_canister_settings(settings: &CanisterSettings) -> bool {
    let CanisterSettings {
        controllers,
        compute_allocation,
        memory_allocation,
        freezing_threshold,
        log_visibility,
        snapshot_visibility,
        wasm_memory_limit,
        wasm_memory_threshold,
    } = settings;
    controllers.is_none()
        && compute_allocation.is_none()
        && memory_allocation.is_none()
        && freezing_threshold.is_none()
        && log_visibility.is_none()
        && snapshot_visibility.is_none()
        && wasm_memory_limit.is_none()
        && wasm_memory_threshold.is_none()
}

/// The hash of the WASM a `CreateCanisterAndInstallCode` proposal installs.
///
/// Read off the API action rather than the protobuf one the rest of the arm uses:
/// the protobuf type stores the whole `wasm_module` and only caches its hash inside
/// it, and `From<api::CreateCanisterAndInstallCode>` cannot reconstruct the module,
/// so it sets `wasm_module: None` and the hash is lost in the conversion.
fn create_canister_wasm_module_hash(action: &ApiAction) -> Option<&[u8]> {
    match action {
        ApiAction::CreateCanisterAndInstallCode(a) => a.wasm_module_hash.as_deref(),
        _ => None,
    }
}

/// The "8 year gang" dissolve delay bonus base, which is zero for every neuron that
/// didn't have the maximum dissolve delay when the maximum was reduced, and is not
/// worth displaying in that case.
fn eight_year_gang_bonus(base_e8s: Option<u64>) -> Option<Nat> {
    base_e8s.filter(|&base| base > 0).map(Nat::from)
}

/// The canister and method an `ExecuteNnsFunction` payload is destined for, or `None`
/// for a payload that isn't Candid and therefore can't be decoded anyway. Resolving is
/// deliberately skipped for such payloads so that an unrecognized function doesn't turn
/// a hex-printable payload into a hard error.
fn nns_function_target(
    function: &NnsFunction,
    payload: &[u8],
) -> AnyhowResult<Option<(CanisterId, &'static str)>> {
    if payload.starts_with(b"DIDL") {
        nns_function_canister_and_method(*function)
            .map(Some)
            .map_err(|e| anyhow!(e))
    } else {
        Ok(None)
    }
}

/// `NnsFunction::canister_and_function` was removed from the governance crate, so the
/// mapping has to be maintained here.
fn nns_function_canister_and_method(
    function: NnsFunction,
) -> Result<(CanisterId, &'static str), String> {
    let pair = match function {
        NnsFunction::Unspecified => return Err("Unspecified NNS function".to_string()),
        NnsFunction::AssignNoid => (REGISTRY_CANISTER_ID, "add_node_operator"),
        NnsFunction::CreateSubnet => (REGISTRY_CANISTER_ID, "create_subnet"),
        NnsFunction::AddNodeToSubnet => (REGISTRY_CANISTER_ID, "add_nodes_to_subnet"),
        NnsFunction::RemoveNodesFromSubnet => (REGISTRY_CANISTER_ID, "remove_nodes_from_subnet"),
        NnsFunction::ChangeSubnetMembership => (REGISTRY_CANISTER_ID, "change_subnet_membership"),
        NnsFunction::NnsCanisterInstall => (ROOT_CANISTER_ID, "add_nns_canister"),
        NnsFunction::HardResetNnsRootToVersion => {
            (LIFELINE_CANISTER_ID, "hard_reset_root_to_version")
        }
        NnsFunction::RecoverSubnet => (REGISTRY_CANISTER_ID, "recover_subnet"),
        NnsFunction::ReviseElectedGuestosVersions => {
            (REGISTRY_CANISTER_ID, "revise_elected_guestos_versions")
        }
        NnsFunction::UpdateNodeOperatorConfig => {
            (REGISTRY_CANISTER_ID, "update_node_operator_config")
        }
        NnsFunction::DeployGuestosToAllSubnetNodes => {
            (REGISTRY_CANISTER_ID, "deploy_guestos_to_all_subnet_nodes")
        }
        NnsFunction::ReviseElectedHostosVersions => {
            (REGISTRY_CANISTER_ID, "revise_elected_hostos_versions")
        }
        NnsFunction::DeployHostosToSomeNodes => {
            (REGISTRY_CANISTER_ID, "deploy_hostos_to_some_nodes")
        }
        NnsFunction::UpdateConfigOfSubnet => (REGISTRY_CANISTER_ID, "update_subnet"),
        NnsFunction::IcpXdrConversionRate => {
            (CYCLES_MINTING_CANISTER_ID, "set_icp_xdr_conversion_rate")
        }
        NnsFunction::ClearProvisionalWhitelist => {
            (REGISTRY_CANISTER_ID, "clear_provisional_whitelist")
        }
        NnsFunction::SetAuthorizedSubnetworks => {
            (CYCLES_MINTING_CANISTER_ID, "set_authorized_subnetwork_list")
        }
        NnsFunction::SetFirewallConfig => (REGISTRY_CANISTER_ID, "set_firewall_config"),
        NnsFunction::AddFirewallRules => (REGISTRY_CANISTER_ID, "add_firewall_rules"),
        NnsFunction::RemoveFirewallRules => (REGISTRY_CANISTER_ID, "remove_firewall_rules"),
        NnsFunction::UpdateFirewallRules => (REGISTRY_CANISTER_ID, "update_firewall_rules"),
        NnsFunction::StopOrStartNnsCanister => (ROOT_CANISTER_ID, "stop_or_start_nns_canister"),
        NnsFunction::RemoveNodes => (REGISTRY_CANISTER_ID, "remove_nodes"),
        NnsFunction::UninstallCode => (CanisterId::ic_00(), "uninstall_code"),
        NnsFunction::UpdateNodeRewardsTable => (REGISTRY_CANISTER_ID, "update_node_rewards_table"),
        NnsFunction::AddOrRemoveDataCenters => (REGISTRY_CANISTER_ID, "add_or_remove_data_centers"),
        NnsFunction::RemoveNodeOperators => (REGISTRY_CANISTER_ID, "remove_node_operators"),
        NnsFunction::RerouteCanisterRanges => (REGISTRY_CANISTER_ID, "reroute_canister_ranges"),
        NnsFunction::PrepareCanisterMigration => {
            (REGISTRY_CANISTER_ID, "prepare_canister_migration")
        }
        NnsFunction::CompleteCanisterMigration => {
            (REGISTRY_CANISTER_ID, "complete_canister_migration")
        }
        NnsFunction::AddSnsWasm => (SNS_WASM_CANISTER_ID, "add_wasm"),
        NnsFunction::UpdateSubnetType => (CYCLES_MINTING_CANISTER_ID, "update_subnet_type"),
        NnsFunction::ChangeSubnetTypeAssignment => {
            (CYCLES_MINTING_CANISTER_ID, "change_subnet_type_assignment")
        }
        NnsFunction::UpdateSnsWasmSnsSubnetIds => (SNS_WASM_CANISTER_ID, "update_sns_subnet_list"),
        NnsFunction::InsertSnsWasmUpgradePathEntries => {
            (SNS_WASM_CANISTER_ID, "insert_upgrade_path_entries")
        }
        NnsFunction::BitcoinSetConfig => (ROOT_CANISTER_ID, "call_canister"),
        NnsFunction::AddApiBoundaryNodes => (REGISTRY_CANISTER_ID, "add_api_boundary_nodes"),
        NnsFunction::RemoveApiBoundaryNodes => (REGISTRY_CANISTER_ID, "remove_api_boundary_nodes"),
        NnsFunction::DeployGuestosToSomeApiBoundaryNodes => (
            REGISTRY_CANISTER_ID,
            "deploy_guestos_to_some_api_boundary_nodes",
        ),
        NnsFunction::DeployGuestosToAllUnassignedNodes => (
            REGISTRY_CANISTER_ID,
            "deploy_guestos_to_all_unassigned_nodes",
        ),
        NnsFunction::UpdateSshReadonlyAccessForAllUnassignedNodes => (
            REGISTRY_CANISTER_ID,
            "update_ssh_readonly_access_for_all_unassigned_nodes",
        ),
        NnsFunction::SubnetRentalRequest => {
            (SUBNET_RENTAL_CANISTER_ID, "execute_rental_request_proposal")
        }
        NnsFunction::PauseCanisterMigrations => (MIGRATION_CANISTER_ID, "disable_api"),
        NnsFunction::UnpauseCanisterMigrations => (MIGRATION_CANISTER_ID, "enable_api"),
        NnsFunction::SetSubnetOperationalLevel => {
            (REGISTRY_CANISTER_ID, "set_subnet_operational_level")
        }
        _ => return Err(format!("Unknown or obsolete NNS function: {function:?}")),
    };
    Ok(pair)
}

/// `part` as a percentage of `total`, rounded to two decimal places.
fn percentage(part: &u64, total: &u64) -> BigDecimal {
    if *total == 0 {
        return BigDecimal::from(0);
    }
    (BigDecimal::from(*part) / BigDecimal::from(*total) * 100_u8).round(2)
}

/// A governance topic, tolerating ids this build of quill doesn't know about.
///
/// Governance adds topics over time, and neither a neuron's following nor a whole
/// proposal should become undisplayable because one id in it postdates this binary.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ProposalTopic {
    Known(Topic),
    Unknown(i32),
}

impl From<i32> for ProposalTopic {
    fn from(topic: i32) -> Self {
        Topic::try_from(topic).map_or(Self::Unknown(topic), Self::Known)
    }
}

impl Display for ProposalTopic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Known(topic) => write!(f, "{topic:?}"),
            Self::Unknown(id) => write!(f, "unknown topic {id}"),
        }
    }
}

fn topic(topic: &i32) -> ProposalTopic {
    ProposalTopic::from(*topic)
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

/// The textual form of an ICRC-1 account named by a governance command. The owner is
/// optional in the wire format, so an account can legitimately arrive without one.
fn icrc1_helper(account: &Account) -> AnyhowResult<String> {
    let Some(owner) = account.owner else {
        return Ok("unknown account".to_string());
    };
    let subaccount = account.subaccount.as_ref().map_or(Ok([0; 32]), |s| {
        s.subaccount[..]
            .try_into()
            .map_err(|_| anyhow!("subaccount had wrong length"))
    })?;
    Ok(icrc1_account(owner.0, Some(subaccount)).to_string())
}
