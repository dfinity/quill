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
    #[clap(long)]
    pem_file: Option<String>,

    #[clap(subcommand)]
    command: commands::Command,
}

fn main() {
    let opts = CliOpts::parse();
    let command = opts.command;
    let env = EnvironmentImpl::new(std::path::PathBuf::from(
        opts.pem_file.unwrap_or("/dev/null".to_string()),
    ))
    .expect("Couldn't instantiate the environment");
    match commands::exec(&env, command) {
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(255);
        }
        _ => {}
    };
}
