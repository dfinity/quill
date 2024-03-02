use crate::lib::{get_account_id, mnemonic_to_pem, AnyhowResult, AuthInfo};
use anyhow::{anyhow, Context};
use bip39::{Language, Mnemonic};
use clap::Parser;
use rand::{rngs::OsRng, RngCore};
use std::path::PathBuf;

/// Generate a mnemonic seed phrase and generate or recover PEM.
#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct GenerateOpts {
    /// Number of words: 12 or 24.
    #[clap(long, default_value = "12")]
    words: u32,

    /// File to write the seed phrase to.
    #[clap(long, default_value = "seed.txt")]
    seed_file: PathBuf,

    /// File to write the PEM to.
    #[clap(long)]
    pem_file: Option<PathBuf>,

    /// A seed phrase in quotes to use to generate the PEM file.
    #[clap(long)]
    phrase: Option<String>,

    /// Overwrite any existing seed file.
    #[clap(long)]
    overwrite_seed_file: bool,

    /// Overwrite any existing PEM file.
    #[clap(long)]
    overwrite_pem_file: bool,
}

/// Generate or recover mnemonic seed phrase and/or PEM file.
pub fn exec(opts: GenerateOpts) -> AnyhowResult {
    if opts.seed_file.exists() && !opts.overwrite_seed_file {
        return Err(anyhow!("Seed file exists and overwrite is not set."));
    }
    if let Some(path) = &opts.pem_file {
        if path.exists() && !opts.overwrite_pem_file {
            return Err(anyhow!("PEM file exists and overwrite is not set."));
        }
    }
    let bytes = match opts.words {
        12 => 16,
        24 => 32,
        _ => return Err(anyhow!("Words must be 12 or 24.")),
    };
    let mnemonic = match opts.phrase {
        Some(phrase) => {
            Mnemonic::from_phrase(&phrase, Language::English).context("Failed to parse mnemonic")?
        }
        None => {
            let mut key = vec![0u8; bytes];
            OsRng.fill_bytes(&mut key);
            Mnemonic::from_entropy(&key, Language::English).unwrap()
        }
    };
    let pem = mnemonic_to_pem(&mnemonic).context("Failed to convert mnemonic to PEM")?;
    let mut phrase = mnemonic.into_phrase();
    phrase.push('\n');
    std::fs::write(opts.seed_file, phrase)?;
    if let Some(path) = opts.pem_file {
        std::fs::write(path, &pem)?;
    }
    let principal_id = crate::lib::get_principal(&AuthInfo::PemFile(pem))?;
    let account_id = get_account_id(principal_id, None)?;
    println!("Principal id: {principal_id}");
    println!("Legacy account id: {account_id}");
    Ok(())
}
