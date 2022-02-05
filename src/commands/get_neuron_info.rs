use crate::{
    commands::send::submit_unsigned_ingress,
    lib::{governance_canister_id, AnyhowResult},
};
use candid::Encode;
use clap::Parser;

#[derive(Parser)]
pub struct GetNeuronInfoOpts {
    pub ident: u64,

    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,
}

// We currently only support a subset of the functionality.
pub async fn exec(opts: GetNeuronInfoOpts) -> AnyhowResult {
    let args = Encode!(&opts.ident)?;
    submit_unsigned_ingress(
        governance_canister_id(),
        "get_neuron_info",
        args,
        opts.dry_run,
    )
    .await
}
