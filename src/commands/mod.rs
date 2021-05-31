use crate::lib::environment::Environment;
use crate::lib::DfxResult;
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
    Sign(sign::SignOpts),
    Transfer(transfer::TransferOpts),
    NeuronStake(neuron_stake::StakeOpts),
    NeuronManage(neuron_manage::ManageOpts),
}

pub fn exec(env: &dyn Environment, cmd: Command) -> DfxResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::PublicIds => public::exec(env),
        Command::Transfer(v) => runtime.block_on(async {
            transfer::exec(env, v).await.and_then(|out| {
                println!("{}", out);
                Ok(())
            })
        }),
        Command::Sign(v) => runtime.block_on(async {
            sign::exec(env, v).await.and_then(|out| {
                println!("{}", out.buffer);
                Ok(())
            })
        }),
        Command::NeuronStake(v) => runtime.block_on(async {
            neuron_stake::exec(env, v).await.and_then(|out| {
                println!("{}", out);
                Ok(())
            })
        }),
        Command::NeuronManage(v) => runtime.block_on(async {
            neuron_manage::exec(env, v).await.and_then(|out| {
                println!("{}", out);
                Ok(())
            })
        }),
        Command::Send(v) => runtime.block_on(async { send::exec(env, v).await }),
    }
}
