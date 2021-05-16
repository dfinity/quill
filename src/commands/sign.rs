use crate::lib::environment::Environment;
use crate::lib::get_local_candid;
use crate::lib::sign::sign_transport::SignReplicaV2Transport;
use crate::lib::sign::signed_message::SignedMessageV1;
use crate::lib::DfxResult;
use crate::lib::{blob_from_arguments, get_candid_type};
use anyhow::{anyhow, bail};
use chrono::Utc;
use clap::Clap;
use humanize_rs::duration;
use ic_types::principal::Principal;
use std::option::Option;
use std::time::SystemTime;

/// Sign a canister call and generate message file in json
#[derive(Clap)]
pub struct SignOpts {
    /// Specifies the name of the canister to call.
    pub canister_name: String,

    /// Specifies the method name to call on the canister.
    pub method_name: String,

    /// Sends a query request to a canister.
    #[clap(long)]
    pub query: bool,

    /// Sends an update request to a canister. This is the default if the method is not a query method.
    #[clap(long, conflicts_with("query"))]
    pub update: bool,

    /// Specifies the argument to pass to the method.
    pub argument: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    pub r#type: Option<String>,

    /// Specifies how long will the message be valid in seconds, default to be 300s (5 minutes)
    #[clap(long, default_value("5m"))]
    pub expire_after: String,
}

pub async fn exec(env: &dyn Environment, opts: SignOpts) -> DfxResult {
    let callee_canister = opts.canister_name.as_str();
    let method_name = opts.method_name.as_str();

    let canister_id =
        Principal::from_text(callee_canister).expect("Coouldn't convert canister id to principal");
    let spec = get_local_candid(canister_id.clone());

    let method_type = spec.and_then(|spec| get_candid_type(spec, method_name));
    let is_query_method = match &method_type {
        Some((_, f)) => Some(f.is_query()),
        None => None,
    };

    let is_query = match is_query_method {
        Some(true) => !opts.update,
        Some(false) => {
            if opts.query {
                bail!(
                    "Invalid method call: {} is not a query method.",
                    method_name
                );
            } else {
                false
            }
        }
        None => opts.query,
    };

    // Get the argument, get the type, convert the argument to the type and return
    // an error if any of it doesn't work.
    let arg_value = {
        let arguments = opts.argument.as_deref();
        let arg_type = opts.r#type.as_deref();
        blob_from_arguments(arguments, arg_type, &method_type)?
    };
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let network = env
        .get_network_descriptor()
        .providers
        .first()
        .expect("Cannot get network provider (url).")
        .to_string();

    let sender = env
        .get_selected_identity_principal()
        .expect("Selected identity not instantiated.");

    let timeout = duration::parse(&opts.expire_after)
        .map_err(|_| anyhow!("Cannot parse expire_after as a duration (e.g. `1h`, `1h 30m`)"))?;
    //let timeout = Duration::from_secs(opts.expire_after);
    let expiration_system_time = SystemTime::now()
        .checked_add(timeout)
        .ok_or_else(|| anyhow!("Time wrapped around."))?;
    let chorono_timeout = chrono::Duration::seconds(timeout.as_secs() as i64);
    let creation = Utc::now();
    let expiration = creation
        .checked_add_signed(chorono_timeout)
        .ok_or_else(|| anyhow!("Expiration datetime overflow."))?;

    let message_template = SignedMessageV1::new(
        creation,
        expiration,
        network,
        sender,
        canister_id.clone(),
        method_name.to_string(),
        arg_value.clone(),
    );

    let mut sign_agent = agent.clone();
    sign_agent.set_transport(SignReplicaV2Transport::new(message_template));

    let canister_id = Principal::from_text(opts.canister_name)?;

    if is_query {
        sign_agent
            .query(&canister_id, method_name)
            .with_effective_canister_id(canister_id)
            .with_arg(&arg_value)
            .expire_at(expiration_system_time)
            .call()
            .await
            .map(|_| ())
            .map_err(|e| anyhow!(e))
    } else {
        sign_agent
            .update(&canister_id, method_name)
            .with_effective_canister_id(canister_id)
            .with_arg(&arg_value)
            .expire_at(expiration_system_time)
            .call()
            .await
            .map(|_| ())
            .map_err(|e| anyhow!(e))
    }
}
