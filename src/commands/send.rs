use crate::commands::request_status;
use crate::lib::{
    get_ic_url, parse_query_response, read_from_file,
    signing::{Ingress, IngressWithRequestId},
    AnyhowResult, TargetCanister,
};
use anyhow::{anyhow, Context, Error};
use candid::Decode;
use clap::Parser;
use ic_agent::agent::ReplicaV2Transport;
use ic_agent::{agent::http_transport::ReqwestHttpReplicaV2Transport, RequestId};
use ic_sns_governance::pb::v1::ManageNeuronResponse;
use ic_types::principal::Principal;
use ledger_canister::{BlockHeight, Tokens, TransferError};
use std::str::FromStr;

/// Sends a signed message or a set of messages.
#[derive(Parser)]
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
    target_canister: TargetCanister,
) -> AnyhowResult {
    let msg = crate::lib::signing::sign("", canister_id, method_name, args, target_canister)?;
    let ingress = msg.message;
    send(
        &ingress,
        &SendOpts {
            file_name: Default::default(),
            yes: false,
            dry_run,
        },
    )
    .await
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
    let result = request_status::submit(&message.request_status).await?;
    print_response(result, &method_name)?;
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
            let response = parse_query_response(transport.query(canister_id, content).await?)?;
            print_response(response, &method_name)?;
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
            println!("Request ID: {}", request_id);
        }
        _ => unreachable!(),
    }
    Ok(())
}

enum SupportedResponse {
    ManageNeuronResponse,
    TransferResponse,
    AccountBalanceResponse,
}

impl FromStr for SupportedResponse {
    type Err = ();

    fn from_str(input: &str) -> Result<SupportedResponse, Self::Err> {
        match input {
            "account_balance" => Ok(SupportedResponse::AccountBalanceResponse),
            "transfer" => Ok(SupportedResponse::TransferResponse),
            "manage_neuron" => Ok(SupportedResponse::ManageNeuronResponse),
            _ => Err(()),
        }
    }
}

fn print_response(blob: Vec<u8>, method_name: &String) -> AnyhowResult {
    let response_type = SupportedResponse::from_str(method_name.as_str())
        .map_err(|_| format!("{} is not a supported return type", method_name))
        .map_err(Error::msg)?;

    match response_type {
        SupportedResponse::AccountBalanceResponse => {
            let response = Decode!(blob.as_slice(), Tokens)?;
            println!("Response: {:?\n}", response);
        }
        SupportedResponse::TransferResponse => {
            let response = Decode!(blob.as_slice(), Result<BlockHeight, TransferError>)?;
            println!("Response: {:?\n}", response);
        }
        SupportedResponse::ManageNeuronResponse => {
            let response = Decode!(blob.as_slice(), ManageNeuronResponse)?;
            println!("Response: {:?\n}", response);
        }
    }

    Ok(())
}
