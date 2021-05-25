use crate::lib::environment::Environment;
use crate::lib::identity::Identity as NanoIdentity;
use crate::lib::DfxResult;
use anyhow::anyhow;
use clap::Clap;
use ic_agent::identity::Identity;
use ic_types::Principal;

/// Prints the principal id.
#[derive(Clap)]
pub struct PrincipalIdOpts {}

pub fn exec(env: &dyn Environment, _opts: PrincipalIdOpts) -> DfxResult {
    let principal_id = get_principal(env)?;
    println!("{}", principal_id.to_text());
    Ok(())
}

pub fn get_principal(env: &dyn Environment) -> DfxResult<Principal> {
    let identity = NanoIdentity::load(env.get_pem().ok_or_else(|| anyhow!("No PEM provided"))?);
    identity.sender().map_err(|err| anyhow!("{}", err))
}
