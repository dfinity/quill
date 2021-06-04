use crate::commands::request_status;
use crate::lib::{
    get_agent, get_candid_type, get_local_candid,
    sign::sign_transport::{SignReplicaV2Transport, SignedMessageWithRequestId},
    AnyhowResult,
};
use anyhow::anyhow;
use ic_agent::AgentError;
use ic_types::principal::Principal;
use std::time::SystemTime;

async fn sign(
    pem: &Option<String>,
    canister_id: Principal,
    method_name: &str,
    args: Vec<u8>,
) -> AnyhowResult<SignedMessageWithRequestId> {
    let spec = get_local_candid(&canister_id.to_string());
    let method_type = spec.and_then(|spec| get_candid_type(spec, method_name));
    let is_query = match &method_type {
        Some((_, f)) => f.is_query(),
        _ => false,
    };

    let mut sign_agent = get_agent(pem)?;

    let timeout = std::time::Duration::from_secs(5 * 60);
    let expiration_system_time = SystemTime::now()
        .checked_add(timeout)
        .ok_or_else(|| anyhow!("Time wrapped around."))?;

    let data = SignedMessageWithRequestId::new();
    let transport = SignReplicaV2Transport { data: data.clone() };
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

    let data = data.read().unwrap().clone();
    Ok(data)
}

/// Generates a bundle of signed messages (ingress + request status query).
pub async fn sign_ingress_with_request_status_query(
    pem: &Option<String>,
    canister_id: Principal,
    method_name: &str,
    args: Vec<u8>,
) -> AnyhowResult<String> {
    let msg_with_req_id = sign(pem, canister_id.clone(), &method_name, args).await?;
    let request_id = msg_with_req_id
        .request_id
        .expect("No request id for transfer call found");
    let req_status_signed_msg = request_status::sign(pem, request_id, canister_id).await?;
    let mut out = String::new();
    out.push_str("{ \"ingress\": ");
    out.push_str(&msg_with_req_id.buffer);
    out.push_str(", \"request_status\": ");
    out.push_str(&req_status_signed_msg);
    out.push_str("}");
    Ok(out)
}
