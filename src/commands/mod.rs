//! This module implements the nano command-line API.

use crate::lib::AnyhowResult;
use clap::Clap;
use tokio::runtime::Runtime;

mod neuron_manage;
mod neuron_stake;
mod public;
mod request_status;
mod send;
mod sign;
mod transfer;

pub use public::get_ids;

#[derive(Clap)]
pub enum Command {
    /// Prints the principal id and the accound id.
    PublicIds,
    Send(send::SendOpts),
    Transfer(transfer::TransferOpts),
    NeuronStake(neuron_stake::StakeOpts),
    NeuronManage(neuron_manage::ManageOpts),
}

pub fn exec(pem: &Option<String>, cmd: Command) -> AnyhowResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::PublicIds => public::exec(pem),
        Command::Transfer(opts) => runtime.block_on(async {
            transfer::exec(pem, opts).await.and_then(|out| {
                println!("{}", serde_json::to_string(&out)?);
                Ok(())
            })
        }),
        Command::NeuronStake(opts) => runtime.block_on(async {
            neuron_stake::exec(pem, opts).await.and_then(|out| {
                println!("{}", serde_json::to_string(&out)?);
                Ok(())
            })
        }),
        Command::NeuronManage(opts) => runtime.block_on(async {
            neuron_manage::exec(pem, opts).await.and_then(|out| {
                println!("{}", serde_json::to_string(&out)?);
                Ok(())
            })
        }),
        Command::Send(opts) => runtime.block_on(async { send::exec(pem, opts).await }),
    }
}
