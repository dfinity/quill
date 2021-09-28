#![warn(unused_extern_crates)]
use clap::{crate_version, AppSettings, Clap};
mod commands;
mod lib;

#[cfg(target_os = "macos")]
const PKCS11_LIBPATH: &str = "/Library/OpenSC/lib/pkcs11/opensc-pkcs11.so";
#[cfg(target_os = "linux")]
const PKCS11_LIBPATH: &str = "/usr/local/lib/opensc-pkcs11.so";
#[cfg(target_os = "windows")]
const PKCS11_LIBPATH: &str = "who-knows?";

/// Ledger & Governance ToolKit for cold wallets.
#[derive(Clap)]
#[clap(name("quill"), version = crate_version!(), global_setting = AppSettings::ColoredHelp)]
pub struct CliOpts {
    /// Path to your PEM file (use "-" for STDIN)
    #[clap(long)]
    pem_file: Option<String>,

    #[clap(subcommand)]
    command: commands::Command,
}

fn main() {
    let opts = CliOpts::parse();
    let command = opts.command;
    let pem = opts.pem_file.map(|path| match path.as_str() {
        // read from STDIN
        "-" => {
            let mut buffer = String::new();
            use std::io::Read;
            if let Err(err) = std::io::stdin().read_to_string(&mut buffer) {
                eprintln!("Couldn't read from STDIN: {:?}", err);
                std::process::exit(1);
            }
            buffer
        }
        path => std::fs::read_to_string(path).unwrap_or_else(|err| {
            eprintln!("Couldn't read PEM file: {:?}", err);
            std::process::exit(1);
        }),
    });
    let auth = match std::env::var("NITROHSM_PIN") {
        Ok(_) => lib::AuthInfo::NitroHsm(lib::HSMInfo {
            libpath: std::path::PathBuf::from(
                std::env::var("NITROHSM_LIBPATH").unwrap_or_else(|_| PKCS11_LIBPATH.to_string()),
            ),
            slot: std::env::var("NITROHSM_SLOT").map_or(0, |s| s.parse().unwrap()),
            ident: std::env::var("NITROHSM_ID").unwrap_or_else(|_| "01".to_string()),
        }),
        Err(_) => match pem {
            Some(path) => lib::AuthInfo::PemFile(path),
            None => lib::AuthInfo::NoAuth,
        },
    };
    if let Err(err) = commands::exec(&auth, command) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
