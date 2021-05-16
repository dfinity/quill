use super::signed_message::SignedMessageV1;

use ic_agent::agent::ReplicaV2Transport;
use ic_agent::{AgentError, RequestId};
use ic_types::Principal;
use std::future::Future;
use std::pin::Pin;

pub(crate) struct SignReplicaV2Transport {
    message_template: SignedMessageV1,
}

impl SignReplicaV2Transport {
    pub fn new(message_template: SignedMessageV1) -> Self {
        Self { message_template }
    }
}

impl ReplicaV2Transport for SignReplicaV2Transport {
    fn read_state<'a>(
        &'a self,
        _effective_canister_id: Principal,
        _envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(_: &SignReplicaV2Transport) -> Result<Vec<u8>, AgentError> {
            Err(AgentError::MessageError(
                "read_state calls not supported".to_string(),
            ))
        }

        Box::pin(run(self))
    }

    fn call<'a>(
        &'a self,
        _effective_canister_id: Principal,
        envelope: Vec<u8>,
        request_id: RequestId,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send + 'a>> {
        async fn run(
            s: &SignReplicaV2Transport,
            envelope: Vec<u8>,
            request_id: RequestId,
        ) -> Result<(), AgentError> {
            let message = s
                .message_template
                .clone()
                .with_call_type("update".to_string())
                .with_request_id(request_id)
                .with_content(hex::encode(&envelope));
            let json = serde_json::to_string(&message)
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            println!("{}", json);
            Ok(())
        }

        Box::pin(run(self, envelope, request_id))
    }

    fn query<'a>(
        &'a self,
        _effective_canister_id: Principal,
        envelope: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(s: &SignReplicaV2Transport, envelope: Vec<u8>) -> Result<Vec<u8>, AgentError> {
            let message = s
                .message_template
                .clone()
                .with_call_type("query".to_string())
                .with_content(hex::encode(&envelope));
            let json = serde_json::to_string(&message)
                .map_err(|x| AgentError::MessageError(x.to_string()))?;
            println!("{}", json);
            Ok(Vec::new())
        }

        Box::pin(run(self, envelope))
    }

    fn status<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AgentError>> + Send + 'a>> {
        async fn run(_: &SignReplicaV2Transport) -> Result<Vec<u8>, AgentError> {
            Err(AgentError::MessageError(
                "status calls not supported".to_string(),
            ))
        }

        Box::pin(run(self))
    }
}
