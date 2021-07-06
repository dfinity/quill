use crate::{
    commands::sign::sign_ingress_with_request_status_query,
    lib::{governance_canister_id, sign::signed_message::IngressWithRequestId, AnyhowResult},
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

pub async fn exec(pem: &Option<String>, opts: ListOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let mut msgs = Vec::new();

    if !opts.neuron_ids.is_empty() {
        let args = Encode!(&ListNeurons {
            neuron_ids: opts.neuron_ids,
            include_neurons_readable_by_caller: opts.include_neurons_readable_by_caller,
        })?;
        msgs.push(args);
    };

    if msgs.is_empty() {
        return Err(anyhow!("No instructions provided"));
    }

    let mut generated = Vec::new();
    for args in msgs {
        generated.push(
            sign_ingress_with_request_status_query(pem, governance_canister_id(), "list_neurons", args)
                .await?,
        );
    }
    Ok(generated)
}
