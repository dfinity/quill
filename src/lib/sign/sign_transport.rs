use crate::lib::sign::signed_message::{Ingress, RequestStatus};
use ic_agent::agent::ReplicaV2Transport;
use ic_agent::{AgentError, RequestId};
use ic_types::Principal;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SignedMessageWithRequestId {
    pub buffer: String,
    pub request_id: Option<RequestId>,
}

impl SignedMessageWithRequestId {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            buffer: String::new(),
            request_id: None,
        }))
    }
}

pub struct SignReplicaV2Transport {
    pub data: Arc<RwLock<SignedMessageWithRequestId>>,
}

fn run(
    s: &SignReplicaV2Transport,
    envelope: Vec<u8>,
    request_id: Option<RequestId>,
) -> Result<(), AgentError> {
    let message = Ingress::default().with_content(hex::encode(&envelope));
    let message = match request_id {
        Some(request_id) => message
            .with_call_type("update".to_string())
            .with_request_id(request_id),
        None => message.with_call_type("query".to_string()),
    };
    let mut data = s.data.write().unwrap();
    data.request_id = request_id;
    data.buffer =
        serde_json::to_string(&message).map_err(|err| AgentError::MessageError(err.to_string()))?;
    Ok(())
}

impl ReplicaV2Transport for SignReplicaV2Transport {
    fn read_state<'a>(
        &'a self,
        canister_id: Principal,
        content: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn filler(
            s: &SignReplicaV2Transport,
            canister_id: Principal,
            content: Vec<u8>,
        ) -> Result<Vec<u8>, AgentError> {
            let status_req = RequestStatus {
                request_id: s.data.read().unwrap().request_id.clone().unwrap().into(),
                canister_id: canister_id.to_string(),
                content: hex::encode(content),
            };
            s.data.write().unwrap().buffer = serde_json::to_string(&status_req)
                .map_err(|err| AgentError::MessageError(err.to_string()))?;
            Err(AgentError::MissingReplicaTransport())
        }
        Box::pin(filler(self, canister_id, content))
    }

    fn call<'a>(
        &'a self,
        _effective_canister_id: Principal,
        envelope: Vec<u8>,
        request_id: RequestId,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send + 'a>> {
        run(self, envelope, Some(request_id)).expect("Couldn't execute call");
        async fn filler() -> Result<(), AgentError> {
            Ok(())
        }
        Box::pin(filler())
    }

    fn query<'a>(
        &'a self,
        _effective_canister_id: Principal,
        envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        run(self, envelope, None).expect("Couldn't execute call");
        async fn filler() -> Result<Vec<u8>, AgentError> {
            Err(AgentError::MissingReplicaTransport())
        }
        Box::pin(filler())
    }

    fn status<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        unimplemented!()
    }
}
