use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use clap::Clap;

/// Renames an existing identity.
#[derive(Clap)]
pub struct RenameOpts {
    /// The current name of the identity.
    from: String,

    /// The new name of the identity.
    to: String,
}

pub fn exec(env: &dyn Environment, opts: RenameOpts) -> DfxResult {
    let from = opts.from.as_str();
    let to = opts.to.as_str();

    println!(r#"Renaming identity "{}" to "{}"."#, from, to);

    let mut identity_manager = IdentityManager::new(env)?;
    let renamed_default = identity_manager.rename(env, from, to)?;

    println!(r#"Renamed identity "{}" to "{}"."#, from, to);
    if renamed_default {
        println!(r#"Now using identity: "{}"."#, to);
    }

    Ok(())
}
