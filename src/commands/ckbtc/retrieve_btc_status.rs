use candid::Encode;
use clap::Parser;
use ic_ckbtc_minter::queries::RetrieveBtcStatusRequest;

use crate::{
    commands::send::submit_unsigned_ingress,
    lib::{ckbtc_minter_canister_id, AnyhowResult},
};

/// Sends a message to check the status of a ckBTC-to-BTC conversion.
///
/// This conversion can be performed with the `quill ckbtc retrieve-btc` command.
#[derive(Parser)]
pub struct RetrieveBtcStatusOpts {
    /// The block index to check.
    block_index: u64,
    /// Will display the signed message, but not send it.
    #[clap(long)]
    dry_run: bool,
    /// Skips confirmation and sends the message immediately.
    #[clap(long, short)]
    yes: bool,
    /// Uses ckTESTBTC instead of ckBTC.
    #[clap(long)]
    testnet: bool,
}

#[tokio::main]
pub async fn exec(opts: RetrieveBtcStatusOpts, fetch_root_key: bool) -> AnyhowResult {
    let args = RetrieveBtcStatusRequest {
        block_index: opts.block_index,
    };
    submit_unsigned_ingress(
        ckbtc_minter_canister_id(opts.testnet),
        "retrieve_btc_status",
        Encode!(&args)?,
        opts.yes,
        opts.dry_run,
        fetch_root_key,
    )
    .await?;
    Ok(())
}
