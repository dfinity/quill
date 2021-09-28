#![warn(unused_extern_crates)]
use clap::{crate_version, AppSettings, Clap};
mod commands;
mod lib;

/// Ledger & Governance ToolKit for cold wallets.
#[derive(Clap)]
#[clap(name("quill"), version = crate_version!(), global_setting = AppSettings::ColoredHelp)]
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
    let auth = if opts.hsm {
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
        lib::AuthInfo::NitroHsm(hsm)
    } else {
        match pem {
            Some(path) => lib::AuthInfo::PemFile(path),
            None => lib::AuthInfo::NoAuth,
        }
    };
    if let Err(err) = commands::exec(&auth, command) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
