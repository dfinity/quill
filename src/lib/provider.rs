use crate::config::NetworkType;
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::DfxResult;
use crate::lib::network::NetworkDescriptor;
use crate::util::expiry_duration;
use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};

lazy_static! {
    static ref NETWORK_CONTEXT: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
}

fn set_network_context(network: Option<String>) {
    let name = network.unwrap_or_else(|| "ic".to_string());

    let mut n = NETWORK_CONTEXT.write().unwrap();
    *n = Some(name);
}

// always returns at least one url
pub fn get_network_descriptor<'a>(
    _env: &'a (dyn Environment + 'a),
    network: Option<String>,
) -> DfxResult<NetworkDescriptor> {
    set_network_context(network);
    Ok(NetworkDescriptor {
        name: "ic".to_string(),
        providers: vec!["https://ic0.app".to_string()],
        r#type: NetworkType::Persistent,
        is_ic: true,
    })
}

pub fn create_agent_environment<'a>(
    env: &'a (dyn Environment + 'a),
    network: Option<String>,
) -> DfxResult<AgentEnvironment<'a>> {
    let network_descriptor = get_network_descriptor(env, network)?;
    let timeout = expiry_duration();
    AgentEnvironment::new(env, network_descriptor, env.get_pem(), timeout)
}
