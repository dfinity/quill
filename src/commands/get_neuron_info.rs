use crate::{
    commands::{send::submit_unsigned_ingress, QueryOpts},
    lib::{governance_canister_id, AnyhowResult, ROLE_NNS_GOVERNANCE},
};
use candid::Encode;
use clap::Parser;

/// Queries for information about a neuron, such as its voting power and age.
#[derive(Parser)]
pub struct GetNeuronInfoOpts {
    /// The neuron identifier.
    pub ident: u64,

    #[clap(flatten)]
    pub query_opts: QueryOpts,
}

// We currently only support a subset of the functionality.
#[tokio::main]
pub async fn exec(opts: GetNeuronInfoOpts, fetch_root_key: bool) -> AnyhowResult {
    let args = Encode!(&opts.ident)?;
    submit_unsigned_ingress(
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "get_neuron_info",
        args,
        opts.query_opts,
        fetch_root_key,
    )
    .await
}
