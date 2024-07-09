use std::fmt::Write;

use anyhow::Context;
use candid::Decode;
use ic_sns_wasm::pb::v1::ListDeployedSnsesResponse;

use crate::lib::AnyhowResult;

pub fn display_list_snses(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, ListDeployedSnsesResponse)?;
    let mut fmt = String::new();
    for sns in response.instances {
        let root = sns.root_canister_id.context("root canister was null")?;
        writeln!(
            fmt,
            "https://dashboard.internetcomputer.org/sns/{root}\nRoot canister: {root}"
        )?;
        if let Some(ledger) = sns.ledger_canister_id {
            writeln!(fmt, "Ledger canister: {ledger}")?;
        }
        if let Some(governance) = sns.governance_canister_id {
            writeln!(fmt, "Governance canister: {governance}")?;
        }
        if let Some(swap) = sns.swap_canister_id {
            writeln!(fmt, "Swap canister: {swap}")?;
        }
        if let Some(index) = sns.index_canister_id {
            writeln!(fmt, "Index canister: {index}")?;
        }
        fmt.push('\n');
    }
    Ok(fmt)
}
