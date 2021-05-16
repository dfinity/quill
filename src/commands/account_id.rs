use crate::lib::environment::Environment;
use crate::lib::nns_types::account_identifier::AccountIdentifier;
use crate::lib::DfxResult;
use anyhow::anyhow;
use clap::Clap;

/// Prints the selected identity's AccountIdentifier.
#[derive(Clap)]
pub struct AccountIdOpts {}

pub async fn exec(env: &dyn Environment, _opts: AccountIdOpts) -> DfxResult {
    let sender = env
        .get_selected_identity_principal()
        .ok_or_else(|| anyhow!("No PEM provided"))?;
    println!("{}", AccountIdentifier::new(sender, None));
    Ok(())
}
