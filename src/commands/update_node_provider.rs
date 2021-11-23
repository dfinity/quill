use crate::{
    lib::signing::{sign_ingress, Ingress},
    lib::{governance_canister_id, AnyhowResult, AuthInfo},
};
use candid::{CandidType, Encode};
use clap::Parser;
use ledger_canister::AccountIdentifier;

#[derive(CandidType)]
pub struct UpdateNodeProvider {
    pub reward_account: Option<AccountIdentifier>,
}

/// Signs a neuron configuration change.
#[derive(Parser)]
pub struct UpdateNodeProviderOpts {
    /// The account identifier of the reward account.
    #[clap(long)]
    reward_account: String,
}

pub fn exec(auth: &AuthInfo, opts: UpdateNodeProviderOpts) -> AnyhowResult<Vec<Ingress>> {
    let reward_account = ledger_canister::AccountIdentifier::from_hex(&opts.reward_account)
        .map_err(|e| {
            anyhow::Error::msg(format!(
                "Account {} is not valid address, {}",
                &opts.reward_account, e,
            ))
        })?;
    let args = Encode!(&UpdateNodeProvider {
        reward_account: Some(reward_account)
    })?;
    Ok(vec![sign_ingress(
        auth,
        governance_canister_id(),
        "update_node_provider",
        args,
    )?])
}
