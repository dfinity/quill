use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Clap;
use tokio::runtime::Runtime;

mod account_id;
mod principal;
mod send;
mod sign;
mod transfer;

#[derive(Clap)]
pub enum Command {
    GetPrincipal(principal::GetPrincipalOpts),
    Send(send::SendOpts),
    Sign(sign::SignOpts),
    AccountId(account_id::AccountIdOpts),
    Transfer(transfer::TransferOpts),
}

pub fn exec(env: &dyn Environment, cmd: Command) -> DfxResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::GetPrincipal(v) => principal::exec(env, v),
        Command::Send(v) => runtime.block_on(async { send::exec(env, v).await }),
        Command::Sign(v) => runtime.block_on(async { sign::exec(env, v).await }),
        Command::AccountId(v) => runtime.block_on(async { account_id::exec(env, v).await }),
        Command::Transfer(v) => runtime.block_on(async { transfer::exec(env, v).await }),
    }
}
