//! This module implements the command-line API.

use crate::{get_auth, lib::AnyhowResult, BaseOpts};
use anyhow::Context;
use clap::{Args, Parser};
use std::io::{self, Write};
use tokio::runtime::Runtime;

mod account_balance;
mod claim_neurons;
mod generate;
mod get_neuron_info;
mod get_proposal_info;
mod list_neurons;
mod list_proposals;
mod neuron_manage;
mod neuron_stake;
mod public;
mod qrcode;
mod replace_node_provide_id;
mod request_status;
mod send;
mod transfer;
mod update_node_provider;

pub use public::get_ids;

#[derive(Parser)]
pub enum Command {
    /// Prints the principal id and the account id.
    PublicIds(BaseOpts<public::PublicOpts>),
    Send(BaseOpts<send::SendOpts>),
    Transfer(BaseOpts<transfer::TransferOpts>),
    /// Claim seed neurons from the Genesis Token Canister.
    ClaimNeurons(BaseOpts<Empty>),
    NeuronStake(BaseOpts<neuron_stake::StakeOpts>),
    NeuronManage(BaseOpts<neuron_manage::ManageOpts>),
    /// Signs the query for all neurons belonging to the signing principal.
    ListNeurons(BaseOpts<list_neurons::ListNeuronsOpts>),
    ListProposals(BaseOpts<list_proposals::ListProposalsOpts>),
    GetProposalInfo(BaseOpts<get_proposal_info::GetProposalInfoOpts>),
    GetNeuronInfo(BaseOpts<get_neuron_info::GetNeuronInfoOpts>),
    /// Queries a ledger account balance.
    AccountBalance(BaseOpts<account_balance::AccountBalanceOpts>),
    /// Update node provider details
    UpdateNodeProvider(BaseOpts<update_node_provider::UpdateNodeProviderOpts>),
    ReplaceNodeProviderId(BaseOpts<replace_node_provide_id::ReplaceNodeProviderIdOpts>),
    /// Generate a mnemonic seed phrase and generate or recover PEM.
    Generate(BaseOpts<generate::GenerateOpts>),
    /// Print QR Scanner dapp QR code: scan to start dapp to submit QR results.
    ScannerQRCode,
    /// Print QR code for data e.g. principal id.
    QRCode(BaseOpts<qrcode::QRCodeOpts>),
}

#[derive(Args)]
pub struct Empty;

