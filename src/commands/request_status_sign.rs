use crate::lib::sign::sign_transport::SignReplicaV2Transport;
use crate::lib::DfxResult;
use crate::lib::{environment::Environment, sign::signed_message::SignedStatusRequest};
use anyhow::{anyhow, Context};
use clap::Clap;
use ic_agent::{AgentError, RequestId};
use ic_types::Principal;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

/// Requests the status of a specified call from a canister.
#[derive(Clap)]
pub struct RequestStatusSignOpts {
    /// Specifies the request identifier.
    /// The request identifier is an hexadecimal string starting with 0x.
    #[clap(validator(is_request_id))]
    pub request_id: String,
}

pub async fn exec(env: &dyn Environment, opts: RequestStatusSignOpts) -> DfxResult {
    let canister_id = Principal::from_text(crate::lib::nns_types::LEDGER_CANISTER_ID)
        .expect("Couldn't parse canister id");
    let request_id =
        RequestId::from_str(&opts.request_id[2..]).context("Invalid argument: request_id")?;

    let mut agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let buffer = Arc::new(RwLock::new(String::new()));
    let req = SignedStatusRequest {
        canister_id: canister_id.to_string(),
        request_id: request_id.into(),
        content: "".to_owned(),
    };
    let transport = SignReplicaV2Transport::new(buffer.clone(), Some(req));
    agent.set_transport(transport);
    match agent.request_status_raw(&request_id, canister_id).await {
        Err(AgentError::MissingReplicaTransport()) => {
            println!("{}", *buffer.read().unwrap());
            return Ok(());
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
