//! This module implements the command-line API.

use crate::lib::{get_principal, AnyhowResult, AuthInfo, ParsedAccount, ParsedSubaccount};
use anyhow::{bail, Context};
use clap::{Args, Parser};
use icrc_ledger_types::icrc1::account::Account;
use std::io::{self, Write};

mod account_balance;
mod ckbtc;
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
mod replace_node_provider_id;
mod request_status;
mod send;
mod sns;
mod transfer;
mod update_node_provider;

#[derive(Parser)]
pub enum Command {
    PublicIds(public::PublicOpts),
    Send(send::SendOpts),
    Transfer(transfer::TransferOpts),
    ClaimNeurons(claim_neurons::ClaimNeuronOpts),
    NeuronStake(neuron_stake::StakeOpts),
    NeuronManage(neuron_manage::ManageOpts),
    ListNeurons(list_neurons::ListNeuronsOpts),
    ListProposals(list_proposals::ListProposalsOpts),
    GetProposalInfo(get_proposal_info::GetProposalInfoOpts),
    GetNeuronInfo(get_neuron_info::GetNeuronInfoOpts),
    AccountBalance(account_balance::AccountBalanceOpts),
    UpdateNodeProvider(update_node_provider::UpdateNodeProviderOpts),
    ReplaceNodeProviderId(replace_node_provider_id::ReplaceNodeProviderIdOpts),
    #[clap(subcommand)]
    Ckbtc(ckbtc::CkbtcCommand),
    Sns(sns::SnsOpts),
    Generate(generate::GenerateOpts),
    /// Print QR Scanner dapp QR code: scan to start dapp to submit QR results.
    ScannerQRCode,
    QRCode(qrcode::QRCodeOpts),
}

pub fn dispatch(auth: &AuthInfo, cmd: Command, fetch_root_key: bool, qr: bool) -> AnyhowResult {
    match cmd {
        Command::PublicIds(opts) => public::exec(auth, opts)?,
        Command::Transfer(opts) => {
            let out = transfer::exec(auth, opts)?;
            print_vec(qr, &out)?;
        }
        Command::NeuronStake(opts) => {
            let out = neuron_stake::exec(auth, opts)?;
            print_vec(qr, &out)?;
        }
        Command::NeuronManage(opts) => {
            let out = neuron_manage::exec(auth, opts)?;
            print_vec(qr, &out)?;
        }
        Command::ListNeurons(opts) => {
            let out = list_neurons::exec(auth, opts)?;
            print_vec(qr, &out)?;
        }
        Command::ClaimNeurons(_) => {
            claim_neurons::exec(auth).and_then(|out| print_vec(qr, &out))?;
        }
        Command::ListProposals(opts) => {
            list_proposals::exec(opts, fetch_root_key)?;
        }
        Command::GetProposalInfo(opts) => {
            get_proposal_info::exec(opts, fetch_root_key)?;
        }
        Command::GetNeuronInfo(opts) => {
            get_neuron_info::exec(opts, fetch_root_key)?;
        }
        Command::AccountBalance(opts) => {
            account_balance::exec(auth, opts, fetch_root_key)?;
        }
        Command::UpdateNodeProvider(opts) => {
            let out = update_node_provider::exec(auth, opts)?;
            print(&out)?;
        }
        Command::ReplaceNodeProviderId(opts) => {
            let out = replace_node_provider_id::exec(auth, opts)?;
            print(&out)?;
        }
        Command::Send(opts) => {
            send::exec(opts, fetch_root_key)?;
        }
        Command::Generate(opts) => generate::exec(opts)?,
        Command::Ckbtc(subcmd) => ckbtc::dispatch(auth, subcmd, qr, fetch_root_key)?,
        Command::Sns(opts) => sns::dispatch(auth, opts, qr, fetch_root_key)?,
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
        Command::QRCode(opts) => qrcode::exec(opts)?,
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
            eprintln!("{e}");
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
    qrcode::print_qr(json.as_str())?;
    if pause {
        let mut input_string = String::new();
        std::io::stdin()
            .read_line(&mut input_string)
            .context("Failed to read line")?;
    }
    Ok(())
}

fn print_vec<T>(qr: bool, arg: &[T]) -> AnyhowResult
where
    T: serde::ser::Serialize,
{
    if qr {
        for (i, a) in arg.iter().enumerate() {
            print_qr(&a, i != arg.len() - 1).context("Failed to print QR code")?;
        }
        Ok(())
    } else {
        print(arg)
    }
}

fn get_account(
    auth: Option<&AuthInfo>,
    account: Option<ParsedAccount>,
    subaccount: Option<ParsedSubaccount>,
) -> AnyhowResult<Account> {
    let mut account = if let Some(acct) = account {
        acct.0
    } else if let Some(auth) = auth {
        let principal = get_principal(auth)?;
        Account {
            owner: principal,
            subaccount: None,
        }
    } else {
        bail!("neither auth nor account present")
    };
    if let Some(subaccount) = subaccount {
        account.subaccount = Some(subaccount.0 .0);
    }
    Ok(account)
}

#[derive(Args)]
pub struct SendingOpts {
    /// Skips confirmation and sends the message directly.
    #[clap(long, short)]
    yes: bool,

    /// Will display the query, but not send it.
    #[clap(long)]
    dry_run: bool,

    /// Always displays the response in IDL format.
    #[clap(long)]
    raw: bool,
}
