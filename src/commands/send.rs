use crate::commands::request_status;
use crate::lib::{
    get_ic_url, parse_query_response, read_from_file,
    signing::{Ingress, IngressWithRequestId},
    AnyhowResult, AuthInfo,
};
use anyhow::{anyhow, bail, Context};
use candid::{CandidType, Principal};
use clap::Parser;
use ic_agent::agent::Transport;
use ic_agent::{agent::http_transport::ReqwestTransport, RequestId};
use icp_ledger::{Subaccount, Tokens};
use serde::{Deserialize, Serialize};
use std::io::IsTerminal;
use std::path::PathBuf;
use std::str::FromStr;

use super::SendingOpts;

#[derive(
    Serialize,
    Deserialize,
    CandidType,
    Clone,
    Copy,
    Hash,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct Memo(pub u64);

#[derive(CandidType)]
pub struct TimeStamp {
    pub timestamp_nanos: u64,
}

#[derive(CandidType)]
pub struct SendArgs {
    pub memo: Memo,
    pub amount: Tokens,
    pub fee: Tokens,
    pub from_subaccount: Option<Subaccount>,
    pub to: String,
    pub created_at_time: Option<TimeStamp>,
}

/// Sends a signed message or a set of messages.
#[derive(Parser)]
pub struct SendOpts {
    /// Path to the signed message (`-` for stdin)
    file_name: Option<PathBuf>,

    #[clap(flatten)]
    sending_opts: SendingOpts,
}

#[tokio::main]
pub async fn exec(opts: SendOpts, fetch_root_key: bool) -> AnyhowResult {
    let file_name = if let Some(file_name) = &opts.file_name {
        file_name.as_path()
    } else if !std::io::stdin().is_terminal() {
        "-".as_ref()
    } else {
        bail!("File name must be provided if not being piped")
    };
    let json = read_from_file(file_name)?;
    if let Ok(val) = serde_json::from_str::<Ingress>(&json) {
        send(&val, &opts).await?;
    } else if let Ok(vals) = serde_json::from_str::<Vec<Ingress>>(&json) {
        for msg in vals {
            send(&msg, &opts).await?;
        }
    } else if let Ok(vals) = serde_json::from_str::<Vec<IngressWithRequestId>>(&json) {
        for tx in vals {
            submit_ingress_and_check_status(&tx, &opts, fetch_root_key).await?;
        }
    } else {
        return Err(anyhow!("Invalid JSON content"));
    }
    Ok(())
}

pub async fn submit_unsigned_ingress(
    canister_id: Principal,
    role: &str,
    method_name: &str,
    args: Vec<u8>,
    sending_opts: SendingOpts,
    fetch_root_key: bool,
) -> AnyhowResult {
    let msg = crate::lib::signing::sign_ingress_with_request_status_query(
        &AuthInfo::NoAuth,
        canister_id,
        role,
        method_name,
        args,
    )?;
    submit_ingress_and_check_status(
        &msg,
        &SendOpts {
            file_name: None,
            sending_opts,
        },
        fetch_root_key,
    )
    .await
}

async fn submit_ingress_and_check_status(
    message: &IngressWithRequestId,
    opts: &SendOpts,
    fetch_root_key: bool,
) -> AnyhowResult {
    send(&message.ingress, opts).await?;
    if opts.sending_opts.dry_run {
        return Ok(());
    }
    let (_, _, method_name, _, role) = &message.ingress.parse()?;
    match request_status::submit(
        &message.request_status,
        Some(method_name.to_string()),
        role,
        opts.sending_opts.raw,
        fetch_root_key,
    )
    .await
    {
        Ok(result) => println!("{}", result.trim()),
        Err(err) => println!("{err}"),
    };
    Ok(())
}

async fn send(message: &Ingress, opts: &SendOpts) -> AnyhowResult {
    let (sender, canister_id, method_name, args, role) = message.parse()?;
    let call_type = &message.call_type;

    println!("Sending message with\n");
    println!("  Call type:   {call_type}");
    println!("  Sender:      {sender}");
    println!("  Canister id: {canister_id}");
    println!("  Method name: {method_name}");
    println!("  Arguments:   {args}");

    if opts.sending_opts.dry_run {
        return Ok(());
    }

    if message.call_type == "update" && !opts.sending_opts.yes {
        if !std::io::stdin().is_terminal() {
            eprintln!("Cannot confirm y/n if the input is being piped.");
            eprintln!("To confirm sending this message, rerun `quill send` with the `-y` flag.");
            std::process::exit(1);
        }
        println!("\nDo you want to send this message? [y/N]");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !["y", "yes"].contains(&input.to_lowercase().trim()) {
            std::process::exit(0);
        }
    }

    let transport = ReqwestTransport::create(get_ic_url())?;
    let content = hex::decode(&message.content)?;

    match message.call_type.as_str() {
        "query" => {
            let response = parse_query_response(
                ic_agent::agent::Transport::query(&transport, canister_id, content).await?,
                canister_id,
                &role,
                &method_name,
            )?;
            println!("Response: {response}");
        }
        "update" => {
            let request_id = RequestId::from_str(
                &message
                    .clone()
                    .request_id
                    .context("Cannot get request_id from the update message")?,
            )?;
            transport.call(canister_id, content, request_id).await?;
            let request_id = format!("0x{}", String::from(request_id));
            println!("Request ID: {request_id}");
        }
        _ => unreachable!(),
    }
    Ok(())
}
