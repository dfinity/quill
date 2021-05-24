use crate::lib::DfxResult;
use crate::lib::{get_candid_type, get_idl_string, get_local_candid};
use anyhow::{anyhow, bail};
use chrono::{TimeZone, Utc};
use ic_agent::RequestId;
use ic_types::principal::Principal;
use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use std::convert::TryFrom;
use std::time::Duration;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub(crate) struct SignedStatusRequest {
    pub canister_id: String,
    pub request_id: String,
    pub content: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub(crate) struct SignedMessage {
    pub call_type: String,
    pub request_id: Option<String>,
    pub content: String,
}

impl SignedMessage {
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

    pub fn parse(&self) -> DfxResult<(Principal, Principal, String, String)> {
        let content = hex::decode(&self.content)?;
        let cbor: Value = serde_cbor::from_slice(&content)
            .map_err(|_| anyhow!("Invalid cbor data in the content of the message."))?;

        if let Value::Map(m) = cbor {
            let cbor_content = m
                .get(&Value::Text("content".to_string()))
                .ok_or_else(|| anyhow!("Invalid cbor content"))?;
            if let Value::Map(m) = cbor_content {
                let ingress_expiry = m
                    .get(&Value::Text("ingress_expiry".to_string()))
                    .ok_or_else(|| anyhow!("Invalid cbor content"))?;
                if let Value::Integer(s) = ingress_expiry {
                    let seconds_since_epoch_cbor = Duration::from_nanos(*s as u64).as_secs();
                    let expiration_from_cbor = Utc.timestamp(seconds_since_epoch_cbor as i64, 0);
                    if Utc::now() > expiration_from_cbor {
                        bail!("The message has been expired at: {}", expiration_from_cbor);
                    }
                } else {
                    bail!("Invalid cbor content");
                }

                let sender = m
                    .get(&Value::Text("sender".to_string()))
                    .ok_or_else(|| anyhow!("Invalid cbor content"))?;
                if let Value::Bytes(sender) = sender {
                    let sender = Principal::try_from(sender)
                        .map_err(|_| anyhow!("Invalid cbor content."))?;
                    let canister_id = m
                        .get(&Value::Text("canister_id".to_string()))
                        .ok_or_else(|| anyhow!("Invalid cbor content"))?;
                    if let Value::Bytes(canister_id) = canister_id {
                        let canister_id = Principal::try_from(canister_id)
                            .map_err(|_| anyhow!("Invalid cbor content."))?;
                        let method_name = m
                            .get(&Value::Text("method_name".to_string()))
                            .ok_or_else(|| anyhow!("Invalid cbor content"))?;
                        if let Value::Text(method_name) = method_name {
                            let spec = get_local_candid(&canister_id.to_string());
                            let method_type =
                                spec.and_then(|spec| get_candid_type(spec, method_name));
                            let arg = m
                                .get(&Value::Text("arg".to_string()))
                                .ok_or_else(|| anyhow!("Invalid cbor content"))?;
                            if let Value::Bytes(arg) = arg {
                                return Ok((
                                    sender,
                                    canister_id,
                                    method_name.to_string().to_string(),
                                    get_idl_string(arg, "pp", &method_type)?,
                                ));
                            }
                        }
                    }
                }
            }
        }

        bail!("Invalid cbor content");
    }
}
