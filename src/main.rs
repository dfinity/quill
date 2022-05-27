#![warn(unused_extern_crates)]
use crate::lib::AnyhowResult;
use anyhow::{Context, Error};
use bip39::Mnemonic;
use clap::{crate_version, Parser};

mod commands;
mod lib;

/// Ledger & Governance ToolKit for cold wallets.
#[derive(Parser)]
#[clap(name("quill"), version = crate_version!())]
pub struct CliOpts {
    /// Path to your PEM file (use "-" for STDIN)
    #[clap(long)]
    pem_file: Option<String>,

    #[clap(long)]
    hsm: bool,

    #[clap(long)]
    hsm_libpath: Option<String>,

    #[clap(long)]
    hsm_slot: Option<usize>,

    #[clap(long)]
    hsm_id: Option<String>,

    /// Path to your seed file (use "-" for STDIN)
    #[clap(long)]
    seed_file: Option<String>,

    /// Output the result(s) as UTF-8 QR codes.
    #[clap(long)]
    qr: bool,

    #[clap(subcommand)]
    command: commands::Command,
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
    // Get PEM from the file if provided, or try to convert from the seed file
    let auth: lib::AuthInfo = if opts.hsm {
        let mut hsm = lib::HSMInfo::new();
        if let Some(path) = opts.hsm_libpath {
            hsm.libpath = std::path::PathBuf::from(path);
        }
        if let Some(slot) = opts.hsm_slot {
            hsm.slot = slot;
        }
        if let Some(id) = opts.hsm_id {
            hsm.ident = id;
        }
        Ok::<_, Error>(lib::AuthInfo::NitroHsm(hsm))
    } else {
        let pem = read_pem(opts.pem_file, opts.seed_file)?;
        if let Some(pem) = pem {
            Ok(lib::AuthInfo::PemFile(pem))
        } else {
            Ok(lib::AuthInfo::NoAuth)
        }
    }?;

    commands::exec(&auth, opts.qr, opts.command)
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

#[cfg(test)]
mod tests {
    use crate::read_pem;
    use bip39::Mnemonic;

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
        let mnemonic = crate::lib::mnemonic_to_pem(&Mnemonic::parse(phrase).unwrap()).unwrap();

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
}
