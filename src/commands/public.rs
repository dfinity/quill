use crate::lib::{get_identity, nns_types::account_identifier::AccountIdentifier, AnyhowResult};
use anyhow::anyhow;
use ic_types::principal::Principal;

pub fn exec(pem: &Option<String>) -> AnyhowResult {
    let (principal_id, account_id) = get_ids(pem)?;
    println!("Principal id: {}", principal_id.to_text());
    println!("Account id: {}", account_id);
    Ok(())
}

pub fn get_ids(pem: &Option<String>) -> AnyhowResult<(Principal, AccountIdentifier)> {
    let principal_id = get_identity(pem.as_ref().ok_or(anyhow!("No PEM file provided"))?)
        .sender()
        .map_err(|e| anyhow!(e))?;
    let account_id = AccountIdentifier::new(principal_id.clone(), None);
    Ok((principal_id, account_id))
}
