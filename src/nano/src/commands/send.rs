use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::get_local_candid_path;
use crate::lib::sign::signed_message::SignedMessageV1;
use crate::util::get_candid_type;
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
    /// Specifies the file name of the message
    file_name: String,
}

pub async fn exec(_env: &dyn Environment, opts: SendOpts) -> DfxResult {
    let file_name = opts.file_name;
    let path = Path::new(&file_name);
    let mut file = File::open(&path).map_err(|_| anyhow!("Message file doesn't exist."))?;
    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| anyhow!("Cannot read the message file."))?;
    let message: SignedMessageV1 =
        serde_json::from_str(&json).map_err(|_| anyhow!("Invalid json message."))?;
    message.validate()?;

    let canister_id = Principal::from_text(&message.canister_id)?;
    let path = get_local_candid_path(canister_id.clone());
    let method_type = path.and_then(|path| get_candid_type(&path, &message.method_name));

    eprintln!("Will send message:");
    eprintln!("  Creation:    {}", message.creation);
    eprintln!("  Expiration:  {}", message.expiration);
    eprintln!("  Network:     {}", message.network);
    eprintln!("  Call type:   {}", message.call_type);
    eprintln!("  Sender:      {}", message.sender);
    eprintln!("  Canister id: {}", message.canister_id);
    eprintln!("  Method name: {}", message.method_name);
    eprintln!(
        "  Arg:         {:?}",
        match &method_type {
            None => candid::IDLArgs::from_bytes(&message.arg),
            Some((env, func)) =>
                candid::IDLArgs::from_bytes_with_types(&message.arg, &env, &func.rets),
        }
    );

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
        }
        "update" => {
            let request_id = RequestId::from_str(
                &message
                    .request_id
                    .expect("Cannot get request_id from the update message"),
            )?;
            transport.call(canister_id, content, request_id).await?;
            eprint!("Request ID: ");
            println!("0x{}", String::from(request_id));
        }
        // message.validate() guarantee that call_type must be query or update
        _ => unreachable!(),
    }
    Ok(())
}
