use crate::{
    commands::sign::sign_ingress,
    lib::{governance_canister_id, sign::signed_message::Ingress, AnyhowResult},
};
use candid::{CandidType, Encode};

#[derive(CandidType)]
pub struct ListNeurons {
    pub neuron_ids: Vec<u64>,
    pub include_neurons_readable_by_caller: bool,
}

// We currently only support a subset of the functionality.
pub async fn exec(pem: &Option<String>) -> AnyhowResult<Vec<Ingress>> {
    let args = Encode!(&ListNeurons {
        neuron_ids: Vec::new(),
        include_neurons_readable_by_caller: true,
    })?;
    Ok(vec![
        sign_ingress(pem, governance_canister_id(), "list_neurons", args).await?,
    ])
}
