//! This module implements the command-line API.

use crate::lib::{AnyhowResult, AuthInfo};
use anyhow::Context;
use clap::Parser;
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
    PublicIds(public::PublicOpts),
    Send(send::SendOpts),
    Transfer(transfer::TransferOpts),
    /// Claim seed neurons from the Genesis Token Canister.
    ClaimNeurons,
    NeuronStake(neuron_stake::StakeOpts),
    NeuronManage(neuron_manage::ManageOpts),
    /// Signs the query for all neurons belonging to the signing principal.
    ListNeurons(list_neurons::ListNeuronsOpts),
    ListProposals(list_proposals::ListProposalsOpts),
    GetProposalInfo(get_proposal_info::GetProposalInfoOpts),
    GetNeuronInfo(get_neuron_info::GetNeuronInfoOpts),
    /// Queries a ledger account balance.
    AccountBalance(account_balance::AccountBalanceOpts),
    /// Update node provider details
    UpdateNodeProvider(update_node_provider::UpdateNodeProviderOpts),
    ReplaceNodeProviderId(replace_node_provide_id::ReplaceNodeProviderIdOpts),
    /// Generate a mnemonic seed phrase and generate or recover PEM.
    Generate(generate::GenerateOpts),
    /// Print QR Scanner dapp QR code: scan to start dapp to submit QR results.
    ScannerQRCode,
    /// Print QR code for data e.g. principal id.
    QRCode(qrcode::QRCodeOpts),
}

pub fn exec(auth: &AuthInfo, qr: bool, fetch_root_key: bool, cmd: Command) -> AnyhowResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::PublicIds(opts) => public::exec(auth, opts),
        Command::Transfer(opts) => transfer::exec(auth, opts).and_then(|out| print_vec(qr, &out)),
        Command::NeuronStake(opts) => {
            neuron_stake::exec(auth, opts).and_then(|out| print_vec(qr, &out))
        }
        Command::NeuronManage(opts) => {
            neuron_manage::exec(auth, opts).and_then(|out| print_vec(qr, &out))
        }
        Command::ListNeurons(opts) => {
            list_neurons::exec(auth, opts).and_then(|out| print_vec(qr, &out))
        }
        Command::ClaimNeurons => claim_neurons::exec(auth).and_then(|out| print_vec(qr, &out)),
        Command::ListProposals(opts) => {
            runtime.block_on(async { list_proposals::exec(opts, fetch_root_key).await })
        }
        Command::GetProposalInfo(opts) => {
            runtime.block_on(async { get_proposal_info::exec(opts, fetch_root_key).await })
        }
        Command::GetNeuronInfo(opts) => {
            runtime.block_on(async { get_neuron_info::exec(opts, fetch_root_key).await })
        }
        Command::AccountBalance(opts) => {
            runtime.block_on(async { account_balance::exec(opts, fetch_root_key).await })
        }
        Command::UpdateNodeProvider(opts) => {
            update_node_provider::exec(auth, opts).and_then(|out| print(&out))
        }
        Command::ReplaceNodeProviderId(opts) => {
            replace_node_provide_id::exec(auth, opts).and_then(|out| print(&out))
        }
        Command::Send(opts) => runtime.block_on(async { send::exec(opts, fetch_root_key).await }),
        Command::Generate(opts) => generate::exec(opts),
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
            Ok(())
        }
        Command::QRCode(opts) => qrcode::exec(opts),
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
