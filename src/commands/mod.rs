//! This module implements the command-line API.

use crate::lib::{qr, require_canister_ids, require_pem, AnyhowResult};
use anyhow::Context;
use clap::Parser;
use std::io::{self, Write};
use tokio::runtime::Runtime;

mod account_balance;
mod generate;
mod public;
mod qrcode;
mod request_status;
mod send;
mod transfer;

use crate::SnsCanisterIds;

#[derive(Parser)]
pub enum Command {
    /// Prints the principal id and the account id.
    PublicIds(public::PublicOpts),
    /// Queries a ledger account balance.
    AccountBalance(account_balance::AccountBalanceOpts),
    /// Signs a ledger transfer message to the provided 'to' account.
    Transfer(transfer::TransferOpts),
    /// Generate a mnemonic seed phrase and generate or recover PEM.
    Generate(generate::GenerateOpts),
    /// Print QR Scanner dapp QR code: scan to start dapp to submit QR results.
    ScannerQRCode,
    /// Print QR code for data e.g. principal id.
    QRCode(qrcode::QRCodeOpts),
    /// Sends signed messages to the Internet computer.
    Send(send::SendOpts),
}

pub fn exec(
    pem: &Option<String>,
    canister_ids: &Option<SnsCanisterIds>,
    qr: bool,
    cmd: Command,
) -> AnyhowResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::PublicIds(opts) => public::exec(pem, opts),
        Command::AccountBalance(opts) => {
            let canister_ids = require_canister_ids(canister_ids)?;
            runtime.block_on(async { account_balance::exec(&canister_ids, opts).await })
        }
        Command::Transfer(opts) => {
            let pem = require_pem(pem)?;
            let canister_ids = require_canister_ids(canister_ids)?;
            transfer::exec(&pem, &canister_ids, opts).and_then(|out| print_vec(qr, &out))
        }
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
            print_qr(&a, i != arg.len() - 1).context("Failed to print QR code")?;
        }
        Ok(())
    }
}
