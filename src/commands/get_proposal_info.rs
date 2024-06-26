use crate::{
    commands::{send::submit_unsigned_ingress, SendingOpts},
    lib::{governance_canister_id, AnyhowResult, ROLE_NNS_GOVERNANCE},
};
use candid::Encode;
use clap::Parser;

#[derive(Parser)]
pub struct GetProposalInfoOpts {
    /// The proposal identifier.
    pub ident: u64,

    #[command(flatten)]
    sending_opts: SendingOpts,
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
        opts.sending_opts,
        fetch_root_key,
    )
    .await
}
