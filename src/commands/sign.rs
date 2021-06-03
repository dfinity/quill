use crate::lib::get_candid_type;
use crate::lib::get_local_candid;
use crate::lib::sign::sign_transport::SignReplicaV2Transport;
use crate::lib::sign::sign_transport::SignedMessageWithRequestId;
use crate::lib::{get_agent, AnyhowResult};
use anyhow::anyhow;
use ic_agent::AgentError;
use ic_types::principal::Principal;
use std::time::SystemTime;

pub async fn sign(
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
