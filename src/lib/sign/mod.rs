use crate::lib::sign::signed_message::Ingress;
use ic_agent::RequestId;
pub mod signed_message;

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
