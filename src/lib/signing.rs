use crate::lib::get_idl_string;
use crate::lib::{AnyhowResult, AuthInfo};
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use ic_agent::agent::UpdateBuilder;
use ic_agent::RequestId;
use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use std::convert::TryFrom;
use std::time::Duration;

#[cfg(feature = "ledger")]
use super::ledger::LedgerIdentity;
use super::{get_agent, get_default_role};

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
    pub role: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct IngressWithRequestId {
    pub ingress: Ingress,
    pub request_status: RequestStatus,
}

impl Ingress {
    pub fn parse(&self) -> AnyhowResult<(Principal, Principal, String, String, String)> {
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
                    let role = if let Some(role) = &self.role {
                        role
                    } else {
                        get_default_role(sender).unwrap_or("<none>")
                    };
                    return Ok((
                        sender,
                        canister_id,
                        method_name.to_string(),
                        get_idl_string(arg, canister_id, role, method_name, "args")?,
                        role.to_owned(),
                    ));
                }
            }
        }
        Err(anyhow!("Invalid cbor content"))
    }
}

pub fn request_status_sign(
    auth: &AuthInfo,
    request_id: RequestId,
    canister_id: Principal,
) -> AnyhowResult<RequestStatus> {
    let agent = get_agent(auth)?;
    let val = agent.sign_request_status(canister_id, request_id)?;
    Ok(RequestStatus {
        canister_id: canister_id.to_string(),
        request_id: request_id.into(),
        content: hex::encode(val.signed_request_status),
    })
}

pub fn sign(
    auth: &AuthInfo,
    canister_id: Principal,
    method_name: &str,
    args: Vec<u8>,
    role: &str,
    #[allow(unused)] is_staking: bool,
) -> AnyhowResult<SignedMessageWithRequestId> {
    let ingress_expiry = Duration::from_secs(5 * 60);
    let agent = get_agent(auth)?;
    let signed_update = sign_with(auth, is_staking, || {
        UpdateBuilder::new(&agent, canister_id, method_name.to_string())
            .with_arg(args)
            .expire_after(ingress_expiry)
            .sign()
    })?;
    let content = hex::encode(signed_update.signed_update);
    let request_id = signed_update.request_id;

    Ok(SignedMessageWithRequestId {
        message: Ingress {
            call_type: "update".to_string(),
            request_id: Some(request_id.into()),
            content,
            role: Some(role.to_owned()),
        },
        request_id: Some(request_id),
    })
}

fn sign_with<T>(auth: &AuthInfo, #[allow(unused)] is_staking: bool, f: impl FnOnce() -> T) -> T {
    #[cfg(feature = "ledger")]
    if is_staking && matches!(auth, AuthInfo::Ledger) {
        return LedgerIdentity::with_staking(f);
    }
    f()
}

/// Generates a bundle of signed messages (ingress + request status query).
pub fn sign_ingress_with_request_status_query(
    auth: &AuthInfo,
    canister_id: Principal,
    role: &str,
    method_name: &str,
    args: Vec<u8>,
) -> AnyhowResult<IngressWithRequestId> {
    sign_ingress_with_request_status_query_internal(
        auth,
        canister_id,
        role,
        method_name,
        args,
        false,
    )
}

/// Same as [`sign_ingress_with_request_status_query`], but signals that the request is staking.
pub fn sign_staking_ingress_with_request_status_query(
    auth: &AuthInfo,
    canister_id: Principal,
    role: &str,
    method_name: &str,
    args: Vec<u8>,
) -> AnyhowResult<IngressWithRequestId> {
    sign_ingress_with_request_status_query_internal(
        auth,
        canister_id,
        role,
        method_name,
        args,
        true,
    )
}

fn sign_ingress_with_request_status_query_internal(
    auth: &AuthInfo,
    canister_id: Principal,
    role: &str,
    method_name: &str,
    args: Vec<u8>,
    is_staking: bool,
) -> AnyhowResult<IngressWithRequestId> {
    #[cfg(feature = "ledger")]
    if matches!(auth, AuthInfo::Ledger)
        && !super::ledger::supported_transaction(&canister_id, method_name)
    {
        bail!(
            "Cannot use --ledger with this command. This version of Quill only supports transfers \
            and certain neuron management operations with a Ledger device"
        );
    }
    let msg_with_req_id = sign(auth, canister_id, method_name, args, role, is_staking)?;
    let request_id = msg_with_req_id
        .request_id
        .context("No request id for transfer call found")?;
    let request_status = request_status_sign(auth, request_id, canister_id)?;
    let message = IngressWithRequestId {
        ingress: msg_with_req_id.message,
        request_status,
    };
    Ok(message)
}
