use crate::commands::request_status;
use crate::lib::environment::Environment;
use crate::lib::sign::signed_message::SignedMessage;
use crate::lib::DfxResult;
use anyhow::anyhow;
use clap::Clap;
use ic_agent::agent::ReplicaV2Transport;
use ic_agent::{agent::http_transport::ReqwestHttpReplicaV2Transport, RequestId};
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

    if let Ok(val) = serde_json::from_str::<SignedMessage>(&json) {
        messages.push(val);
    } else if let Ok(val) = serde_json::from_str::<Vec<SignedMessage>>(&json) {
        messages.extend_from_slice(&val);
    } else {
        return Err(anyhow!("Invalid JSON content"));
    }

    for msg in messages {
        send(env, msg, opts.dry_run).await?;
    }

    Ok(())
}

async fn send(env: &dyn Environment, message: SignedMessage, dry_run: bool) -> DfxResult {
    let (sender, canister_id, method_name, args) = message.parse()?;

    eprintln!("  Call type:   {}", message.call_type);
    eprintln!("  Sender:      {}", sender);
    eprintln!("  Canister id: {}", canister_id);
    eprintln!("  Method name: {}", method_name);
    eprintln!("  Arguments:   {}", args);

    if dry_run {
        return Ok(());
    }

    // Not using dialoguer because it doesn't support non terminal env like bats e2e
    eprintln!("\nDo you want to send this message? [y/N]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !["y", "yes"].contains(&input.to_lowercase().trim()) {
        return Ok(());
    }

    let network = env
        .get_network_descriptor()
        .providers
        .first()
        .expect("Cannot get network provider (url).")
        .to_string();
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
