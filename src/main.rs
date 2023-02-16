#![warn(unused_extern_crates)]
#![allow(special_module_name)]
use std::path::{Path, PathBuf};

use crate::lib::AnyhowResult;
use anyhow::Context;
use bip39::{Language, Mnemonic};
use clap::{crate_version, Args, Parser};
use lib::AuthInfo;

mod commands;
mod lib;

/// Ledger & Governance ToolKit for cold wallets.
#[derive(Parser)]
#[clap(name("quill"), version = crate_version!())]
pub struct CliOpts {
    #[clap(flatten, next_help_heading = "COMMON")]
    global_opts: GlobalOpts,
    #[clap(subcommand)]
    command: commands::Command,
}

#[derive(Args)]
struct GlobalOpts {
    /// Path to your PEM file (use "-" for STDIN)
    #[clap(long, group = "auth", global = true)]
    pem_file: Option<PathBuf>,

    #[clap(long, group = "auth", global = true)]
    hsm: bool,

    #[clap(long, global = true)]
    hsm_libpath: Option<PathBuf>,

    #[clap(long, global = true)]
    hsm_slot: Option<usize>,

    #[clap(long, global = true)]
    hsm_id: Option<String>,

    /// Path to your seed file (use "-" for STDIN)
    #[clap(long, global = true)]
    seed_file: Option<PathBuf>,

    /// Output the result(s) as UTF-8 QR codes.
    #[clap(long, global = true)]
    qr: bool,

    /// Fetches the root key before making requests so that interfacing with local instances is possible.
    /// DO NOT USE WITH ANY REAL INFORMATION
    #[clap(
        long = "insecure-local-dev-mode",
        name = "insecure-local-dev-mode",
        global = true
    )]
    fetch_root_key: bool,
}

fn main() -> AnyhowResult {
    let opts = CliOpts::parse();
    let qr = opts.global_opts.qr;
    let fetch_root_key = opts.global_opts.fetch_root_key;
    let auth = get_auth(opts.global_opts)?;
    commands::dispatch(&auth, opts.command, fetch_root_key, qr)?;
    Ok(())
}

fn get_auth(opts: GlobalOpts) -> AnyhowResult<AuthInfo> {
    // Get PEM from the file if provided, or try to convert from the seed file
    if opts.hsm {
        let mut hsm = lib::HSMInfo::new();
        if let Some(path) = opts.hsm_libpath {
            hsm.libpath = path;
        }
        if let Some(slot) = opts.hsm_slot {
            hsm.slot = slot;
        }
        if let Some(id) = opts.hsm_id {
            hsm.ident = id;
        }
        Ok(lib::AuthInfo::NitroHsm(hsm))
    } else {
        let pem = read_pem(opts.pem_file.as_deref(), opts.seed_file.as_deref())?;
        if let Some(pem) = pem {
            Ok(lib::AuthInfo::PemFile(pem))
        } else {
            Ok(lib::AuthInfo::NoAuth)
        }
    }
}

// Get PEM from the file if provided, or try to convert from the seed file
fn read_pem(pem_file: Option<&Path>, seed_file: Option<&Path>) -> AnyhowResult<Option<String>> {
    match (pem_file, seed_file) {
        (Some(pem_file), _) => read_file(pem_file, "PEM").map(Some),
        (_, Some(seed_file)) => {
            let seed = read_file(seed_file, "seed")?;
            let mnemonic = parse_mnemonic(&seed)?;
            let mnemonic = lib::mnemonic_to_pem(&mnemonic)?;
            Ok(Some(mnemonic))
        }
        _ => Ok(None),
    }
}

fn parse_mnemonic(phrase: &str) -> AnyhowResult<Mnemonic> {
    Mnemonic::from_phrase(phrase, Language::English)
        .context("Couldn't parse the seed phrase as a valid mnemonic. {:?}")
}

fn read_file(path: impl AsRef<Path>, name: &str) -> AnyhowResult<String> {
    let path = path.as_ref();
    if path == Path::new("-") {
        // read from STDIN
        let mut buffer = String::new();
        use std::io::Read;
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map(|_| buffer)
            .context(format!("Couldn't read {} from STDIN", name))
    } else {
        std::fs::read_to_string(path).with_context(|| format!("Couldn't read {} file", name))
    }
}

#[cfg(test)]
mod tests {
    use crate::read_pem;
    use bip39::{Language, Mnemonic};

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

        let res = read_pem(Some(pem_file.path()), None);

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
        let mnemonic =
            crate::lib::mnemonic_to_pem(&Mnemonic::from_phrase(phrase, Language::English).unwrap())
                .unwrap();

        let pem = read_pem(None, Some(seed_file.path()))
            .expect("Unable to read seed_file")
            .expect("None returned instead of Some");

        assert_eq!(mnemonic, pem);
    }

    #[test]
    fn test_read_pem_from_non_existing_file() {
        let dir = tempfile::tempdir().expect("Cannot create temp dir");
        let non_existing_file = dir.path().join("non_existing_pem_file");

        read_pem(Some(&non_existing_file), None).unwrap_err();

        read_pem(None, Some(&non_existing_file)).unwrap_err();
    }
}
