use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Clap;
use tokio::runtime::Runtime;

mod identity;
mod ledger;
mod send;
mod sign;

#[derive(Clap)]
pub enum Command {
    Identity(identity::IdentityOpt),
    Ledger(ledger::LedgerOpts),
    Send(send::SendOpts),
    Sign(sign::SignOpts),
}

pub fn exec(env: &dyn Environment, cmd: Command) -> DfxResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::Identity(v) => identity::exec(env, v),
        Command::Send(v) => runtime.block_on(async { send::exec(env, v).await }),
        Command::Sign(v) => runtime.block_on(async { sign::exec(env, v).await }),
        Command::Ledger(v) => ledger::exec(env, v),
    }
}
