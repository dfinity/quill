use crate::{
    commands::sign::sign_ingress,
    lib::{governance_canister_id, sign::signed_message::Ingress, AnyhowResult},
};
use candid::{CandidType, Encode};
use clap::Clap;

#[derive(CandidType)]
pub struct ListNeurons {
    pub neuron_ids: Vec<u64>,
    pub include_neurons_readable_by_caller: bool,
}

/// Signs a neuron configuration change.
#[derive(Clap)]
pub struct ListNeuronsOpts {
    /// The id of the neuron to manage.
    neuron_id: Vec<u64>,
}

// We currently only support a subset of the functionality.
pub async fn exec(pem: &Option<String>, opts: ListNeuronsOpts) -> AnyhowResult<Vec<Ingress>> {
    let args = Encode!(&ListNeurons {
        neuron_ids: opts.neuron_id.clone(),
        include_neurons_readable_by_caller: opts.neuron_id.is_empty(),
    })?;
    Ok(vec![
        sign_ingress(pem, governance_canister_id(), "list_neurons", args).await?,
    ])
}
