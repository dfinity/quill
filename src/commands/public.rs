use crate::lib::{get_identity, AnyhowResult};
use anyhow::anyhow;
use ic_base_types::PrincipalId;
use ic_types::principal::Principal;
use ledger_canister::AccountIdentifier;
use std::convert::TryFrom;

/// Prints the account and the principal ids.
pub fn exec(pem: &Option<String>) -> AnyhowResult {
    let (principal_id, account_id) = get_ids(pem)?;
    println!("Principal id: {}", principal_id.to_text());
    println!("Account id: {}", account_id);
    Ok(())
}

/// Returns the account id and the principal id if the private key was provided.
pub fn get_ids(pem: &Option<String>) -> AnyhowResult<(Principal, AccountIdentifier)> {
    let principal_id = get_identity(
        pem.as_ref()
            .ok_or_else(|| anyhow!("No PEM file provided"))?,
    )
    .sender()
    .map_err(|e| anyhow!(e))?;
    let base_types_principal =
        PrincipalId::try_from(principal_id.as_slice()).map_err(|err| anyhow!(err))?;
    let account_id = AccountIdentifier::new(base_types_principal, None);
    Ok((principal_id, account_id))
}
