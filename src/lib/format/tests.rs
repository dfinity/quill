//! Regression tests for response formatting. These cover units and missing-field
//! handling that are easy to get wrong in a template and that no golden file in
//! `tests/output` exercises, since those only cover request generation.

use candid::{Encode, Nat, Principal};
use ic_base_types::PrincipalId;
use ic_nervous_system_clients::canister_status::{
    CanisterStatusResultV2, CanisterStatusType, DefiniteCanisterSettingsArgs,
};
use ic_nns_common::pb::v1::{NeuronId, ProposalId};
use ic_nns_governance_api::{
    manage_neuron::{DisburseMaturity, ManageNeuronProposalCommand, NeuronIdOrSubaccount},
    neuron::Followees,
    proposal::Action,
    Account, CanisterSettings, InstallCode, ListNeuronsResponse, ManageNeuronProposal, Neuron,
    Proposal, ProposalInfo, UpdateCanisterSettings, UpdateStandardEngineReplicaVersion,
};
use ic_sns_root::{CanisterSummary, GetSnsCanistersSummaryResponse};
use icp_ledger::TransferError;

const NNS_ROOT: Principal = Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 3, 1, 1]);

fn canister_status(
    freezing_threshold_seconds: u64,
    idle_cycles_burned_per_day: u64,
    memory_allocation_bytes: u64,
) -> CanisterStatusResultV2 {
    CanisterStatusResultV2 {
        status: CanisterStatusType::Running,
        module_hash: None,
        settings: DefiniteCanisterSettingsArgs {
            controllers: vec![PrincipalId(NNS_ROOT)],
            compute_allocation: Nat::from(0_u32),
            memory_allocation: Nat::from(memory_allocation_bytes),
            freezing_threshold: Nat::from(freezing_threshold_seconds),
            wasm_memory_limit: None,
            wasm_memory_threshold: None,
        },
        memory_size: Nat::from(4_000_000_u64),
        memory_metrics: None,
        cycles: Nat::from(3_500_000_000_000_u64),
        idle_cycles_burned_per_day: Nat::from(idle_cycles_burned_per_day),
        query_stats: None,
    }
}

fn canisters_summary(status: CanisterStatusResultV2) -> Vec<u8> {
    let summary = || CanisterSummary {
        canister_id: Some(PrincipalId(NNS_ROOT)),
        status: Some(status.clone()),
    };
    let response = GetSnsCanistersSummaryResponse {
        root: Some(summary()),
        governance: Some(summary()),
        ledger: Some(summary()),
        swap: Some(summary()),
        index: Some(summary()),
        dapps: vec![],
        archives: vec![],
    };
    Encode!(&response).unwrap()
}

fn proposal(action: Action, topic: i32) -> Vec<u8> {
    let info = ProposalInfo {
        id: Some(ProposalId { id: 7 }),
        topic,
        proposal: Some(Proposal {
            title: Some("test".into()),
            summary: String::new(),
            url: String::new(),
            action: Some(action),
            self_describing_action: None,
        }),
        ..Default::default()
    };
    Encode!(&Some(info)).unwrap()
}

fn update_settings(settings: CanisterSettings) -> Action {
    Action::UpdateCanisterSettings(UpdateCanisterSettings {
        canister_id: Some(PrincipalId(NNS_ROOT)),
        settings: Some(settings),
    })
}

/// The freezing threshold is a number of seconds of idle operation, not a cycle balance.
#[test]
fn freezing_threshold_is_a_duration() {
    const THIRTY_DAYS: u64 = 30 * 24 * 60 * 60;
    let fmt = super::sns_root::display_canisters_summary(&canisters_summary(canister_status(
        THIRTY_DAYS,
        100_000_000_000,
        0,
    )))
    .unwrap();
    assert!(
        fmt.contains(
            "Freezing threshold: 30 days of idle operation \
             (3.0T cycles at the current idle usage of 100B cycles/day)"
        ),
        "{fmt}"
    );
}

/// A canister that has never burned a cycle used to panic the summary on a division by zero.
#[test]
fn zero_idle_burn_does_not_panic() {
    let fmt =
        super::sns_root::display_canisters_summary(&canisters_summary(canister_status(60, 0, 0)))
            .unwrap();
    assert!(
        fmt.contains("Freezing threshold: 1 minute of idle operation"),
        "{fmt}"
    );
}

