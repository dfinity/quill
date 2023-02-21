use std::str::FromStr;

use crate::{
    lib::signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    lib::{registry_canister_id, AnyhowResult, AuthInfo, ROLE_NNS_REGISTRY},
};
use anyhow::{anyhow, Context};
use candid::{CandidType, Encode};
use clap::Parser;
use ic_base_types::PrincipalId;

/// Signs a message to replace Node Provide ID in targeted Node Operator Record
#[derive(Parser)]
pub struct ReplaceNodeProviderIdOpts {
    /// The Principal id of the node operator. This principal is the entity that
    /// is able to add and remove nodes.
    #[clap(long)]
    node_operator_id: String,

    /// The new Principal id of the node provider.
    #[clap(long)]
    node_provider_id: String,
}

/// The payload to update an existing Node Operator (without going through the proposal process)
#[derive(CandidType)]
pub struct UpdateNodeOperatorConfigDirectlyPayload {
    pub node_operator_id: Option<PrincipalId>,
    pub node_provider_id: Option<PrincipalId>,
}

pub fn exec(
    auth: &AuthInfo,
    opts: ReplaceNodeProviderIdOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let node_operator_id = PrincipalId::from_str(&opts.node_operator_id)
        .map_err(|e| anyhow!(e))
        .with_context(|| {
            format!(
                "node_operator_id {} is not valid Principal",
                &opts.node_operator_id
            )
        })?;
    let node_provider_id = PrincipalId::from_str(&opts.node_provider_id)
        .map_err(|e| anyhow!(e))
        .with_context(|| {
            format!(
                "node_provider_id {} is not valid Principal",
                &opts.node_provider_id
            )
        })?;
    let args = Encode!(&UpdateNodeOperatorConfigDirectlyPayload {
        node_operator_id: Some(node_operator_id),
        node_provider_id: Some(node_provider_id),
    })?;
    Ok(vec![sign_ingress_with_request_status_query(
        auth,
        registry_canister_id(),
        ROLE_NNS_REGISTRY,
        "update_node_operator_config_directly",
        args,
    )?])
}
