use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::icpts::ICPTs;
use crate::lib::provider::create_agent_environment;
use anyhow::anyhow;
use clap::Clap;
use std::str::FromStr;
use tokio::runtime::Runtime;
mod account_id;
mod transfer;

/// Ledger commands.
#[derive(Clap)]
#[clap(name("ledger"))]
pub struct LedgerOpts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    AccountId(account_id::AccountIdOpts),
    Transfer(transfer::TransferOpts),
}

pub fn exec(env: &dyn Environment, opts: LedgerOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, None)?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::AccountId(v) => account_id::exec(&agent_env, v).await,
            SubCommand::Transfer(v) => transfer::exec(&agent_env, v).await,
        }
    })
}

fn get_icpts_from_args(
    amount: Option<String>,
    icp: Option<String>,
    e8s: Option<String>,
) -> DfxResult<ICPTs> {
    if amount.is_none() {
        let icp = match icp {
            Some(s) => {
                // validated by e8s_validator
                let icps = s.parse::<u64>().unwrap();
                ICPTs::from_icpts(icps).map_err(|err| anyhow!(err))?
            }
            None => ICPTs::from_e8s(0),
        };
        let icp_from_e8s = match e8s {
            Some(s) => {
                // validated by e8s_validator
                let e8s = s.parse::<u64>().unwrap();
                ICPTs::from_e8s(e8s)
            }
            None => ICPTs::from_e8s(0),
        };
        let amount = icp + icp_from_e8s;
        Ok(amount.map_err(|err| anyhow!(err))?)
    } else {
        Ok(ICPTs::from_str(&amount.unwrap())
            .map_err(|err| anyhow!("Could not add ICPs and e8s: {}", err))?)
    }
}
