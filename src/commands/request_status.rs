use crate::lib::get_ic_url;
use crate::lib::{get_agent, signing::RequestStatus, AnyhowResult};
use anyhow::{anyhow, Context};
use ic_agent::agent::{ReplicaV2Transport, Replied, RequestStatusResponse};
use ic_agent::AgentError::MessageError;
use ic_agent::{Agent, AgentError, RequestId};
use ic_types::Principal;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

pub async fn submit(req: &RequestStatus) -> AnyhowResult<Vec<u8>> {
    let canister_id =
        Principal::from_text(&req.canister_id).context("Cannot parse the canister id")?;
    let request_id = RequestId::from_str(&req.request_id).context("Cannot parse the request_id")?;
    let mut agent = get_agent("")?;
    update_agent_root_key(&mut agent).await?;
    agent.set_transport(ProxySignReplicaV2Transport {
        req: req.clone(),
        http_transport: Arc::new(
            ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create(get_ic_url())
                .context("Failed to create an agent")?,
        ),
    });
    let Replied::CallReplied(blob) = async {
        loop {
            match agent
                .request_status_raw(&request_id, canister_id, false)
                .await?
            {
                RequestStatusResponse::Replied { reply } => return Ok(reply),
                RequestStatusResponse::Rejected {
                    reject_code,
                    reject_message,
                } => {
                    return Err(anyhow!(AgentError::ReplicaError {
                        reject_code,
                        reject_message,
                    }))
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
    Ok(blob)
}

/// If testing locally or on a testnet, certification will fail due to ic-agent having
/// the root key hardcoded. If IC_URL is set, use the `fetch_root_key` method
/// on agent. Agent will update itself with the correct key.
async fn update_agent_root_key(agent: &mut Agent) -> AnyhowResult {
    if std::env::var("IC_URL").is_ok() {
        agent.fetch_root_key().await?;
    }
    Ok(())
}

pub(crate) struct ProxySignReplicaV2Transport {
    req: RequestStatus,
    http_transport: Arc<dyn 'static + ReplicaV2Transport + Send + Sync>,
}

impl ReplicaV2Transport for ProxySignReplicaV2Transport {
    fn call<'a>(
        &'a self,
        _effective_canister_id: Principal,
        _envelope: Vec<u8>,
        _request_id: RequestId,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send + 'a>> {
        unimplemented!()
    }

    fn read_state<'a>(
        &'a self,
        _canister_id: Principal,
        _content: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(transport: &ProxySignReplicaV2Transport) -> Result<Vec<u8>, AgentError> {
            let canister_id = Principal::from_text(transport.req.canister_id.clone())
                .map_err(|err| MessageError(format!("Unable to parse canister_id: {:?}", err)))?;
            let envelope = hex::decode(transport.req.content.clone()).map_err(|err| {
                MessageError(format!(
                    "Unable to decode request content (should be hexadecimal encoded): {}",
                    err
                ))
            })?;
            transport
                .http_transport
                .read_state(canister_id, envelope)
                .await
        }

        Box::pin(run(self))
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
