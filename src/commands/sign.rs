use crate::lib::{
    get_candid_type, get_identity, get_local_candid,
    sign::signed_message::{Ingress, IngressWithRequestId, RequestStatus},
    sign::SignedMessageWithRequestId,
    AnyhowResult,
};
use anyhow::anyhow;
use ic_agent::{to_request_id, AgentError, Identity, RequestId};
use ic_types::{hash_tree::Label, principal::Principal};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

const IC_REQUEST_DOMAIN_SEPARATOR: &[u8; 11] = b"\x0Aic-request";

pub fn sign(
    pem: &str,
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

    let identity = get_identity(pem);
    let ingress_expiry = get_expiry_timestamp()?;

    let (content, request_id) = if is_query {
        let req = QueryContent::QueryRequest {
            sender: identity.sender().map_err(AgentError::SigningError)?,
            canister_id,
            method_name: method_name.to_string(),
            arg: args,
            ingress_expiry,
        };
        let (bytes, _) = sign_content(identity, req)?;
        (hex::encode(bytes), None)
    } else {
        let req = CallRequestContent::CallRequest {
            canister_id,
            method_name: method_name.into(),
            arg: args,
            sender: identity.sender().map_err(AgentError::SigningError)?,
            ingress_expiry,
        };
        let (bytes, request_id) = sign_content(identity, req)?;
        (hex::encode(bytes), Some(request_id))
    };

    Ok(SignedMessageWithRequestId {
        message: Ingress {
            call_type: if is_query { "query" } else { "update" }.to_string(),
            request_id: request_id.map(|v| v.into()),
            content,
        },
        request_id,
    })
}

// A request as submitted to /api/v2/.../call
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum CallRequestContent {
    #[serde(rename = "call")]
    CallRequest {
        ingress_expiry: u64,
        sender: Principal,
        canister_id: Principal,
        method_name: String,
        #[serde(with = "serde_bytes")]
        arg: Vec<u8>,
    },
}

// A request as submitted to /api/v2/.../query
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum QueryContent {
    #[serde(rename = "query")]
    QueryRequest {
        ingress_expiry: u64,
        sender: Principal,
        canister_id: Principal,
        method_name: String,
        #[serde(with = "serde_bytes")]
        arg: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Envelope<T: Serialize> {
    pub content: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serde_bytes")]
    pub sender_pubkey: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serde_bytes")]
    pub sender_sig: Option<Vec<u8>>,
}

fn sign_content<'a, T>(
    identity: Box<dyn Identity + Send + Sync>,
    request: T,
) -> Result<(Vec<u8>, RequestId), AgentError>
where
    T: 'a + Serialize,
{
    let request_id = to_request_id(&request)?;
    let mut msg = vec![];
    msg.extend_from_slice(IC_REQUEST_DOMAIN_SEPARATOR);
    msg.extend_from_slice(request_id.as_slice());
    let signature = identity.sign(&msg).map_err(AgentError::SigningError)?;

    let envelope = Envelope {
        content: request,
        sender_pubkey: signature.public_key,
        sender_sig: signature.signature,
    };

    let mut serialized_bytes = Vec::new();
    let mut serializer = serde_cbor::Serializer::new(&mut serialized_bytes);
    serializer.self_describe()?;
    envelope.serialize(&mut serializer)?;

    Ok((serialized_bytes, request_id))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "request_type")]
pub enum ReadStateContent {
    #[serde(rename = "read_state")]
    ReadStateRequest {
        ingress_expiry: u64,
        sender: Principal,
        paths: Vec<Vec<Label>>,
    },
}

fn get_expiry_timestamp() -> AnyhowResult<u64> {
    let timeout = std::time::Duration::from_secs(5 * 60);
    Ok(SystemTime::now()
        .checked_add(timeout)
        .ok_or_else(|| anyhow!("Time wrapped around."))?
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time wrapped around")
        .as_nanos() as u64)
}

pub fn request_status_sign(
    pem: &str,
    request_id: RequestId,
    canister_id: Principal,
) -> AnyhowResult<RequestStatus> {
    let paths: Vec<Vec<Label>> = vec![vec!["request_status".into(), request_id.as_slice().into()]];

    let identity = get_identity(pem);
    let ingress_expiry = get_expiry_timestamp()?;

    let req = ReadStateContent::ReadStateRequest {
        sender: identity.sender().map_err(AgentError::SigningError)?,
        paths,
        ingress_expiry,
    };
    let (bytes, _) = sign_content(identity, req)?;
    Ok(RequestStatus {
        canister_id: canister_id.to_string(),
        request_id: request_id.into(),
        content: hex::encode(bytes),
    })
}
/// Generates a bundle of signed messages (ingress + request status query).
pub fn sign_ingress_with_request_status_query(
    pem: &str,
    canister_id: Principal,
    method_name: &str,
    args: Vec<u8>,
) -> AnyhowResult<IngressWithRequestId> {
    let msg_with_req_id = sign(pem, canister_id, method_name, args)?;
    let request_id = msg_with_req_id
        .request_id
        .expect("No request id for transfer call found");
    let request_status = request_status_sign(pem, request_id, canister_id)?;
    let message = IngressWithRequestId {
        ingress: msg_with_req_id.message,
        request_status,
    };
    Ok(message)
}

/// Generates a signed ingress message.
pub fn sign_ingress(
    pem: &str,
    canister_id: Principal,
    method_name: &str,
    args: Vec<u8>,
) -> AnyhowResult<Ingress> {
    let msg = sign(pem, canister_id, method_name, args)?;
    Ok(msg.message)
}
