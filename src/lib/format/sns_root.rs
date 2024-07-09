use candid::{Decode, Principal};
use ic_sns_root::{CanisterSummary, GetSnsCanistersSummaryResponse};
use indicatif::HumanBytes;
use itertools::Itertools;
use std::fmt::Write;

use crate::lib::{format::format_n_cycles, AnyhowResult};

use super::format_t_cycles;

pub fn display_canisters_summary(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, GetSnsCanistersSummaryResponse)?;
    let root = response.root_canister_summary().canister_id().0;
    let governance = response.governance_canister_summary().canister_id().0;
    let mut fmt = format!(
        "\
System canisters:

Root:
{root}

Governance:
{governance}

Ledger:
{ledger}

Index:
{index}

Swap:
{swap}",
        root = display_canister_summary(response.root_canister_summary(), root, governance)?,
        governance =
            display_canister_summary(response.governance_canister_summary(), root, governance)?,
        ledger = display_canister_summary(response.ledger_canister_summary(), root, governance)?,
        index = display_canister_summary(response.index_canister_summary(), root, governance)?,
        swap = display_canister_summary(response.swap_canister_summary(), root, governance)?,
    );
    if !response.dapps.is_empty() {
        fmt.push_str("\n\nDapp canisters:");
        for dapp in &response.dapps {
            write!(
                fmt,
                "\n\n{}",
                display_canister_summary(dapp, root, governance)?
            )?;
        }
    }
    if !response.archives.is_empty() {
        fmt.push_str("\n\nArchive canisters:");
        for archive in &response.archives {
            write!(
                fmt,
                "\n\n{}",
                display_canister_summary(archive, root, governance)?
            )?;
        }
    }
    Ok(fmt)
}

fn display_canister_summary(
    summary: &CanisterSummary,
    root: Principal,
    governance: Principal,
) -> AnyhowResult<String> {
    const NNS_ROOT: Principal =
        Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x01]);
    let status = summary.status();
    let canister_id = summary.canister_id();
    let mut fmt = format!(
        "\
Canister ID: {canister_id}, status: {status:?}
Cycles: {cycles}, memory usage: {memory}",
        status = status.status(),
        cycles = format_t_cycles(status.cycles.clone()),
        memory = HumanBytes(status.memory_size().get())
    );
    if let Some(hash) = &status.module_hash {
        write!(fmt, "\nInstalled module: hash {}", hex::encode(hash))?;
    }
    let freezing = &status.settings.freezing_threshold;
    let idle = &status.idle_cycles_burned_per_day;
    let freezing_time = freezing.clone() / idle.clone();
    write!(
        fmt,
        "
Freezing threshold: {freezing} cycles ({freezing_time} days at current idle usage of {idle}/day)
Memory allocation: {memory}%, compute allocation: {compute}%
Controllers: {controllers}",
        freezing = format_t_cycles(freezing.clone()),
        idle = format_n_cycles(idle.clone()),
        memory = status.settings.memory_allocation,
        compute = status.settings.compute_allocation,
        controllers = status
            .settings
            .controllers
            .iter()
            .format_with(", ", |c, f| if c.0 == NNS_ROOT {
                f(&"NNS root")
            } else if c.0 == governance {
                f(&"SNS governance")
            } else if c.0 == root {
                f(&"SNS root")
            } else if *c == canister_id {
                f(&"self")
            } else {
                f(c)
            })
    )?;

    Ok(fmt)
}
