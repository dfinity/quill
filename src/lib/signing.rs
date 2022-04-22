use crate::lib::AnyhowResult;
use crate::lib::{get_candid_type, get_idl_string, get_local_candid, TargetCanister};
use anyhow::{anyhow, Context};
use ic_agent::{
    agent::{QueryBuilder, UpdateBuilder},
    RequestId,
};
use ic_types::principal::Principal;
use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::time::Duration;

use super::get_agent;

#[derive(Debug)]
pub struct MessageError(String);

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}
impl std::error::Error for MessageError {}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum CallType {
    Update,
    Query,
}

impl Display for CallType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let call_type_string = match self {
            CallType::Update => "update",
            CallType::Query => "query",
        };

        write!(f, "{}", call_type_string)
    }
}

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ingress {
    pub call_type: CallType,
    pub request_id: Option<String>,
    pub content: String,
    pub target_canister: TargetCanister,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IngressWithRequestId {
    pub ingress: Ingress,
    pub request_status: RequestStatus,
}

impl Ingress {
    pub fn parse(&self) -> AnyhowResult<(Principal, Principal, String, String)> {
        let cbor: Value = serde_cbor::from_slice(&hex::decode(&self.content)?)
            .context("Invalid cbor data in the content of the message.")?;
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
                        get_idl_string(arg, self.target_canister, method_name, "args")?,
                    ));
                }
            }
        }
        Err(anyhow!("Invalid cbor content"))
    }
}

pub fn request_status_sign(
    pem: &str,
    request_id: RequestId,
    canister_id: Principal,
) -> AnyhowResult<RequestStatus> {
    let agent = get_agent(pem)?;
    let val = agent.sign_request_status(canister_id, request_id)?;
    Ok(RequestStatus {
        canister_id: canister_id.to_string(),
        request_id: request_id.into(),
        content: hex::encode(val.signed_request_status),
    })
}

pub fn sign(
    pem: &str,
    method_name: &str,
    args: Vec<u8>,
    target_canister: TargetCanister,
) -> AnyhowResult<SignedMessageWithRequestId> {
    let spec = get_local_candid(target_canister)?;
    let method_type = get_candid_type(spec, method_name);
    let is_query = match &method_type {
        Some((_, f)) => f.is_query(),
        _ => false,
    };

    let ingress_expiry = Duration::from_secs(5 * 60);
    let canister_id = Principal::from(target_canister);

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
            call_type: if is_query {
                CallType::Query
            } else {
                CallType::Update
            },
            request_id: request_id.map(|v| v.into()),
            content,
            target_canister,
        },
        request_id,
    })
}

/// Generates a bundle of signed messages (ingress + request status query).
pub fn sign_ingress_with_request_status_query(
    pem: &str,
    method_name: &str,
    args: Vec<u8>,
    target_canister: TargetCanister,
) -> AnyhowResult<IngressWithRequestId> {
    let msg_with_req_id = sign(pem, method_name, args, target_canister)?;
    let request_id = msg_with_req_id
        .request_id
        .context("No request id for transfer call found")?;
    let canister_id = Principal::from(target_canister);
    let request_status = request_status_sign(pem, request_id, canister_id)?;
    let message = IngressWithRequestId {
        ingress: msg_with_req_id.message,
        request_status,
    };
    Ok(message)
}