/// The memory allocation is a byte count, and zero means the canister has none.
#[test]
fn memory_allocation_is_a_byte_count() {
    let fmt = super::sns_root::display_canisters_summary(&canisters_summary(canister_status(
        60,
        1,
        1 << 30,
    )))
    .unwrap();
    assert!(fmt.contains("Memory allocation: 1.00 GiB"), "{fmt}");

    let fmt =
        super::sns_root::display_canisters_summary(&canisters_summary(canister_status(60, 1, 0)))
            .unwrap();
    assert!(
        fmt.contains("Memory allocation: none (best-effort)"),
        "{fmt}"
    );

    let fmt = super::nns_governance::display_get_proposal(&proposal(
        update_settings(CanisterSettings {
            memory_allocation: Some(1 << 30),
            ..Default::default()
        }),
        1,
    ))
    .unwrap();
    assert!(fmt.contains("Memory allocation: 1.00 GiB"), "{fmt}");
}

/// `allowed_window_nanos` is how long the ledger accepts a transaction for, not a deadline.
#[test]
fn tx_too_old_reports_the_window_as_a_duration() {
    let result: Result<u64, TransferError> = Err(TransferError::TxTooOld {
        allowed_window_nanos: 24 * 60 * 60 * 1_000_000_000,
    });
    let fmt = super::ledger::display_icp_transfer(&Encode!(&result).unwrap()).unwrap();
    assert!(
        fmt.contains("only accepts transactions created within the last 1 day"),
        "{fmt}"
    );
}

/// A proposal that changes only the WASM memory threshold must not display as empty.
#[test]
fn wasm_memory_threshold_is_displayed() {
    let fmt = super::nns_governance::display_get_proposal(&proposal(
        update_settings(CanisterSettings {
            wasm_memory_threshold: Some(1 << 28),
            ..Default::default()
        }),
        1,
    ))
    .unwrap();
    assert!(fmt.contains("WASM memory threshold: 256.00 MiB"), "{fmt}");

    let fmt = super::nns_governance::display_get_proposal(&proposal(
        update_settings(CanisterSettings::default()),
        1,
    ))
    .unwrap();
    assert!(fmt.contains("No changes to canister settings"), "{fmt}");
}

/// An `Account` is allowed to arrive without an owner; displaying one used to panic.
#[test]
fn ownerless_account_is_displayable() {
    let fmt = super::nns_governance::display_get_proposal(&proposal(
        Action::ManageNeuron(Box::new(ManageNeuronProposal {
            id: None,
            neuron_id_or_subaccount: Some(NeuronIdOrSubaccount::NeuronId(NeuronId { id: 42 })),
            command: Some(ManageNeuronProposalCommand::DisburseMaturity(
                DisburseMaturity {
                    percentage_to_disburse: 50,
                    to_account: Some(Account {
                        owner: None,
                        subaccount: None,
                    }),
                    to_account_identifier: None,
                },
            )),
        })),
        1,
    ))
    .unwrap();
    assert!(
        fmt.contains("Disburse 50% of the maturity from neuron 42 to unknown account"),
        "{fmt}"
    );
}

/// A topic id from a later governance release must not abort the whole display.
#[test]
fn unknown_topic_still_displays() {
    let fmt = super::nns_governance::display_get_proposal(&proposal(
        update_settings(CanisterSettings::default()),
        999,
    ))
    .unwrap();
    assert!(fmt.contains("\"test\" (unknown topic 999)"), "{fmt}");
}

