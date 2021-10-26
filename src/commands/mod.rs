//! This module implements the command-line API.

use crate::lib::{require_pem, AnyhowResult};
use clap::Clap;
use std::io::{self, Write};
use tokio::runtime::Runtime;

mod account_balance;
mod get_proposal_info;
mod list_neurons;
mod list_proposals;
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
    /// Prints the principal id and the account id.
    PublicIds(public::PublicOpts),
    Send(send::SendOpts),
    Transfer(transfer::TransferOpts),
    NeuronStake(neuron_stake::StakeOpts),
    NeuronManage(neuron_manage::ManageOpts),
    /// Signs the query for all neurons belonging to the signin principal.
    ListNeurons(list_neurons::ListNeuronsOpts),
    ListProposals(list_proposals::ListProposalsOpts),
    GetProposalInfo(get_proposal_info::GetProposalInfoOpts),
    /// Queries a ledger account balance
    AccountBalance(account_balance::AccountBalanceOpts),
}

pub fn exec(pem: &Option<String>, cmd: Command) -> AnyhowResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::PublicIds(opts) => public::exec(pem, opts),
        Command::Transfer(opts) => {
            require_pem(pem)?;
            runtime.block_on(async { transfer::exec(pem, opts).await.and_then(|out| print(&out)) })
        }
        Command::NeuronStake(opts) => runtime.block_on(async {
            require_pem(pem)?;
            neuron_stake::exec(pem, opts)
                .await
                .and_then(|out| print(&out))
        }),
        Command::NeuronManage(opts) => runtime.block_on(async {
            require_pem(pem)?;
            neuron_manage::exec(pem, opts)
                .await
                .and_then(|out| print(&out))
        }),
        Command::ListNeurons(opts) => runtime.block_on(async {
            require_pem(pem)?;
            list_neurons::exec(pem, opts)
                .await
                .and_then(|out| print(&out))
        }),
        Command::ListProposals(opts) => {
            runtime.block_on(async { list_proposals::exec(opts).await.and_then(|out| print(&out)) })
        }
        Command::GetProposalInfo(opts) => runtime.block_on(async {
            get_proposal_info::exec(opts)
                .await
                .and_then(|out| print(&out))
        }),
        Command::AccountBalance(opts) => {
            runtime.block_on(async { account_balance::exec(opts).await })
        }
        Command::Send(opts) => runtime.block_on(async { send::exec(opts).await }),
    }
}

// Using println! for printing to STDOUT and piping it to other tools leads to
// the problem that when the other tool closes its stream, the println! macro
// panics on the error and the whole binary crashes. This function provides a
// graceful handling of the error.
fn print<T>(arg: &T) -> AnyhowResult
where
    T: ?Sized + serde::ser::Serialize,
{
    if let Err(e) = io::stdout().write_all(serde_json::to_string(&arg)?.as_bytes()) {
        if e.kind() != std::io::ErrorKind::BrokenPipe {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}
