use crate::{
    commands::sign::sign_ingress,
    lib::{governance_canister_id, sign::signed_message::Ingress, AnyhowResult, AuthInfo},
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
    /// The optional ids of the specific neuron to query. Note that these ids
    /// may only be those that occur in the usual output from `list-neurons`,
    /// i.e., they should be ids of the user's own neurons. The purpose of
    /// this option is to narrow the query, and not to allow querying of
    /// arbtirary neuron ids.
    neuron_id: Vec<u64>,
}

// We currently only support a subset of the functionality.
pub async fn exec(auth: &AuthInfo, opts: ListNeuronsOpts) -> AnyhowResult<Vec<Ingress>> {
    let args = Encode!(&ListNeurons {
        neuron_ids: opts.neuron_id.clone(),
        include_neurons_readable_by_caller: opts.neuron_id.is_empty(),
    })?;
    Ok(vec![
        sign_ingress(auth, governance_canister_id(), "list_neurons", args).await?,
    ])
}
