#![warn(unused_extern_crates)]
use crate::lib::environment::EnvironmentImpl;

use clap::{AppSettings, Clap};

mod commands;
mod lib;
mod util;

/// Ledger & Governance ToolKit.
#[derive(Clap)]
#[clap(name("nano"), global_setting = AppSettings::ColoredHelp)]
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
            std::io::stdin()
                .read_to_string(&mut buffer)
                .expect("Couldn't read from STDIN");
            buffer
        }
        path => std::fs::read_to_string(path).expect("Couldn't read PEM file"),
    });
    let env = EnvironmentImpl::new(pem).expect("Couldn't instantiate the environment");
    match commands::exec(&env, command) {
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(255);
        }
        _ => {}
    };
}
