use crate::lib::environment::Environment;
use crate::lib::nns_types::account_identifier::AccountIdentifier;
use crate::lib::AnyhowResult;
use anyhow::anyhow;
use ic_types::principal::Principal;

pub fn exec(env: &dyn Environment) -> AnyhowResult {
    let (principal_id, account_id) = get_ids(env)?;
    println!("Principal id: {}", principal_id.to_text());
    println!("Account id: {}", account_id);
    Ok(())
}

pub fn get_ids(env: &dyn Environment) -> AnyhowResult<(Principal, AccountIdentifier)> {
    let identity = env.identity().ok_or_else(|| anyhow!("No PEM provided"))?;
    let principal_id = identity.sender().map_err(|err| anyhow!("{}", err))?;
    let account_id = AccountIdentifier::new(principal_id.clone(), None);
    Ok((principal_id, account_id))
}
