use crate::lib::environment::Environment;
use crate::lib::sign::sign_transport::SignReplicaV2Transport;
use crate::lib::sign::sign_transport::SignedMessageWithRequestId;
use crate::lib::DfxResult;
use anyhow::{anyhow, Context};
use clap::Clap;
use ic_agent::{AgentError, RequestId};
use ic_types::Principal;
use std::str::FromStr;

/// Requests the status of a specified call from a canister.
#[derive(Clap)]
pub struct RequestStatusSignOpts {
    /// Specifies the request identifier.
    /// The request identifier is an hexadecimal string starting with 0x.
    #[clap(validator(is_request_id))]
    pub request_id: String,
}

pub async fn exec(env: &dyn Environment, opts: RequestStatusSignOpts) -> DfxResult<String> {
    let canister_id = Principal::from_text(crate::lib::nns_types::LEDGER_CANISTER_ID)
        .expect("Couldn't parse canister id");
    let request_id =
        RequestId::from_str(&opts.request_id[2..]).context("Invalid argument: request_id")?;

    let mut agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let data = SignedMessageWithRequestId::new();
    data.write().unwrap().request_id = Some(request_id);
    let transport = SignReplicaV2Transport { data: data.clone() };
    agent.set_transport(transport);
    match agent.request_status_raw(&request_id, canister_id).await {
        Err(AgentError::MissingReplicaTransport()) => {
            return Ok(data.read().unwrap().buffer.clone());
        }
        val => panic!("Unexpected output from the signing agent: {:?}", val),
    }
}

pub fn is_request_id(v: &str) -> Result<(), String> {
    // A valid Request Id starts with `0x` and is a series of 64 hexadecimals.
    if !v.starts_with("0x") {
        Err(String::from("A Request ID needs to start with 0x."))
    } else if v.len() != 66 {
        Err(String::from(
            "A Request ID is 64 hexadecimal prefixed with 0x.",
        ))
    } else if v[2..].contains(|c: char| !c.is_ascii_hexdigit()) {
        Err(String::from(
            "A Request ID is 64 hexadecimal prefixed with 0x. An invalid character was found.",
        ))
    } else {
        Ok(())
    }
}
