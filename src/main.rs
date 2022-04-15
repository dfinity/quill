#![warn(unused_extern_crates)]

use crate::lib::AnyhowResult;
use anyhow::Context;
use bip39::Mnemonic;
use clap::{crate_version, Parser};
use ic_base_types::CanisterId;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

mod commands;
mod lib;

/// Cold wallet toolkit for interacting with a Service Nervous System's Ledger & Governance canisters.
#[derive(Parser)]
#[clap(name("sns-quill"), version = crate_version!())]
pub struct CliOpts {
    /// Path to your PEM file (use "-" for STDIN)
    #[clap(long)]
    pem_file: Option<String>,

    /// Path to your seed file (use "-" for STDIN)
    #[clap(long)]
    seed_file: Option<String>,

    /// Output the result(s) as UTF-8 QR codes.
    #[clap(long)]
    qr: bool,

    /// Path to the SNS cluster's canister ids
    #[clap(long)]
    canister_ids_file: Option<String>,

    #[clap(subcommand)]
    command: commands::Command,
}

#[derive(Debug, Clone)]
pub struct CanisterIds {
    pub governance_canister_id: CanisterId,
    pub ledger_canister_id: CanisterId,
    pub root_canister_id: CanisterId,
}

fn main() {
    let opts = CliOpts::parse();
    if let Err(err) = run(opts) {
        for (level, cause) in err.chain().enumerate() {
            if level == 0 {
                eprintln!("Error: {}", err);
                continue;
            }
            if level == 1 {
                eprintln!("Caused by:");
            }
            eprintln!("{:width$}{}", "", cause, width = level * 2);
        }
        std::process::exit(1);
    }
}

fn run(opts: CliOpts) -> AnyhowResult<()> {
    let pem = read_pem(opts.pem_file, opts.seed_file)?;
    let canister_ids = read_sns_canister_ids(opts.canister_ids_file)?;
    commands::exec(&pem, &canister_ids, opts.qr, opts.command)
}

// Get PEM from the file if provided, or try to convert from the seed file
fn read_pem(pem_file: Option<String>, seed_file: Option<String>) -> AnyhowResult<Option<String>> {
    match (pem_file, seed_file) {
        (Some(pem_file), _) => read_file(&pem_file, "PEM").map(Some),
        (_, Some(seed_file)) => {
            let seed = read_file(&seed_file, "seed")?;
            let mnemonic = parse_mnemonic(&seed)?;
            let mnemonic = lib::mnemonic_to_pem(&mnemonic)?;
            Ok(Some(mnemonic))
        }
        _ => Ok(None),
    }
}

fn read_sns_canister_ids(file_path: Option<String>) -> AnyhowResult<Option<CanisterIds>> {
    match file_path {
        Some(file_path) => {
            let path = PathBuf::from(file_path);
            let file = File::open(path).context("Could not open the SNS Canister Ids file")?;
            let ids: HashMap<String, String> = serde_json::from_reader(file)
                .context("Could not parse the SNS Canister Ids file")?;

            let governance_canister_id = ids
                .get("governance_canister_id")
                .map(|id| CanisterId::from_str(id))
                .expect("Could not parse governance_canister_id as CanisterId")?;

            let ledger_canister_id = ids
                .get("ledger_canister_id")
                .map(|id| CanisterId::from_str(id))
                .expect("Could not parse ledger_canister_id as CanisterId")?;

            let root_canister_id = ids
                .get("root_canister_id")
                .map(|id| CanisterId::from_str(id))
                .expect("Could not parse root_canister_id as CanisterId")?;

            Ok(Some(CanisterIds {
                governance_canister_id,
                ledger_canister_id,
                root_canister_id,
            }))
        }
        _ => Ok(None),
    }
}

fn parse_mnemonic(phrase: &str) -> AnyhowResult<Mnemonic> {
    Mnemonic::parse(phrase).context("Couldn't parse the seed phrase as a valid mnemonic. {:?}")
}

fn read_file(path: &str, name: &str) -> AnyhowResult<String> {
    match path {
        // read from STDIN
        "-" => {
            let mut buffer = String::new();
            use std::io::Read;
            std::io::stdin()
                .read_to_string(&mut buffer)
                .map(|_| buffer)
                .context(format!("Couldn't read {} from STDIN", name))
        }
        path => std::fs::read_to_string(path).context(format!("Couldn't read {} file", name)),
    }
}

#[test]
fn test_read_pem_none_none() {
    let res = read_pem(None, None);
    assert_eq!(None, res.expect("read_pem(None, None) failed"));
}

#[test]
fn test_read_pem_from_pem_file() {
    use std::io::Write;

    let mut pem_file = tempfile::NamedTempFile::new().expect("Cannot create temp file");

    let content = "pem".to_string();
    pem_file
        .write_all(content.as_bytes())
        .expect("Cannot write to temp file");

    let res = read_pem(Some(pem_file.path().to_str().unwrap().to_string()), None);

    assert_eq!(Some(content), res.expect("read_pem from pem file"));
}

#[test]
fn test_read_pem_from_seed_file() {
    use std::io::Write;

    let mut seed_file = tempfile::NamedTempFile::new().expect("Cannot create temp file");

    let phrase = "ozone drill grab fiber curtain grace pudding thank cruise elder eight about";
    seed_file
        .write_all(phrase.as_bytes())
        .expect("Cannot write to temp file");
    let mnemonic = lib::mnemonic_to_pem(&Mnemonic::parse(phrase).unwrap()).unwrap();

    let pem = read_pem(None, Some(seed_file.path().to_str().unwrap().to_string()))
        .expect("Unable to read seed_file")
        .expect("None returned instead of Some");

    assert_eq!(mnemonic, pem);
}

#[test]
fn test_read_pem_from_non_existing_file() {
    let dir = tempfile::tempdir().expect("Cannot create temp dir");
    let non_existing_file = dir
        .path()
        .join("non_existing_pem_file")
        .as_path()
        .to_str()
        .unwrap()
        .to_string();

    read_pem(Some(non_existing_file.clone()), None).unwrap_err();

    read_pem(None, Some(non_existing_file)).unwrap_err();
}
