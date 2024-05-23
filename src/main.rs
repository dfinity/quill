#![warn(unused_extern_crates)]
#![allow(special_module_name)]
use std::io::{stdin, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use crate::lib::AnyhowResult;
use anyhow::{bail, Context};
use clap::{crate_version, ArgGroup, Args, Parser};
use dialoguer::Password;
use k256::SecretKey;
use lib::AuthInfo;
use pkcs8::DecodePrivateKey;
use ring::signature::Ed25519KeyPair;
use sec1::pem::PemLabel;

mod commands;
mod lib;

/// Ledger & Governance ToolKit for cold wallets.
#[derive(Parser)]
#[command(name("quill"), version = crate_version!(), help_expected = true)]
pub struct CliOpts {
    #[command(flatten, next_help_heading = "COMMON")]
    global_opts: GlobalOpts,
    #[command(subcommand)]
    command: commands::Command,
}

#[derive(Args)]
#[command(
    group(ArgGroup::new("pkcs11").multiple(true).conflicts_with_all(&["seeded", "ledgerhq"])),
    group(ArgGroup::new("seeded").multiple(true).conflicts_with_all(&["pkcs11", "ledgerhq"])),
    group(ArgGroup::new("ledgerhq").multiple(true).conflicts_with_all(&["seeded", "pkcs11"])),
    group(ArgGroup::new("auth").multiple(true)),
)]
struct GlobalOpts {
    /// Path to your PEM file (use "-" for STDIN)
    #[arg(long, groups = &["seeded", "auth"], global = true)]
    pem_file: Option<PathBuf>,

    /// If the PEM file is encrypted, read the password from this file (use "-" for STDIN)
    #[arg(long, groups = &["seeded", "auth"], global = true)]
    password_file: Option<PathBuf>,

    /// Use a hardware key to sign messages.
    #[cfg_attr(not(feature = "hsm"), arg(hide = true))]
    #[arg(long, groups = &["pkcs11", "auth"], global = true)]
    hsm: bool,

    /// Path to the PKCS#11 module to use.
    #[cfg_attr(not(feature = "hsm"), arg(hide = true))]
    #[cfg_attr(
        target_os = "windows",
        doc = r"Defaults to C:\Program Files\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll"
    )]
    #[cfg_attr(
        target_os = "macos",
        doc = "Defaults to /Library/OpenSC/lib/pkcs11/opensc-pkcs11.so"
    )]
    #[cfg_attr(
        target_os = "linux",
        doc = "Defaults to /usr/lib/x86_64-linux-gnu/opensc-pkcs11.so"
    )]
    #[arg(long, global = true, groups = &["pkcs11", "auth"])]
    hsm_libpath: Option<PathBuf>,

    /// The slot that the hardware key is in. If OpenSC is installed, `pkcs11-tool --list-slots`
    #[cfg_attr(not(feature = "hsm"), arg(hide = true))]
    #[arg(long, global = true, groups = &["pkcs11", "auth"])]
    hsm_slot: Option<usize>,

    /// The ID of the key to use. Consult your hardware key's documentation.
    #[cfg_attr(not(feature = "hsm"), arg(hide = true))]
    #[arg(long, global = true, groups = &["pkcs11", "auth"])]
    hsm_id: Option<String>,

    /// No longer supported, included for compatibility
    #[arg(long, hide = true)]
    seed_file: Option<PathBuf>,

    /// Authenticate using a Ledger hardware wallet.
    #[cfg_attr(not(feature = "ledger"), arg(hide = true))]
    #[arg(long, global = true, groups = &["ledgerhq", "auth"])]
    ledger: bool,

    /// Output the result(s) as UTF-8 QR codes.
    #[arg(long, global = true)]
    qr: bool,

    /// Fetches the root key before making requests so that interfacing with local instances is possible.
    /// DO NOT USE WITH ANY REAL INFORMATION
    #[arg(
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
    let auth = if let commands::Command::Generate(_) = &opts.command {
        AuthInfo::NoAuth
    } else {
        get_auth(opts.global_opts)?
    };
    commands::dispatch(&auth, opts.command, fetch_root_key, qr)?;
    Ok(())
}

