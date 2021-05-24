use super::signed_message::SignedMessage;
use ic_agent::agent::ReplicaV2Transport;
use ic_agent::{AgentError, RequestId};
use ic_types::Principal;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

pub(crate) struct SignReplicaV2Transport {
    buffer: Arc<RwLock<String>>,
    message: SignedMessage,
}

impl SignReplicaV2Transport {
    pub fn new(buffer: Arc<RwLock<String>>, message: SignedMessage) -> Self {
        Self { buffer, message }
    }
}

fn run(
    s: &SignReplicaV2Transport,
    envelope: Vec<u8>,
    request_id: Option<RequestId>,
) -> Result<(), AgentError> {
    let message = s.message.clone().with_content(hex::encode(&envelope));
    let message = match request_id {
        Some(request_id) => message
            .with_call_type("update".to_string())
            .with_request_id(request_id),
        None => message.with_call_type("query".to_string()),
    };
    *(s.buffer.write().unwrap()) =
        serde_json::to_string(&message).map_err(|err| AgentError::MessageError(err.to_string()))?;
    Ok(())
}

impl ReplicaV2Transport for SignReplicaV2Transport {
    fn read_state<'a>(
        &'a self,
        _effective_canister_id: Principal,
        _envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        unimplemented!()
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
            Ok(Vec::new())
        }
        Box::pin(filler())
    }

    fn status<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        unimplemented!()
    }
}
