use crate::commands::request_status;
use crate::lib::sign::sign_transport::{Message, SignedMessageWithRequestId};
use crate::lib::{
    get_agent, get_candid_type, get_ic_url, get_local_candid, read_from_file,
    sign::sign_transport::SignReplicaV2Transport,
    sign::signed_message::{parse_query_response, Ingress, IngressWithRequestId},
    AnyhowResult,
};
use anyhow::anyhow;
use candid::CandidType;
use clap::Clap;
use ic_agent::agent::ReplicaV2Transport;
use ic_agent::AgentError;
use ic_agent::{agent::http_transport::ReqwestHttpReplicaV2Transport, RequestId};
use ic_types::principal::Principal;
use ledger_canister::{AccountIdentifier, ICPTs, Subaccount};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;

#[derive(
    Serialize, Deserialize, CandidType, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Memo(pub u64);

impl Default for Memo {
    fn default() -> Memo {
        Memo(0)
    }
}

#[derive(CandidType)]
pub struct TimeStamp {
    pub timestamp_nanos: u64,
}

#[derive(CandidType)]
pub struct SendArgs {
    pub memo: Memo,
    pub amount: ICPTs,
    pub fee: ICPTs,
    pub from_subaccount: Option<Subaccount>,
    pub to: AccountIdentifier,
    pub created_at_time: Option<TimeStamp>,
}

/// Sends a signed message or a set of messages.
#[derive(Clap)]
pub struct SendOpts {
    /// Path to the signed message
    file_name: String,

    /// Will display the signed message, but not send it.
    #[clap(long)]
    dry_run: bool,

    /// Skips confirmation and sends the message directly.
    #[clap(long)]
    yes: bool,
}

pub async fn exec(opts: SendOpts) -> AnyhowResult {
    let json = read_from_file(&opts.file_name)?;
    if let Ok(val) = serde_json::from_str::<Ingress>(&json) {
        send(&val, &opts).await?;
    } else if let Ok(vals) = serde_json::from_str::<Vec<Ingress>>(&json) {
        for msg in vals {
            send(&msg, &opts).await?;
        }
    } else if let Ok(vals) = serde_json::from_str::<Vec<IngressWithRequestId>>(&json) {
        for tx in vals {
            submit_ingress_and_check_status(&tx, &opts).await?;
        }
    } else {
        return Err(anyhow!("Invalid JSON content"));
    }
    Ok(())
}

pub async fn submit_unsigned_ingress(
    canister_id: Principal,
    method_name: &str,
    args: Vec<u8>,
    dry_run: bool,
) -> AnyhowResult {
    let spec = get_local_candid(canister_id)?;
    let method_type = get_candid_type(spec, method_name);
    let is_query = match &method_type {
        Some((_, f)) => f.is_query(),
        _ => false,
    };

    let mut sign_agent = get_agent(&None)?;
    let transport = SignReplicaV2Transport::new(None);
    let data = transport.data.clone();
    sign_agent.set_transport(transport);

    if !is_query {
        return Err(anyhow!("Unsigned ingress messages are not supported!"));
    }
    match sign_agent
        .query(&canister_id, method_name)
        .with_effective_canister_id(canister_id)
        .with_arg(&args)
        .call()
        .await
    {
        Err(AgentError::MissingReplicaTransport()) => {}
        val => {
            return Err(anyhow!(
                "Unexpected return value from query execution: {:?}",
                val
            ))
        }
    };

    let msg = SignedMessageWithRequestId::try_from(data.read().unwrap().clone())?;
    if let Message::Ingress(ingress) = msg.message {
        send(
            &ingress,
            &SendOpts {
                file_name: Default::default(),
                yes: false,
                dry_run,
            },
        )
        .await
    } else {
        Err(anyhow!("Unexpected ingress message generated"))
    }
}

async fn submit_ingress_and_check_status(
    message: &IngressWithRequestId,
    opts: &SendOpts,
) -> AnyhowResult {
    send(&message.ingress, opts).await?;
    if opts.dry_run {
        return Ok(());
    }
    let (_, _, method_name, _) = &message.ingress.parse()?;
    match request_status::submit(&message.request_status, Some(method_name.to_string())).await {
        Ok(result) => println!("{}\n", result),
        Err(err) => println!("{}\n", err),
    };
    Ok(())
}

async fn send(message: &Ingress, opts: &SendOpts) -> AnyhowResult {
    let (sender, canister_id, method_name, args) = message.parse()?;

    println!("Sending message with\n");
    println!("  Call type:   {}", message.call_type);
    println!("  Sender:      {}", sender);
    println!("  Canister id: {}", canister_id);
    println!("  Method name: {}", method_name);
    println!("  Arguments:   {}", args);

    if opts.dry_run {
        return Ok(());
    }

    if message.call_type == "update" && !opts.yes {
        println!("\nDo you want to send this message? [y/N]");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !["y", "yes"].contains(&input.to_lowercase().trim()) {
            std::process::exit(0);
        }
    }

    let transport = ReqwestHttpReplicaV2Transport::create(get_ic_url())?;
    let content = hex::decode(&message.content)?;

    match message.call_type.as_str() {
        "query" => {
            let response = parse_query_response(
                transport.query(canister_id, content).await?,
                canister_id,
                &method_name,
            )?;
            println!("Response: {}", response);
        }
        "update" => {
            let request_id = RequestId::from_str(
                &message
                    .clone()
                    .request_id
                    .expect("Cannot get request_id from the update message"),
            )?;
            transport.call(canister_id, content, request_id).await?;
            let request_id = format!("0x{}", String::from(request_id));
            println!("Request ID: {}", request_id);
        }
        _ => unreachable!(),
    }
    Ok(())
}
