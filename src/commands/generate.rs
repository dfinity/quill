use crate::{
    lib::{get_account_id, key_encryption_params, mnemonic_to_key, AnyhowResult},
    read_file,
};
use anyhow::{anyhow, bail, Context};
use bip39::{Language, Mnemonic};
use clap::{Parser, ValueEnum};
use dialoguer::Password;
use ic_agent::{identity::Secp256k1Identity, Identity};
use pkcs8::{EncodePrivateKey, EncryptedPrivateKeyInfo, PrivateKeyInfo};
use rand::{rngs::OsRng, thread_rng, RngCore};
use sec1::{pem::PemLabel, LineEnding};
use std::{
    io::{stdin, IsTerminal},
    path::PathBuf,
};

/// Generate a mnemonic seed phrase and generate or recover PEM.
#[derive(Parser, Debug)]
#[command(about, version, author)]
pub struct GenerateOpts {
    /// Number of words: 12 or 24.
    #[arg(long, default_value = "12")]
    words: u32,

    /// File to write the seed phrase to. If unspecified, it will be printed to the terminal.
    #[arg(long)]
    seed_file: Option<PathBuf>,

    /// File to write the PEM to.
    #[arg(long, default_value = "identity.pem")]
    pem_file: PathBuf,

    /// A seed phrase in quotes to use to generate the PEM file.
    #[arg(long)]
    phrase: Option<String>,

    /// Overwrite any existing seed file.
    #[arg(long)]
    overwrite_seed_file: bool,

    /// Overwrite any existing PEM file.
    #[arg(long)]
    overwrite_pem_file: bool,

    /// Change how PEM files are stored.
    #[arg(long, value_enum, default_value_t = StorageMode::PasswordProtected)]
    storage_mode: StorageMode,

    /// Read the encryption password from this file. Use "-" for STDIN. Required if STDIN is being piped.
    #[arg(long)]
    password_file: Option<PathBuf>,
}

#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq)]
enum StorageMode {
    PasswordProtected,
    Plaintext,
}

/// Generate or recover mnemonic seed phrase and/or PEM file.
pub fn exec(opts: GenerateOpts) -> AnyhowResult {
    if let Some(seed_file) = &opts.seed_file {
        if !opts.overwrite_seed_file && seed_file.exists() {
            bail!(
                "Seed file {} exists and overwrite is not set.",
                seed_file.display()
            );
        }
    }
    if !opts.overwrite_pem_file && opts.pem_file.exists() {
        bail!(
            "PEM file {} exists and overwrite is not set.",
            opts.pem_file.display()
        );
    }
    if opts.storage_mode == StorageMode::PasswordProtected
        && opts.password_file.is_none()
        && !stdin().is_terminal()
    {
        bail!("Must use --password-file if using --storage-mode=password-protected and stdin cannot receive terminal input.");
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
    let key = mnemonic_to_key(&mnemonic).context("Failed to convert mnemonic to PEM")?;
    let phrase = mnemonic.into_phrase();
    if let Some(seed_file) = opts.seed_file {
        std::fs::write(&seed_file, phrase)?;
        println!("Written seed file to {}.", seed_file.display());
        println!("Copy the contents of this file to external media or a piece of paper and store it in a safe place.");
        if opts.storage_mode != StorageMode::Plaintext {
            println!("Be sure to delete the file afterwards, as it is not password-protected like the PEM file is.")
        }
    } else {
        println!(
            "\
Seed phrase: {phrase}
Copy this onto a piece of paper or external media and store it in a safe place."
        );
    }
    let pem = match opts.storage_mode {
        StorageMode::Plaintext => key.to_sec1_pem(LineEnding::default())?,
        StorageMode::PasswordProtected => {
            let password = if let Some(password_file) = opts.password_file {
                read_file(password_file, "password")?
            } else {
                Password::new()
                    .with_prompt("PEM encryption password")
                    .with_confirmation("Re-enter password", "Passwords did not match")
                    .interact()?
            };
            let key_der = key.to_pkcs8_der()?;
            let pki = PrivateKeyInfo::try_from(key_der.as_bytes())?;
            let mut rng = thread_rng();
            let mut salt = [0u8; 16];
            rng.fill_bytes(&mut salt);
            let mut iv = [0u8; 16];
            rng.fill_bytes(&mut iv);
            let doc = pki.encrypt_with_params(key_encryption_params(&salt, &iv), password)?;
            doc.to_pem(EncryptedPrivateKeyInfo::PEM_LABEL, LineEnding::default())?
        }
    };
    std::fs::write(&opts.pem_file, &pem)?;
    println!("Written PEM file to {}.", opts.pem_file.display());
    let principal_id = Secp256k1Identity::from_private_key(key)
        .sender()
        .map_err(|s| anyhow!(s))?;
    let account_id = get_account_id(principal_id, None)?;

    println!("Principal id: {principal_id}");
    println!("Legacy account id: {account_id}");
    Ok(())
}
