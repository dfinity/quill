use crate::lib::environment::Environment;
use crate::lib::DfxResult;
use clap::Clap;
use tokio::runtime::Runtime;

mod account_id;
mod neuron;
mod principal;
mod request_status_sign;
mod request_status_submit;
mod send;
mod sign;
mod transfer;

pub use principal::get_principal;

#[derive(Clap)]
pub enum Command {
    PrincipalId(principal::PrincipalIdOpts),
    Send(send::SendOpts),
    Sign(sign::SignOpts),
    AccountId(account_id::AccountIdOpts),
    Transfer(transfer::TransferOpts),
    StakeToNeuron(neuron::TransferOpts),
    RequestStatusSign(request_status_sign::RequestStatusSignOpts),
    RequestStatusSubmit(request_status_submit::RequestStatusSubmitOpts),
}

pub fn exec(env: &dyn Environment, cmd: Command) -> DfxResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::RequestStatusSign(v) => runtime.block_on(async {
            request_status_sign::exec(env, v).await.and_then(|out| {
                println!("{}", out);
                Ok(())
            })
        }),
        Command::RequestStatusSubmit(v) => {
            runtime.block_on(async { request_status_submit::exec(env, v).await })
        }
        Command::PrincipalId(v) => principal::exec(env, v),
        Command::AccountId(v) => runtime.block_on(async { account_id::exec(env, v).await }),
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
        Command::StakeToNeuron(v) => runtime.block_on(async {
            neuron::exec(env, v).await.and_then(|out| {
                println!("{}", out);
                Ok(())
            })
        }),
        Command::Send(v) => runtime.block_on(async { send::exec(env, v).await }),
    }
}
