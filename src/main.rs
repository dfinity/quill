#![warn(unused_extern_crates)]
use crate::lib::environment::EnvironmentImpl;

use clap::{AppSettings, Clap};

mod commands;
mod config;
mod lib;
mod util;

/// Ledger & Governance ToolKit.
#[derive(Clap)]
#[clap(name("nano"), global_setting = AppSettings::ColoredHelp)]
pub struct CliOpts {
    /// Path to your PEM file
    #[clap(long)]
    pem_file: Option<String>,

    #[clap(subcommand)]
    command: commands::Command,
}

fn main() {
    let opts = CliOpts::parse();
    let command = opts.command;
    let pem = opts
        .pem_file
        .map(|path| std::fs::read_to_string(path).expect("Couldn't read PEM file"));
    let env = EnvironmentImpl::new(pem).expect("Couldn't instantiate the environment");
    match commands::exec(&env, command) {
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(255);
        }
        _ => {}
    };
}
