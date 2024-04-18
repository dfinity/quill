use crate::lib::get_ic_url;
use crate::lib::{
    display_response, get_agent, get_idl_string, signing::RequestStatus, AnyhowResult, AuthInfo,
};
use anyhow::{anyhow, Context};
use candid::Principal;
use ic_agent::agent::http_transport::ReqwestTransport;
use ic_agent::agent::{ReplyResponse, RequestStatusResponse, Transport};
use ic_agent::AgentError::MessageError;
use ic_agent::{AgentError, RequestId};
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

pub async fn submit(
    req: &RequestStatus,
    method_name: Option<String>,
    role: &str,
    raw: bool,
    fetch_root_key: bool,
) -> AnyhowResult<String> {
    let canister_id =
        Principal::from_text(&req.canister_id).context("Couldn't parse canister id")?;
    let request_id =
        RequestId::from_str(&req.request_id).context("Invalid argument: request_id")?;
    let mut agent = get_agent(&AuthInfo::NoAuth)?;
    // fetching root key before replacing the transport layer because the proxy layer does not support the necessary functions
    if fetch_root_key {
        agent.fetch_root_key().await?;
    }
    agent.set_transport(ProxySignTransport {
        req: req.clone(),
        http_transport: Arc::new(
            ReqwestTransport::create(get_ic_url()).context("Failed to create an agent")?,
        ),
    });
    let ReplyResponse { arg: blob } = async {
        loop {
            match agent.request_status_raw(&request_id, canister_id).await? {
                RequestStatusResponse::Replied(reply) => return Ok(reply),
                RequestStatusResponse::Rejected(response) => {
                    return Err(anyhow!(AgentError::CertifiedReject(response)))
                }
                RequestStatusResponse::Unknown
                | RequestStatusResponse::Received
                | RequestStatusResponse::Processing => {
                    println!("The request is being processed...");
                }
                RequestStatusResponse::Done => {
                    return Err(anyhow!(AgentError::RequestStatusDoneNoReply(String::from(
                        request_id
                    ),)))
                }
            };

            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
    .await?;
    let method_str = method_name.unwrap_or_default();
    if raw {
        get_idl_string(&blob, canister_id, role, &method_str, "rets")
    } else {
        display_response(&blob, canister_id, role, &method_str, "rets").or_else(|e| {
            get_idl_string(&blob, canister_id, role, &method_str, "rets").map(|m| {
                format!("Error pretty-printing response: {e}. Falling back to IDL display\n{m}",)
            })
        })
    }
    .context("Invalid IDL blob.")
}

pub(crate) struct ProxySignTransport {
    req: RequestStatus,
    http_transport: Arc<dyn 'static + Transport + Send + Sync>,
}

impl Transport for ProxySignTransport {
    fn read_state<'a>(
        &'a self,
        _canister_id: Principal,
        _content: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(transport: &ProxySignTransport) -> Result<Vec<u8>, AgentError> {
            let canister_id = Principal::from_text(transport.req.canister_id.clone())
                .map_err(|err| MessageError(format!("Unable to parse canister_id: {err}")))?;
            let envelope = hex::decode(transport.req.content.clone()).map_err(|err| {
                MessageError(format!(
                    "Unable to decode request content (should be hexadecimal encoded): {err}",
                ))
            })?;
            transport
                .http_transport
                .read_state(canister_id, envelope)
                .await
        }

        Box::pin(run(self))
    }

    fn read_subnet_state(
        &self,
        subnet_id: Principal,
        envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + '_>> {
        self.http_transport.read_subnet_state(subnet_id, envelope)
    }

    fn call<'a>(
        &'a self,
        _effective_canister_id: Principal,
        _envelope: Vec<u8>,
        _request_id: RequestId,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send + 'a>> {
        unimplemented!()
    }

    fn query<'a>(
        &'a self,
        _effective_canister_id: Principal,
        _envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        unimplemented!()
    }

    fn status<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        unimplemented!()
    }
}
