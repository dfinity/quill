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
    Account, CanisterSettings, ListNeuronsResponse, ManageNeuronProposal, Neuron, Proposal,
    ProposalInfo, UpdateCanisterSettings,
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

/// A proposal carrying proposer-written text, with an action whose rendering is
/// fixed and unambiguous, so a forgery attempt in the text can be told apart
/// from the real thing.
fn text_proposal(title: &str, summary: &str) -> Vec<u8> {
    let info = ProposalInfo {
        id: Some(ProposalId { id: 7 }),
        topic: 1,
        proposal: Some(Proposal {
            title: Some(title.into()),
            summary: summary.into(),
            url: String::new(),
            action: Some(update_settings(CanisterSettings::default())),
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

/// A proposal's title, summary and URL are written by whoever submitted it, and
/// the governance canister validates them by length alone. Terminal control
/// codes in them would let the proposer decide what a reviewer sees before
/// authorizing a vote: `ESC[8m` conceals the lines after it, `ESC[2J` clears the
/// screen so a forgery can be drawn in its place, a lone `CR` overwrites the
/// line just printed, and OSC 52 rewrites the reviewer's clipboard.
#[test]
fn control_codes_are_depicted_in_proposal_text() {
    let fmt = super::nns_governance::display_get_proposal(&text_proposal(
        "Node provider reward\u{1b}[8m",
        "Routine reward.\u{1b}[2J\u{1b}[H\u{1b}]52;c;YWFhYQ==\u{7}\rone line",
    ))
    .unwrap();
    for obeyed in ['\u{1b}', '\r', '\u{7}'] {
        assert!(!fmt.contains(obeyed), "{obeyed:?} survived: {fmt:?}");
    }
    // Depicted rather than dropped, so that the reader can see the attempt.
    assert!(fmt.contains("␛[8m"), "{fmt:?}");
    assert!(fmt.contains("␛[2J␛[H␛]52;c;YWFhYQ==␇␍"), "{fmt:?}");
    // The readable text around them is kept.
    assert!(fmt.contains("Node provider reward"), "{fmt}");
    assert!(fmt.contains("Routine reward."), "{fmt}");
    assert!(fmt.contains("one line"), "{fmt}");
}

/// `|flat` is for the fields that have no business spanning lines. A break in
/// one of those is not laid out and then indented, it is depicted: the reader
/// sees where the proposer put it. U+2424 SYMBOL FOR NEWLINE reads as the end of
/// a line, where U+240A SYMBOL FOR LINE FEED only names the character.
#[test]
fn single_line_fields_depict_their_line_breaks() {
    let fmt = super::nns_governance::display_get_proposal(&text_proposal(
        "Node provider reward\nProposed action: Reward node provider with 12 ICP",
        "Routine reward.",
    ))
    .unwrap();
    assert!(
        fmt.starts_with(
            "\"Node provider reward␤Proposed action: Reward node provider with 12 ICP\""
        ),
        "{fmt:?}"
    );
    // One line in, one line out: the title cannot add a line at any column.
    assert_eq!(
        fmt.lines().filter(|line| line.contains("12 ICP")).count(),
        1,
        "{fmt:?}"
    );
}

/// The other ASCII formatting marks are depicted too, and a tab is the one that
/// matters: it can push text to a column the field was never given. A solitary
/// carriage return is `TerminalSafe`'s to deal with, so it passes through
/// untouched here and is depicted a step later.
#[test]
fn flat_depicts_every_ascii_formatting_mark() {
    use askama::Values;

    let values: &dyn Values = &();
    let flattened = super::filters::flat("a\tb\nc\u{b}d\u{c}e\rf", values).unwrap();
    assert_eq!(flattened, "a␉b␤c␋d␌e\rf");
}

/// A `CRLF` has to be recognized as one line ending here rather than left to the
/// escaper: filters run first, so by the time the escaper sees the value its line
/// feed is already a `␤` and the pair is no longer there to recognize.
#[test]
fn flat_names_a_crlf_line_ending_once() {
    use askama::Values;

    let values: &dyn Values = &();
    assert_eq!(super::filters::flat("a\r\nb", values).unwrap(), "a␤b");
    // Not a line ending, so it keeps its own picture a step later.
    assert_eq!(super::filters::flat("a\r\rb", values).unwrap(), "a\r\rb");
}

/// A summary written on Windows arrives with `CRLF` line endings. The `LF` ends
/// the line on its own, so depicting the `CR` in front of it would put a `␍` at
/// the end of every line to stand for nothing. A `CR` that is not part of a line
/// ending keeps its picture: on its own it overwrites the line just printed,
/// which is how a forged line gets drawn over a real one.
#[test]
fn crlf_line_endings_lose_the_carriage_return() {
    let fmt = super::nns_governance::display_get_proposal(&text_proposal(
        "Reward\r\nAugust\rSeptember",
        "# Reward\r\nMotivation: routine.\rProposed action: forged",
    ))
    .unwrap();
    assert!(!fmt.contains('\r'), "{fmt:?}");
    // An indented field keeps the line the CRLF ended, without the CR.
    assert!(
        fmt.contains("\"# Reward\n    Motivation: routine."),
        "{fmt:?}"
    );
    assert!(fmt.contains("routine.␍Proposed action: forged"), "{fmt:?}");
    // A flattened field names that same line ending once, in the one glyph it
    // has for the purpose, and still depicts the bare CR after it.
    assert!(fmt.starts_with("\"Reward␤August␍September\""), "{fmt:?}");
}

/// Newlines survive, because real summaries are multi-line markdown. That alone
/// would let a summary end in something shaped like one of quill's own fields,
/// so proposer-written text is indented: every genuine field starts at column 0
/// and no line of proposer text does.
#[test]
fn proposal_text_cannot_forge_a_field_line() {
    let forgery = "Proposed action: Reward node provider with 12 ICP";
    let fmt = super::nns_governance::display_get_proposal(&text_proposal(
        "Node provider reward",
        &format!("Routine reward.\n{forgery}"),
    ))
    .unwrap();
    assert!(fmt.contains(&format!("\n    {forgery}")), "{fmt}");
    assert!(
        !fmt.lines().any(|line| line == forgery),
        "forged field line reached column 0: {fmt}"
    );
    // The real action is still rendered, and still at column 0.
    assert!(
        fmt.lines()
            .any(|line| line == "Proposed action: Update settings of canister lifeline"),
        "{fmt}"
    );
}

/// The indentation must not come at the cost of the ordinary case: an honest
/// proposal still renders its text readably.
#[test]
fn honest_proposal_text_is_unchanged() {
    let fmt = super::nns_governance::display_get_proposal(&text_proposal(
        "Node provider reward",
        "Routine monthly node provider reward.",
    ))
    .unwrap();
    assert!(
        fmt.starts_with("\"Node provider reward\" (NeuronManagement)"),
        "{fmt}"
    );
    assert!(
        fmt.contains("Summary: \"Routine monthly node provider reward.\"\n"),
        "{fmt:?}"
    );
}

/// The indent belongs to values, not to layout, which is why it is written per
/// field rather than taken care of once in `format::TerminalSafe`. askama wraps
/// an expression in `AutoEscaper` before `Writable` can pick the specialization
/// that renders a nested template through `Template::render_into_with_values`,
/// so a composed template reaches the escaper as `EscapeDisplay` and streams its
/// own body text through it. An escaper that indented what it was given would
/// therefore indent quill's layout too: every neuron past the first in a list
/// would drift right, and the rule that a field starts at column 0 would not
/// survive composition.
#[test]
fn indent_applies_to_data_not_to_layout() {
    let response = ListNeuronsResponse {
        full_neurons: (1..=3)
            .map(|id| Neuron {
                id: Some(NeuronId { id }),
                cached_neuron_stake_e8s: id * 100_000_000,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let fmt = super::nns_governance::display_list_neurons(&Encode!(&response).unwrap()).unwrap();
    for id in 1..=3 {
        assert!(
            fmt.lines().any(|line| line == format!("Neuron {id}")),
            "neuron {id} was displaced from column 0: {fmt:?}"
        );
    }
    assert!(
        !fmt.contains("\n    "),
        "layout picked up a data indent: {fmt:?}"
    );
}

/// askama's `indent` stops indenting once a value reaches 10,000 characters and
/// passes it through as-is. A summary may be 30,000 bytes, so a proposer could
/// pad past that limit and land a forged field line back at column 0; `indentf`
/// has no such limit.
#[test]
fn a_summary_past_ten_thousand_characters_is_still_indented() {
    let forgery = "Proposed action: Reward node provider with 12 ICP";
    let summary = format!("{}\n{forgery}", "padding. ".repeat(1200));
    assert!(summary.len() > 10_000, "{} chars", summary.len());
    let fmt = super::nns_governance::display_get_proposal(&text_proposal(
        "Node provider reward",
        &summary,
    ))
    .unwrap();
    assert!(fmt.contains(&format!("\n    {forgery}")), "not indented");
    assert!(
        !fmt.lines().any(|line| line == forgery),
        "forged field line reached column 0"
    );
}
