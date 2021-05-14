use crate::config::dfx_version_str;
use crate::lib::environment::EnvironmentImpl;

use clap::{AppSettings, Clap};

mod commands;
mod config;
mod lib;
mod util;

/// The DFINITY Executor.
#[derive(Clap)]
#[clap(name("dfx"), version = dfx_version_str(), global_setting = AppSettings::ColoredHelp)]
pub struct CliOpts {
    #[clap(long)]
    identity: Option<String>,

    #[clap(subcommand)]
    command: commands::Command,
}

fn main() {
    let cli_opts = CliOpts::parse();
    let identity = cli_opts.identity;
    let command = cli_opts.command;
    let result = match EnvironmentImpl::new() {
        Ok(_) => match EnvironmentImpl::new().map(|env| env.with_identity_override(identity)) {
            Ok(env) => commands::exec(&env, command),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    };
    if let Err(err) = result {
        eprintln!("{}", err);

        std::process::exit(255);
    }
}
