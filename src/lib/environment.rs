use crate::lib::{AnyhowResult, NetworkDescriptor};
use ic_agent::{
    identity::{BasicIdentity, Secp256k1Identity},
    Agent, Identity,
};
use std::time::Duration;

pub trait Environment {
    fn get_pem(&self) -> Option<String>;

    fn get_agent(&self) -> Option<Agent>;

    fn get_network_descriptor(&self) -> NetworkDescriptor;

    fn identity(&self) -> Option<Box<dyn ic_agent::Identity + Sync + Send>>;
}

pub struct EnvironmentImpl {
    pem: Option<String>,
}

impl EnvironmentImpl {
    pub fn new(pem: Option<String>) -> AnyhowResult<Self> {
        Ok(EnvironmentImpl { pem })
    }
}

impl Environment for EnvironmentImpl {
    fn get_pem(&self) -> Option<String> {
        self.pem.clone()
    }

    fn get_agent(&self) -> Option<Agent> {
        let url = self.get_network_descriptor().providers[0].clone();
        let timeout = Duration::from_secs(60 * 5);
        let builder = Agent::builder()
            .with_transport(
                ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create(url)
                    .unwrap(),
            )
            .with_ingress_expiry(Some(timeout));

        let builder = match self.identity() {
            Some(identity) => builder.with_boxed_identity(identity),
            None => builder,
        };

        builder.build().ok()
    }

    fn get_network_descriptor(&self) -> NetworkDescriptor {
        NetworkDescriptor {
            name: "ic".to_string(),
            providers: vec!["https://ic0.app".to_string()],
            is_ic: true,
        }
    }

    fn identity(&self) -> Option<Box<dyn Identity + Sync + Send>> {
        let pem = self.pem.clone()?;
        match Secp256k1Identity::from_pem(pem.as_bytes()) {
            Ok(identity) => return Some(Box::new(identity)),
            Err(_) => match BasicIdentity::from_pem(pem.as_bytes()) {
                Ok(identity) => return Some(Box::new(identity)),
                Err(_) => {
                    eprintln!("Couldn't load identity from PEM file");
                    std::process::exit(1);
                }
            },
        }
    }
}
