use crate::{
    lib::signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    lib::{governance_canister_id, AnyhowResult, AuthInfo, ROLE_NNS_GOVERNANCE},
};
use anyhow::{anyhow, Context};
use candid::{CandidType, Encode};
use clap::Parser;

#[derive(CandidType)]
pub struct AccountIdentifier {
    hash: Vec<u8>,
}
#[derive(CandidType)]
pub struct UpdateNodeProvider {
    pub reward_account: Option<AccountIdentifier>,
}

/// Update node provider details.
#[derive(Parser)]
pub struct UpdateNodeProviderOpts {
    /// The account identifier of the reward account.
    #[clap(long)]
    reward_account: String,
}

pub fn exec(
    auth: &AuthInfo,
    opts: UpdateNodeProviderOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let reward_account = icp_ledger::AccountIdentifier::from_hex(&opts.reward_account)
        .map_err(|e| anyhow!(e))
        .with_context(|| format!("Account {} is not valid address", &opts.reward_account))?;
    let args = Encode!(&UpdateNodeProvider {
        reward_account: Some(AccountIdentifier {
            hash: reward_account.hash.to_vec()
        })
    })?;
    Ok(vec![sign_ingress_with_request_status_query(
        auth,
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "update_node_provider",
        args,
    )?])
}
