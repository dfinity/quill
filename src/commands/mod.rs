//! This module implements the command-line API.

use crate::lib::{qr, require_pem, AnyhowResult};
use clap::Parser;
use std::io::{self, Write};
use tokio::runtime::Runtime;

mod account_balance;
mod claim_neurons;
mod generate;
mod get_proposal_info;
mod list_neurons;
mod list_proposals;
mod neuron_manage;
mod neuron_stake;
mod public;
mod qrcode;
mod request_status;
mod send;
mod transfer;

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
    /// Queries a ledger account balance.
    AccountBalance(account_balance::AccountBalanceOpts),
    /// Generate a mnemonic seed phrase and generate or recover PEM.
    Generate(generate::GenerateOpts),
    /// Print QR Scanner dapp QR code: scan to start dapp to submit QR results.
    ScannerQRCode,
    /// Print QR code for data e.g. principal id.
    QRCode(qrcode::QRCodeOpts),
}

pub fn exec(pem: &Option<String>, qr: bool, cmd: Command) -> AnyhowResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::PublicIds(opts) => public::exec(pem, opts),
        Command::Transfer(opts) => {
            let pem = require_pem(pem)?;
            transfer::exec(&pem, opts).and_then(|out| print_vec(qr, &out))
        }
        Command::NeuronStake(opts) => {
            let pem = require_pem(pem)?;
            neuron_stake::exec(&pem, opts).and_then(|out| print_vec(qr, &out))
        }
        Command::NeuronManage(opts) => {
            let pem = require_pem(pem)?;
            neuron_manage::exec(&pem, opts).and_then(|out| print_vec(qr, &out))
        }
        Command::ListNeurons(opts) => {
            let pem = require_pem(pem)?;
            list_neurons::exec(&pem, opts).and_then(|out| print_vec(qr, &out))
        }
        Command::ClaimNeurons => {
            let pem = require_pem(pem)?;
            claim_neurons::exec(&pem).and_then(|out| print_vec(qr, &out))
        }
        Command::ListProposals(opts) => {
            runtime.block_on(async { list_proposals::exec(opts).await })
        }
        Command::GetProposalInfo(opts) => {
            runtime.block_on(async { get_proposal_info::exec(opts).await })
        }
        Command::AccountBalance(opts) => {
            runtime.block_on(async { account_balance::exec(opts).await })
        }
        Command::Send(opts) => runtime.block_on(async { send::exec(opts).await }),
        Command::Generate(opts) => generate::exec(opts),
        // QR code for URL: https://p5deo-6aaaa-aaaab-aaaxq-cai.raw.ic0.app/
        // Source code: https://github.com/ninegua/ic-qr-scanner
        Command::ScannerQRCode => {
            println!(
                "█████████████████████████████████████
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
    qr::print_qr(json.as_str());
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
            print_qr(&a, i != arg.len() - 1).expect("print_qr");
        }
        Ok(())
    }
}
