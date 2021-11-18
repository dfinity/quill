use crate::lib::get_idl_string;
use crate::lib::AnyhowResult;
use crate::lib::{get_candid_type, get_identity, get_local_candid};
use anyhow::anyhow;
use ic_agent::agent::QueryBuilder;
use ic_agent::agent::UpdateBuilder;
use ic_agent::{to_request_id, AgentError, Identity, RequestId};
use ic_types::{hash_tree::Label, principal::Principal};
use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use std::convert::TryFrom;
use std::time::Duration;
use std::time::SystemTime;

use super::get_agent;

const IC_REQUEST_DOMAIN_SEPARATOR: &[u8; 11] = b"\x0Aic-request";

#[derive(Debug)]
pub struct MessageError(String);

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}
impl std::error::Error for MessageError {}

/// Represents a signed message with the corresponding request id.
#[derive(Clone)]
pub struct SignedMessageWithRequestId {
    pub message: Ingress,
    pub request_id: Option<RequestId>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct RequestStatus {
    pub canister_id: String,
    pub request_id: String,
    pub content: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Ingress {
    pub call_type: String,
    pub request_id: Option<String>,
    pub content: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct IngressWithRequestId {
    pub ingress: Ingress,
    pub request_status: RequestStatus,
}

impl Ingress {
    pub fn parse(&self) -> AnyhowResult<(Principal, Principal, String, String)> {
        let cbor: Value = serde_cbor::from_slice(&hex::decode(&self.content)?)
            .map_err(|_| anyhow!("Invalid cbor data in the content of the message."))?;
        if let Value::Map(m) = cbor {
            let cbor_content = m
                .get(&Value::Text("content".to_string()))
                .ok_or_else(|| anyhow!("Invalid cbor content"))?;
            if let Value::Map(m) = cbor_content {
                if let (
                    Some(Value::Bytes(sender)),
                    Some(Value::Bytes(canister_id)),
                    Some(Value::Text(method_name)),
                    Some(Value::Bytes(arg)),
                ) = (
                    m.get(&Value::Text("sender".to_string())),
                    m.get(&Value::Text("canister_id".to_string())),
                    m.get(&Value::Text("method_name".to_string())),
                    m.get(&Value::Text("arg".to_string())),
                ) {
                    let sender = Principal::try_from(sender)?;
                    let canister_id = Principal::try_from(canister_id)?;
                    return Ok((
                        sender,
                        canister_id,
                        method_name.to_string(),
                        get_idl_string(arg, canister_id, method_name, "args")?,
                    ));
                }
            }
        }
        Err(anyhow!("Invalid cbor content"))
    }
}

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

    let ingress_expiry = Duration::from_secs(5 * 60);

    let (content, request_id) = if is_query {
        let bytes = QueryBuilder::new(&get_agent(pem)?, canister_id, method_name.to_string())
            .with_arg(args)
            .expire_after(ingress_expiry)
            .sign()?
            .signed_query;
        (hex::encode(bytes), None)
    } else {
        let signed_update =
            UpdateBuilder::new(&get_agent(pem)?, canister_id, method_name.to_string())
                .with_arg(args)
                .expire_after(ingress_expiry)
                .sign()?;

        (
            hex::encode(signed_update.signed_update),
            Some(signed_update.request_id),
        )
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
