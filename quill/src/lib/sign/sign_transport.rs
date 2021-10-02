use crate::lib::sign::signed_message::{Ingress, RequestStatus};
use ic_agent::agent::ReplicaV2Transport;
use ic_agent::{AgentError, RequestId};
use ic_types::Principal;
use std::convert::TryFrom;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct MessageError(String);

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}
impl std::error::Error for MessageError {}

#[derive(Clone)]
pub enum Message {
    Ingress(Ingress),
    RequestStatus(RequestStatus),
}

impl TryFrom<Message> for Ingress {
    type Error = MessageError;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::Ingress(ingress) => Ok(ingress),
            Message::RequestStatus(_) => Err(MessageError(
                "Expect Ingress but got RequestStatus".to_string(),
            )),
        }
    }
}

impl TryFrom<Message> for RequestStatus {
    type Error = MessageError;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::RequestStatus(request_status) => Ok(request_status),
            Message::Ingress(_) => Err(MessageError(
                "Expect RequestStatus but got Ingress".to_string(),
            )),
        }
    }
}

/// Represents a signed message with the corresponding request id.
#[derive(Clone)]
pub struct SignedMessageWithRequestId {
    pub message: Message,
    pub request_id: Option<RequestId>,
}

#[derive(Clone)]
pub enum TransportState {
    SignedMessageWithRequestId(SignedMessageWithRequestId),
    RequestId(Option<RequestId>),
}

impl TransportState {
    fn get_request_id(&self) -> Option<RequestId> {
        match self {
            TransportState::SignedMessageWithRequestId(msg) => msg.request_id,
            TransportState::RequestId(req_id) => *req_id,
        }
    }
}

impl TryFrom<TransportState> for SignedMessageWithRequestId {
    type Error = MessageError;

    fn try_from(state: TransportState) -> Result<Self, Self::Error> {
        match state {
            TransportState::SignedMessageWithRequestId(msg) => Ok(msg),
            TransportState::RequestId(_) => {
                Err(MessageError("Message is not available".to_string()))
            }
        }
    }
}

/// Implement a "transport" component, which is not using networking, but writes all requests to
/// the specified buffer.
pub struct SignReplicaV2Transport {
    pub data: Arc<RwLock<TransportState>>,
}

impl SignReplicaV2Transport {
    pub fn new(request_id: Option<RequestId>) -> Self {
        Self {
            data: Arc::new(RwLock::new(TransportState::RequestId(request_id))),
        }
    }
}

fn run(s: &SignReplicaV2Transport, envelope: Vec<u8>, request_id: Option<RequestId>) {
    let message = Ingress::default().with_content(hex::encode(&envelope));
    let message = match request_id {
        Some(request_id) => message
            .with_call_type("update".to_string())
            .with_request_id(request_id),
        None => message.with_call_type("query".to_string()),
    };
    let mut data = s.data.write().unwrap();
    *data = TransportState::SignedMessageWithRequestId(SignedMessageWithRequestId {
        request_id,
        message: Message::Ingress(message),
    });
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
                request_id: s.data.read().unwrap().get_request_id().unwrap().into(),
                canister_id: canister_id.to_string(),
                content: hex::encode(content),
            };
            let mut data = s.data.write().unwrap();
            *data = TransportState::SignedMessageWithRequestId(SignedMessageWithRequestId {
                message: Message::RequestStatus(status_req),
                request_id: data.get_request_id(),
            });
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
        run(self, envelope, Some(request_id));
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
        run(self, envelope, None);
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
