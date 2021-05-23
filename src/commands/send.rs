use crate::commands::request_status;
use crate::lib::environment::Environment;
use crate::lib::sign::signed_message::SignedMessageV1;
use crate::lib::DfxResult;
use crate::lib::{get_candid_type, get_idl_string, get_local_candid};
use anyhow::anyhow;
use clap::Clap;
use ic_agent::agent::ReplicaV2Transport;
use ic_agent::{agent::http_transport::ReqwestHttpReplicaV2Transport, RequestId};
use ic_types::Principal;
use std::{fs::File, path::Path};
use std::{io::Read, str::FromStr};

/// Send a signed message
#[derive(Clap)]
pub struct SendOpts {
    /// Path to the signed message
    file_name: String,

    /// Will display the signed message, but not send it.
    #[clap(long)]
    dry_run: bool,
}

pub async fn exec(env: &dyn Environment, opts: SendOpts) -> DfxResult {
    let file_name = opts.file_name;
    let path = Path::new(&file_name);
    let mut file = File::open(&path).map_err(|_| anyhow!("Message file doesn't exist"))?;
    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| anyhow!("Cannot read the message file."))?;

    let mut messages = Vec::new();

    if let Ok(val) = serde_json::from_str::<SignedMessageV1>(&json) {
        messages.push(val);
    } else if let Ok(val) = serde_json::from_str::<Vec<SignedMessageV1>>(&json) {
        messages.extend_from_slice(&val);
    } else {
        return Err(anyhow!("Invalid JSON content"));
    }

    for msg in messages {
        send(env, msg, opts.dry_run).await?;
    }

    Ok(())
}

async fn send(env: &dyn Environment, message: SignedMessageV1, dry_run: bool) -> DfxResult {
    message.validate()?;

    let canister_id = Principal::from_text(&message.canister_id)?;
    let spec = get_local_candid(&message.canister_id);
    let method_type = spec.and_then(|spec| get_candid_type(spec, &message.method_name));

    eprintln!("Will send message:");
    eprintln!("  Creation:    {}", message.creation);
    eprintln!("  Expiration:  {}", message.expiration);
    eprintln!("  Network:     {}", message.network);
    eprintln!("  Call type:   {}", message.call_type);
    eprintln!("  Sender:      {}", message.sender);
    eprintln!("  Canister id: {}", message.canister_id);
    eprintln!("  Method name: {}", message.method_name);
    eprintln!(
        "  Arguments:   {}",
        get_idl_string(&message.arg, "pp", &method_type)?
    );

    if dry_run {
        return Ok(());
    }

    // Not using dialoguer because it doesn't support non terminal env like bats e2e
    eprintln!("\nOkay? [y/N]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !["y", "yes"].contains(&input.to_lowercase().trim()) {
        return Ok(());
    }

    let network = message.network;
    let transport = ReqwestHttpReplicaV2Transport::create(network)?;
    let content = hex::decode(&message.content)?;

    match message.call_type.as_str() {
        "query" => {
            let response = transport.query(canister_id, content).await?;
            eprintln!(
                "To see the content of response, copy-paste the encoded string into cbor.me."
            );
            eprint!("Response: ");
            println!("{}", hex::encode(response));
            Ok(())
        }
        "update" => {
            let request_id = RequestId::from_str(
                &message
                    .request_id
                    .expect("Cannot get request_id from the update message"),
            )?;
            transport.call(canister_id, content, request_id).await?;
            let request_id = format!("0x{}", String::from(request_id));
            eprintln!(
                "Received the request ID: {}; checking for status...",
                request_id
            );
            request_status::exec(env, request_status::RequestStatusOpts { request_id }).await
        }
        // message.validate() guarantee that call_type must be query or update
        _ => unreachable!(),
    }
}
