use crate::commands::request_status;
use crate::lib::{
    get_agent, get_candid_type, get_local_candid,
    sign::sign_transport::{SignReplicaV2Transport, SignedMessageWithRequestId},
    sign::signed_message::{Ingress, IngressWithRequestId},
    AnyhowResult, AuthInfo,
};
use anyhow::anyhow;
use ic_agent::AgentError;
use ic_types::principal::Principal;
use std::convert::TryInto;
use std::time::SystemTime;

async fn sign(
    auth: &AuthInfo,
    canister_id: Principal,
    method_name: &str,
    args: Vec<u8>,
) -> AnyhowResult<SignedMessageWithRequestId> {
    let spec = get_local_candid(canister_id)?;
    let method_type = get_candid_type(spec, method_name);
    let is_query = match &method_type {
        Some((_, f)) => f.is_query(),
        _ => false,
    };

    let mut sign_agent = get_agent(auth)?;

    let timeout = std::time::Duration::from_secs(5 * 60);
    let expiration_system_time = SystemTime::now()
        .checked_add(timeout)
        .ok_or_else(|| anyhow!("Time wrapped around."))?;

    let transport = SignReplicaV2Transport::new(None);
    let data = transport.data.clone();
    sign_agent.set_transport(transport);

    if is_query {
        match sign_agent
            .query(&canister_id, method_name)
            .with_effective_canister_id(canister_id)
            .with_arg(&args)
            .expire_at(expiration_system_time)
            .call()
            .await
        {
            Err(AgentError::MissingReplicaTransport()) => {}
            val => panic!("Unexpected return value from query execution: {:?}", val),
        };
    } else {
        sign_agent
            .update(&canister_id, method_name)
            .with_effective_canister_id(canister_id)
            .with_arg(&args)
            .expire_at(expiration_system_time)
            .call()
            .await
            .map(|_| ())
            .map_err(|e| anyhow!(e))?;
    }

    let message = data.read().unwrap().clone().try_into()?;
    Ok(message)
}

/// Generates a bundle of signed messages (ingress + request status query).
pub async fn sign_ingress_with_request_status_query(
    auth: &AuthInfo,
    canister_id: Principal,
    method_name: &str,
    args: Vec<u8>,
) -> AnyhowResult<IngressWithRequestId> {
    let msg_with_req_id = sign(auth, canister_id, method_name, args).await?;
    let request_id = msg_with_req_id
        .request_id
        .expect("No request id for transfer call found");
    let request_status = request_status::sign(auth, request_id, canister_id).await?;
    let message = IngressWithRequestId {
        ingress: msg_with_req_id.message.try_into()?,
        request_status,
    };
    Ok(message)
}

/// Generates a signed ingress message.
pub async fn sign_ingress(
    auth: &AuthInfo,
    canister_id: Principal,
    method_name: &str,
    args: Vec<u8>,
) -> AnyhowResult<Ingress> {
    let msg = sign(auth, canister_id, method_name, args).await?;
    Ok(msg.message.try_into()?)
}
