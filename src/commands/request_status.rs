use crate::lib::{
    display_response, get_agent, get_idl_string, signing::RequestStatus, AnyhowResult, AuthInfo,
};
use anyhow::{anyhow, Context};
use candid::Principal;
use ic_agent::agent::{ReplyResponse, RequestStatusResponse};
use ic_agent::{AgentError, RequestId};
use std::str::FromStr;

pub async fn submit(
    req: &RequestStatus,
    method_name: Option<String>,
    role: &str,
    raw: bool,
    fetch_root_key: bool,
) -> AnyhowResult<String> {
    let canister_id =
        Principal::from_text(&req.canister_id).context("Couldn't parse canister id")?;
    let request_id =
        RequestId::from_str(&req.request_id).context("Invalid argument: request_id")?;
    let agent = get_agent(&AuthInfo::NoAuth)?;
    if fetch_root_key {
        agent.fetch_root_key().await?;
    }
    let envelope = hex::decode(&req.content)
        .context("Unable to decode request content (should be hexadecimal encoded)")?;
    let ReplyResponse { arg: blob } = async {
        loop {
            match agent
                .request_status_signed(&request_id, canister_id, envelope.clone())
                .await?
            {
                RequestStatusResponse::Replied(reply) => return Ok(reply),
                RequestStatusResponse::Rejected(response) => {
                    return Err(anyhow!(AgentError::CertifiedReject(response)))
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
    let method_str = method_name.unwrap_or_default();
    if raw {
        get_idl_string(&blob, canister_id, role, &method_str, "rets")
    } else {
        display_response(&blob, canister_id, role, &method_str, "rets").or_else(|e| {
            get_idl_string(&blob, canister_id, role, &method_str, "rets").map(|m| {
                format!("Error pretty-printing response: {e}. Falling back to IDL display\n{m}",)
            })
        })
    }
    .context("Invalid IDL blob.")
}