fn get_auth(opts: GlobalOpts) -> AnyhowResult<AuthInfo> {
    if opts.hsm || opts.hsm_libpath.is_some() || opts.hsm_slot.is_some() || opts.hsm_id.is_some() {
        #[cfg(feature = "hsm")]
        {
            let mut hsm = lib::HSMInfo::new()?;
            if let Some(path) = opts.hsm_libpath {
                hsm.libpath = path;
            }
            if let Some(slot) = opts.hsm_slot {
                hsm.slot = slot;
            }
            if let Some(id) = opts.hsm_id {
                hsm.ident = id;
            }
            Ok(AuthInfo::Pkcs11Hsm(hsm))
        }
        #[cfg(not(feature = "hsm"))]
        {
            anyhow::bail!("This build of quill does not support HSM functionality.")
        }
    } else if opts.ledger {
        #[cfg(feature = "ledger")]
        {
            Ok(AuthInfo::Ledger)
        }
        #[cfg(not(feature = "ledger"))]
        {
            anyhow::bail!("This build of quill does not support Ledger functionality.")
        }
    } else if opts.pem_file.is_some() {
        pem_auth(opts)
    } else if opts.seed_file.is_some() {
        bail!("Seed phrases are not accepted by commands directly anymore. Use `quill generate --from-phrase`.");
    } else {
        Ok(AuthInfo::NoAuth)
    }
}

fn pem_auth(opts: GlobalOpts) -> AnyhowResult<AuthInfo> {
    let file = opts
        .pem_file
        .as_ref()
        .expect("pem_file needed for pem_auth");
    let pem = pem::parse_many(read_file(file, "PEM")?)?;
    for document in pem {
        match document.tag() {
            sec1::EcPrivateKey::PEM_LABEL => {
                return Ok(AuthInfo::K256Key(
                    SecretKey::from_sec1_der(document.contents()).with_context(|| {
                        format!("File {} was not a valid secp256k1 key", file.display())
                    })?,
                ))
            }
            "PRIVATE KEY" => {
                Ed25519KeyPair::from_pkcs8_maybe_unchecked(document.contents()).with_context(
                    || format!("File {} was not a valid Ed25519 key", file.display()),
                )?;
                return Ok(AuthInfo::Ed25519Key(document.into_contents()));
            }
            pkcs8::EncryptedPrivateKeyInfo::PEM_LABEL => {
                let password = if let Some(password_file) = &opts.password_file {
                    read_file(password_file, "password")?
                } else if stdin().is_terminal() {
                    Password::new()
                        .with_prompt("PEM decryption password:")
                        .interact()?
                } else {
                    bail!("Must use --password-file if PEM file is encrypted and stdin cannot receive terminal input.");
                };
                return Ok(AuthInfo::K256Key(
                    SecretKey::from_pkcs8_encrypted_der(document.contents(), password)
                        .with_context(|| {
                            format!("Could not read file {} as a secp256k1 key", file.display())
                        })?,
                ));
            }
            _ => {}
        }
    }
    bail!(
        "File {} contained no recognized key formats",
        file.display()
    );
}

fn read_file(path: impl AsRef<Path>, name: &str) -> AnyhowResult<String> {
    let path = path.as_ref();
    if path == Path::new("-") {
        // read from STDIN
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map(|_| buffer)
            .context(format!("Couldn't read {name} from STDIN"))
    } else {
        std::fs::read_to_string(path).with_context(|| format!("Couldn't read {name} file"))
    }
}

fn write_file(path: impl AsRef<Path>, name: &str, content: &[u8]) -> AnyhowResult {
    let path = path.as_ref();
    if path == Path::new("-") {
        // write to STDOUT
        std::io::stdout()
            .lock()
            .write_all(content)
            .context("Couldn't write {name} to STDOUT")
    } else {
        std::fs::write(path, content).with_context(|| format!("Couldn't write {name} file"))
    }
}

#[cfg(test)]
mod tests {
    use crate::CliOpts;
    use clap::CommandFactory;

    #[test]
    fn check_cli() {
        CliOpts::command().debug_assert()
    }
}