/// Proposer-controlled text must not be able to emit terminal control sequences,
/// e.g. to clear the screen and hide the real proposed action.
#[test]
fn proposal_text_cannot_drive_the_terminal() {
    let info = ProposalInfo {
        id: Some(ProposalId { id: 7 }),
        topic: 1,
        proposal: Some(Proposal {
            title: Some("title\x1b[2J\x07".into()),
            summary: "line one\nline two\x1b]52;c;AAAA\x07\r\u{9b}\u{202e}".into(),
            url: String::new(),
            action: Some(Action::Motion(ic_nns_governance_api::Motion {
                motion_text: "motion\x1b[8m".into(),
            })),
            self_describing_action: None,
        }),
        ..Default::default()
    };
    let fmt = super::nns_governance::display_get_proposal(&Encode!(&Some(info)).unwrap()).unwrap();
    assert!(
        !fmt.chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t'),
        "{fmt:?}"
    );
    assert!(!fmt.contains('\u{202e}'), "{fmt:?}");
    assert!(fmt.contains(r#""title\u{1b}[2J\u{7}""#), "{fmt}");
    assert!(fmt.contains("line one\nline two\\u{1b}]52;c;AAAA"), "{fmt}");
    assert!(fmt.contains(r#""motion\u{1b}[8m" (motion)"#), "{fmt}");
}

#[test]
fn terminal_escaper_keeps_ordinary_text() {
    use askama::filters::Escaper;
    let mut out = String::new();
    super::TerminalText
        .write_escaped_str(&mut out, "Grüße, 世界\n\tdone")
        .unwrap();
    assert_eq!(out, "Grüße, 世界\n\tdone");
}

/// Distinct unrecognized topics used to collapse onto each other, losing followees.
#[test]
fn unknown_topics_do_not_collapse() {
    let mut neuron = Neuron {
        id: Some(NeuronId { id: 1 }),
        ..Default::default()
    };
    for (topic, followee) in [(900_i32, 11_u64), (901, 22), (3, 33)] {
        neuron.followees.insert(
            topic,
            Followees {
                followees: vec![NeuronId { id: followee }],
            },
        );
    }
    let response = ListNeuronsResponse {
        full_neurons: vec![neuron],
        ..Default::default()
    };
    let fmt = super::nns_governance::display_list_neurons(&Encode!(&response).unwrap()).unwrap();
    assert!(
        fmt.contains(
            "Followees: neurons 33 (NetworkEconomics), \
             neurons 11 (unknown topic 900), neurons 22 (unknown topic 901)"
        ),
        "{fmt}"
    );
}

/// A proposal whose action is a variant this build of quill does not know about
/// (e.g. added to governance after quill's pinned API) decodes with `action: None`
/// under Candid's `opt` rule. It must be flagged, not silently omitted.
#[test]
fn unknown_action_is_flagged() {
    use candid::CandidType;
    use std::collections::HashMap;

    #[derive(CandidType)]
    enum FutureAction {
        SomeActionFromTheFuture { new_version: String },
    }
    #[derive(CandidType)]
    struct FutureProposal {
        title: Option<String>,
        summary: String,
        url: String,
        action: Option<FutureAction>,
    }
    #[derive(CandidType)]
    struct FutureProposalInfo {
        id: Option<ProposalId>,
        reject_cost_e8s: u64,
        proposal: Option<FutureProposal>,
        proposal_timestamp_seconds: u64,
        ballots: HashMap<u64, ic_nns_governance_api::Ballot>,
        decided_timestamp_seconds: u64,
        executed_timestamp_seconds: u64,
        failed_timestamp_seconds: u64,
        reward_event_round: u64,
        topic: i32,
        status: i32,
        reward_status: i32,
    }
    let info = FutureProposalInfo {
        id: Some(ProposalId { id: 7 }),
        reject_cost_e8s: 0,
        proposal: Some(FutureProposal {
            title: Some("harmless".into()),
            summary: "nothing to see here".into(),
            url: String::new(),
            action: Some(FutureAction::SomeActionFromTheFuture {
                new_version: "deadbeef".into(),
            }),
        }),
        proposal_timestamp_seconds: 0,
        ballots: HashMap::new(),
        decided_timestamp_seconds: 0,
        executed_timestamp_seconds: 0,
        failed_timestamp_seconds: 0,
        reward_event_round: 0,
        topic: 1,
        status: 1,
        reward_status: 1,
    };
    let blob = Encode!(&Some(info)).unwrap();
    let fmt = super::nns_governance::display_get_proposal(&blob).unwrap();
    assert!(fmt.contains("\"harmless\""), "{fmt}");
    assert!(fmt.contains("Proposed action: UNKNOWN"), "{fmt}");
    assert!(
        fmt.contains("does not recognize this proposal's action"),
        "{fmt}"
    );
}

#[test]
fn update_standard_engine_replica_version_is_displayed() {
    let fmt = super::nns_governance::display_get_proposal(&proposal(
        Action::UpdateStandardEngineReplicaVersion(UpdateStandardEngineReplicaVersion {
            old_replica_version_id: Some("aaaa".into()),
            new_replica_version_id: Some("bbbb".into()),
            deployment_progress: Some(0.25),
        }),
        1,
    ))
    .unwrap();
    assert!(
        fmt.contains(
            "Proposed action: Update standard engine replica version\n\
             Old replica version: aaaa\n\
             New replica version: bbbb\n\
             Deployment progress: 0.25"
        ),
        "{fmt}"
    );
}

/// Canister-management proposals must name the canister being acted on, not root
/// (which is merely the canister governance calls to carry the proposal out).
#[test]
fn install_code_names_the_target_canister() {
    let governance = Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 1, 1, 1]);
    let fmt = super::nns_governance::display_get_proposal(&proposal(
        Action::InstallCode(InstallCode {
            canister_id: Some(PrincipalId(governance)),
            install_mode: Some(3),
            ..Default::default()
        }),
        1,
    ))
    .unwrap();
    assert!(fmt.contains("Upgrade canister governance"), "{fmt}");
}
