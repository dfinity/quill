use askama::Template;
use candid::{Decode, Principal};
use ic_nervous_system_clients::canister_status::CanisterStatusResultV2;
use ic_sns_root::GetSnsCanistersSummaryResponse;

use crate::lib::{format::filters, AnyhowResult};

const NNS_ROOT: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x01]);

pub fn display_canisters_summary(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, GetSnsCanistersSummaryResponse)?;
    let root_summary = response.root_canister_summary();
    let governance_summary = response.governance_canister_summary();
    let ledger_summary = response.ledger_canister_summary();
    let index_summary = response.index_canister_summary();
    let swap_summary = response.swap_canister_summary();
    let root = root_summary.canister_id().0;
    let governance = governance_summary.canister_id().0;
    let fmt = CanistersSummary {
        root: SingleCanisterSummary {
            status: root_summary.status().clone(),
            canister_id: root_summary.canister_id().0,
            root,
            governance,
        },
        governance: SingleCanisterSummary {
            status: governance_summary.status().clone(),
            canister_id: governance_summary.canister_id().0,
            root,
            governance,
        },
        ledger: SingleCanisterSummary {
            status: ledger_summary.status().clone(),
            canister_id: ledger_summary.canister_id().0,
            root,
            governance,
        },
        index: SingleCanisterSummary {
            status: index_summary.status().clone(),
            canister_id: index_summary.canister_id().0,
            root,
            governance,
        },
        swap: SingleCanisterSummary {
            status: swap_summary.status().clone(),
            canister_id: swap_summary.canister_id().0,
            root,
            governance,
        },
        dapps: response
            .dapp_canister_summaries()
            .iter()
            .map(|summary| SingleCanisterSummary {
                status: summary.status().clone(),
                canister_id: summary.canister_id().0,
                root,
                governance,
            })
            .collect(),
        archives: response
            .archives_canister_summaries()
            .iter()
            .map(|summary| SingleCanisterSummary {
                status: summary.status().clone(),
                canister_id: summary.canister_id().0,
                root,
                governance,
            })
            .collect(),
    }
    .render()?;
    Ok(fmt)
}

#[derive(Template)]
#[template(path = "sns/canister_summary.txt")]
struct SingleCanisterSummary {
    status: CanisterStatusResultV2,
    canister_id: Principal,
    root: Principal,
    governance: Principal,
}

#[derive(Template)]
#[template(path = "sns/canisters_summary.txt")]
struct CanistersSummary {
    root: SingleCanisterSummary,
    governance: SingleCanisterSummary,
    ledger: SingleCanisterSummary,
    index: SingleCanisterSummary,
    swap: SingleCanisterSummary,
    dapps: Vec<SingleCanisterSummary>,
    archives: Vec<SingleCanisterSummary>,
}
