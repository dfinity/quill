use crate::{
    commands::send::submit_unsigned_ingress,
    lib::{governance_canister_id, AnyhowResult, ROLE_NNS_GOVERNANCE},
};
use candid::Encode;
use clap::Parser;

#[derive(Parser)]
pub struct GetProposalInfoOpts {
    /// The proposal identifier.
    pub ident: u64,

    /// Skips confirmation and sends the message directly.
    #[clap(long)]
    yes: bool,

    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,
}

// We currently only support a subset of the functionality.
#[tokio::main]
pub async fn exec(opts: GetProposalInfoOpts, fetch_root_key: bool) -> AnyhowResult {
    let args = Encode!(&opts.ident)?;
    submit_unsigned_ingress(
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "get_proposal_info",
        args,
        opts.yes,
        opts.dry_run,
        fetch_root_key,
    )
    .await
}
