use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use clap::Clap;

/// Removes an existing identity.
#[derive(Clap)]
pub struct RemoveOpts {
    /// The identity to remove.
    identity: String,
}

pub fn exec(env: &dyn Environment, opts: RemoveOpts) -> DfxResult {
    let name = opts.identity.as_str();
    println!(r#"Removing identity "{}"."#, name);
    IdentityManager::new(env)?.remove(name)?;
    println!(r#"Removed identity "{}"."#, name);
    Ok(())
}
