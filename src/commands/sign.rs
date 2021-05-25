use crate::lib::environment::Environment;
use crate::lib::get_local_candid;
use crate::lib::sign::sign_transport::SignReplicaV2Transport;
use crate::lib::sign::sign_transport::SignedMessageWithRequestId;
use crate::lib::DfxResult;
use crate::lib::{blob_from_arguments, get_candid_type};
use anyhow::{anyhow, bail};
use clap::Clap;
use ic_agent::AgentError;
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
}

pub async fn exec(env: &dyn Environment, opts: SignOpts) -> DfxResult<SignedMessageWithRequestId> {
    let callee_canister = opts.canister_name.as_str();
    let method_name = opts.method_name.as_str();

    let spec = get_local_candid(callee_canister);

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
    let mut sign_agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let timeout = std::time::Duration::from_secs(5 * 60);
    let expiration_system_time = SystemTime::now()
        .checked_add(timeout)
        .ok_or_else(|| anyhow!("Time wrapped around."))?;

    let data = SignedMessageWithRequestId::new();
    let transport = SignReplicaV2Transport { data: data.clone() };
    sign_agent.set_transport(transport);

    let canister_id = Principal::from_text(opts.canister_name)?;

    if is_query {
        match sign_agent
            .query(&canister_id, method_name)
            .with_effective_canister_id(canister_id)
            .with_arg(&arg_value)
            .expire_at(expiration_system_time)
            .call()
            .await
        {
            Err(AgentError::MissingReplicaTransport()) => {}
            val => panic!("Unexpected return value from query execution: {:?}", val),
        };
    } else {
        sign_agent
            .update(&canister_id, method_name)
            .with_effective_canister_id(canister_id)
            .with_arg(&arg_value)
            .expire_at(expiration_system_time)
            .call()
            .await
            .map(|_| ())
            .map_err(|e| anyhow!(e))?;
    }

    let data = data.read().unwrap().clone();
    Ok(data)
}