pub fn dispatch(cmd: Command) -> AnyhowResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::PublicIds(opts) => public::exec(&get_auth(opts.global_opts)?, opts.command_opts)?,
        Command::Transfer(opts) => {
            let qr = opts.global_opts.qr;
            let out = transfer::exec(&get_auth(opts.global_opts)?, opts.command_opts)?;
            print_vec(qr, &out)?;
        }
        Command::NeuronStake(opts) => {
            let qr = opts.global_opts.qr;
            let out = neuron_stake::exec(&get_auth(opts.global_opts)?, opts.command_opts)?;
            print_vec(qr, &out)?;
        }
        Command::NeuronManage(opts) => {
            let qr = opts.global_opts.qr;
            let out = neuron_manage::exec(&get_auth(opts.global_opts)?, opts.command_opts)?;
            print_vec(qr, &out)?;
        }
        Command::ListNeurons(opts) => {
            let qr = opts.global_opts.qr;
            let out = list_neurons::exec(&get_auth(opts.global_opts)?, opts.command_opts)?;
            print_vec(qr, &out)?;
        }
        Command::ClaimNeurons(opts) => {
            let qr = opts.global_opts.qr;
            claim_neurons::exec(&get_auth(opts.global_opts)?)
                .and_then(|out| print_vec(qr, &out))?;
        }
        Command::ListProposals(opts) => runtime.block_on(async {
            list_proposals::exec(opts.command_opts, opts.global_opts.fetch_root_key).await
        })?,
        Command::GetProposalInfo(opts) => runtime.block_on(async {
            get_proposal_info::exec(opts.command_opts, opts.global_opts.fetch_root_key).await
        })?,
        Command::GetNeuronInfo(opts) => runtime.block_on(async {
            get_neuron_info::exec(opts.command_opts, opts.global_opts.fetch_root_key).await
        })?,
        Command::AccountBalance(opts) => runtime.block_on(async {
            account_balance::exec(opts.command_opts, opts.global_opts.fetch_root_key).await
        })?,
        Command::UpdateNodeProvider(opts) => {
            let out = update_node_provider::exec(&get_auth(opts.global_opts)?, opts.command_opts)?;
            print(&out)?;
        }
        Command::ReplaceNodeProviderId(opts) => {
            let out =
                replace_node_provide_id::exec(&get_auth(opts.global_opts)?, opts.command_opts)?;
            print(&out)?;
        }
        Command::Send(opts) => runtime.block_on(async {
            send::exec(opts.command_opts, opts.global_opts.fetch_root_key).await
        })?,
        Command::Generate(opts) => generate::exec(opts.command_opts)?,
        // QR code for URL: https://p5deo-6aaaa-aaaab-aaaxq-cai.raw.ic0.app/
        // Source code: https://github.com/ninegua/ic-qr-scanner
        Command::ScannerQRCode => {
            println!(
                "\
█████████████████████████████████████
█████████████████████████████████████
████ ▄▄▄▄▄ █▀█ █▄▀▄▀▄█ ▄ █ ▄▄▄▄▄ ████
████ █   █ █▀▀▀█ ▀▀█▄▀████ █   █ ████
████ █▄▄▄█ █▀ █▀▀██▀▀█ ▄ █ █▄▄▄█ ████
████▄▄▄▄▄▄▄█▄▀ ▀▄█ ▀▄█▄█▄█▄▄▄▄▄▄▄████
████▄▄▄▄ ▀▄  ▄▀▄ ▄ █▀▄▀▀▀ ▀ ▀▄█▄▀████
████▄█  █ ▄█▀█▄▀█▄  ▄▄ █ █   ▀█▀█████
████▄▀ ▀ █▄▄▄ ▄   █▄▀   █ ▀▀▀▄▄█▀████
████▄██▀▄▀▄▄ █▀█ ▄▄▄▄███▄█▄▀ ▄▄▀█████
████ ▀▄▀▄█▄▀▄▄▄▀█ ▄▄▀▄▀▀▀▄▀▀▀▄ █▀████
████ █▀██▀▄██▀▄█ █▀  █▄█▄▀▀  █▄▀█████
████▄████▄▄▄  ▀▀█▄▄██▄▀█ ▄▄▄ ▀   ████
████ ▄▄▄▄▄ █▄▄██▀▄▀ ▄█▄  █▄█ ▄▄▀█████
████ █   █ █  █▀▄▄▀▄ ▄▀▀▄▄▄ ▄▀ ▄▀████
████ █▄▄▄█ █ █▄▀▄██ ██▄█▀ ▄█  ▄ █████
████▄▄▄▄▄▄▄█▄▄▄▄▄▄██▄▄█▄████▄▄▄██████
█████████████████████████████████████
█████████████████████████████████████"
            );
        }
        Command::QRCode(opts) => qrcode::exec(opts.command_opts)?,
    }
    Ok(())
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

fn print_qr<T>(arg: &T, pause: bool) -> AnyhowResult
where
    T: serde::ser::Serialize,
{
    let json = serde_json::to_string(&arg)?;
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(json.as_bytes()).unwrap();
    let json = e.finish().unwrap();
    let json = base64::encode(json);
    qrcode::print_qr(json.as_str());
    if pause {
        let mut input_string = String::new();
        std::io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");
    }
    Ok(())
}

fn print_vec<T>(qr: bool, arg: &[T]) -> AnyhowResult
where
    T: serde::ser::Serialize,
{
    if !qr {
        print(arg)
    } else {
        for (i, a) in arg.iter().enumerate() {
            print_qr(&a, i != arg.len() - 1).context("Failed to print QR code")?;
        }
        Ok(())
    }
}
