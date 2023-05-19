use crate::lib::{get_identity, AnyhowResult, AuthInfo};
use anyhow::anyhow;
use candid::Principal;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_nervous_system_common::ledger;
use ic_sns_governance::pb::v1::NeuronId;

#[derive(Parser)]
pub struct NeuronIdOpts {
    /// Principal used when calculating the SNS Neuron Id.
    #[clap(long)]
    principal_id: Option<String>,

    /// Memo used when calculating the SNS Neuron Id.
    #[clap(long)]
    memo: u64,
}

/// Prints the SNS Neuron Id.
pub fn exec(auth: &AuthInfo, opts: NeuronIdOpts) -> AnyhowResult {
    let principal_id = match &opts.principal_id {
        Some(principal_id) => Principal::from_text(principal_id)?,
        None => {
            if let AuthInfo::NoAuth = auth {
                Err(anyhow!(
                    "neuron-id cannot be used without specifying a private key"
                ))?
            } else {
                get_identity(auth)?.sender().map_err(|e| anyhow!(e))?
            }
        }
    };

    let neuron_id = NeuronId::from(ledger::compute_neuron_staking_subaccount_bytes(
        PrincipalId::from(principal_id),
        opts.memo,
    ));

    println!("SNS Neuron Id: {}", neuron_id);

    Ok(())
}
