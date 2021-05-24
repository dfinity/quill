use crate::lib::{
    environment::Environment, get_idl_string, read_json, sign::signed_message::RequestStatus,
    DfxResult,
};
use anyhow::{anyhow, Context};
use clap::Clap;
use ic_agent::agent::{Replied, RequestStatusResponse};
use ic_agent::{AgentError, RequestId};
use ic_types::Principal;
use std::str::FromStr;
use std::sync::Arc;

/// Requests the status of a specified call from a canister.
#[derive(Clap)]
pub struct RequestStatusSubmitOpts {
    /// Path to the signed status request
    #[clap(long)]
    file: String,
}

pub async fn exec(env: &dyn Environment, opts: RequestStatusSubmitOpts) -> DfxResult {
    let json = read_json(opts.file)?;
    if let Ok(req) = serde_json::from_str::<RequestStatus>(&json) {
        submit(env, req).await
    } else {
        return Err(anyhow!("Invalid JSON content"));
    }
}

pub async fn submit(env: &dyn Environment, req: RequestStatus) -> DfxResult {
    let canister_id = Principal::from_text(&req.canister_id).expect("Couldn't parse canister id");
    let request_id =
        RequestId::from_str(&req.request_id).context("Invalid argument: request_id")?;
    let mut agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    agent.set_transport(ProxySignReplicaV2Transport {
        req,
        http_transport: Arc::new(
            ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create(
                env.get_network_descriptor().providers[0].clone(),
            )
            .unwrap(),
        ),
    });
    let Replied::CallReplied(blob) = async {
        loop {
            match agent
                .request_status_raw(&request_id, canister_id.clone())
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
    get_idl_string(&blob, "pp", &None).context("Invalid IDL blob.")?;

    Ok(())
}

pub(crate) struct ProxySignReplicaV2Transport {
    req: RequestStatus,
    http_transport: Arc<dyn 'static + ReplicaV2Transport + Send + Sync>,
}

use ic_agent::agent::ReplicaV2Transport;
use std::future::Future;
use std::pin::Pin;

impl ReplicaV2Transport for ProxySignReplicaV2Transport {
    fn read_state<'a>(
        &'a self,
        _canister_id: Principal,
        _content: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        self.http_transport.read_state(
            Principal::from_text(self.req.canister_id.clone()).unwrap(),
            hex::decode(self.req.content.clone()).unwrap(),
        )
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
