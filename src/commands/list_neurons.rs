use crate::{
    commands::sign::sign_ingress,
    lib::{governance_canister_id, sign::signed_message::Ingress, AnyhowResult},
};
use anyhow::anyhow;
use candid::{CandidType, Encode};
use clap::Clap;

#[derive(CandidType)]
pub struct ListNeurons {
    pub neuron_ids: Vec<u64>,
    pub include_neurons_readable_by_caller: bool,
}

/// Signs a neuron configuration change.
#[derive(Clap)]
pub struct ListOpts {
    /// List information about the given neuron ids
    #[clap(long)]
    neuron_ids: Vec<u64>,

    /// Whether to include such neurons
    #[clap(long)]
    include_neurons_readable_by_caller: bool,
}

pub async fn exec(pem: &Option<String>, opts: ListOpts) -> AnyhowResult<Vec<Ingress>> {
    let mut msg_args = Vec::new();

    if !opts.neuron_ids.is_empty() {
        let args = Encode!(&ListNeurons {
            neuron_ids: opts.neuron_ids,
            include_neurons_readable_by_caller: opts.include_neurons_readable_by_caller,
        })?;
        msg_args.push(args);
    };

    if msg_args.is_empty() {
        return Err(anyhow!("No instructions provided"));
    }

    let mut msgs = Vec::new();
    for args in msg_args {
        msgs.push(sign_ingress(pem, governance_canister_id(), "list_neurons", args).await?);
    }
    Ok(msgs)
}
