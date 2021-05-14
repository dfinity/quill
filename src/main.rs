#![warn(unused_extern_crates)]
use clap::{crate_version, AppSettings, Clap};
mod commands;
mod lib;

/// Ledger & Governance ToolKit for cold wallets.
#[derive(Clap)]
#[clap(name("nano"), version = crate_version!(), global_setting = AppSettings::ColoredHelp)]
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
    if let Err(err) = commands::exec(&pem, command) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
