use crate::commands::get_principal;
use crate::lib::{AnyhowResult, AuthInfo};
use crate::AUTH_FLAGS;
use candid::Principal;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_nervous_system_common::ledger;
use ic_sns_governance::pb::v1::NeuronId;

#[derive(Parser)]
pub struct NeuronIdOpts {
    /// Principal used when calculating the SNS Neuron Id.
    #[arg(long, required_unless_present_any = AUTH_FLAGS)]
    principal_id: Option<Principal>,

    /// Memo used when calculating the SNS Neuron Id.
    #[arg(long)]
    memo: u64,
}

/// Prints the SNS Neuron Id.
pub fn exec(auth: &AuthInfo, opts: NeuronIdOpts) -> AnyhowResult {
    let principal_id = if let Some(principal_id) = opts.principal_id {
        principal_id
    } else {
        get_principal(auth)?
    };

    let neuron_id = NeuronId::from(ledger::compute_neuron_staking_subaccount_bytes(
        PrincipalId::from(principal_id),
        opts.memo,
    ));

    println!("SNS Neuron Id: {neuron_id}");

    Ok(())
}
