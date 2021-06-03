use crate::lib::get_idl_string;
use crate::lib::AnyhowResult;
use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use ic_agent::RequestId;
use ic_types::principal::Principal;
use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use std::convert::TryFrom;
use std::time::Duration;

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
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct IngressWithRequestId {
    pub ingress: Ingress,
    pub request_status: RequestStatus,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct NeuronStakeMessage {
    pub transfer: IngressWithRequestId,
    pub claim: IngressWithRequestId,
}

impl Ingress {
    pub fn with_call_type(mut self, request_type: String) -> Self {
        self.call_type = request_type;
        self
    }

    pub fn with_request_id(mut self, request_id: RequestId) -> Self {
        self.request_id = Some(String::from(request_id));
        self
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn parse(&self) -> AnyhowResult<(Principal, Principal, String, String)> {
        let cbor: Value = serde_cbor::from_slice(&hex::decode(&self.content)?)
            .map_err(|_| anyhow!("Invalid cbor data in the content of the message."))?;
        if let Value::Map(m) = cbor {
            let cbor_content = m
                .get(&Value::Text("content".to_string()))
                .ok_or_else(|| anyhow!("Invalid cbor content"))?;
            if let Value::Map(m) = cbor_content {
                if let (
                    Some(Value::Integer(ingress_expiry)),
                    Some(Value::Bytes(sender)),
                    Some(Value::Bytes(canister_id)),
                    Some(Value::Text(method_name)),
                    Some(Value::Bytes(arg)),
                ) = (
                    m.get(&Value::Text("ingress_expiry".to_string())),
                    m.get(&Value::Text("sender".to_string())),
                    m.get(&Value::Text("canister_id".to_string())),
                    m.get(&Value::Text("method_name".to_string())),
                    m.get(&Value::Text("arg".to_string())),
                ) {
                    let seconds_since_epoch_cbor =
                        Duration::from_nanos(*ingress_expiry as u64).as_secs();
                    let expiration_from_cbor = Utc.timestamp(seconds_since_epoch_cbor as i64, 0);
                    if Utc::now() > expiration_from_cbor {
                        return Err(anyhow!(
                            "The message has been expired at: {}",
                            expiration_from_cbor
                        ));
                    }
                    let sender = Principal::try_from(sender)?;
                    let canister_id = Principal::try_from(canister_id)?;
                    return Ok((
                        sender,
                        canister_id.clone(),
                        method_name.to_string().to_string(),
                        get_idl_string(arg, &canister_id.to_string(), &method_name, "args", "pp")?,
                    ));
                }
            }
        }
        Err(anyhow!("Invalid cbor content"))
    }
}
