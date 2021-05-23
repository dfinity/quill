use crate::lib::{identity::Identity as NanoIdentity, DfxResult, NetworkDescriptor};
use ic_agent::{Agent, Identity};
use ic_types::Principal;
use std::time::Duration;

pub trait Environment {
    fn get_pem(&self) -> Option<String>;

    fn get_agent(&self) -> Option<Agent>;

    fn get_network_descriptor(&self) -> NetworkDescriptor;

    fn get_selected_identity_principal(&self) -> Option<Principal>;
}

pub struct EnvironmentImpl {
    pem: Option<String>,
}

impl EnvironmentImpl {
    pub fn new(pem: Option<String>) -> DfxResult<Self> {
        Ok(EnvironmentImpl { pem })
    }
}

impl Environment for EnvironmentImpl {
    fn get_pem(&self) -> Option<String> {
        self.pem.clone()
    }

    fn get_agent(&self) -> Option<Agent> {
        let url = self.get_network_descriptor().providers[0].clone();
        let identity = Box::new(NanoIdentity::load(match self.pem.clone() {
            Some(pem) => pem,
            None => {
                eprintln!("No PEM provided, quitting.");
                return None;
            }
        }));
        // 5 minutes is max ingress timeout
        let timeout = Duration::from_secs(60 * 5);
        Agent::builder()
            .with_transport(
                ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create(url)
                    .unwrap(),
            )
            .with_boxed_identity(identity)
            .with_ingress_expiry(Some(timeout))
            .build()
            .ok()
    }

    fn get_network_descriptor(&self) -> NetworkDescriptor {
        NetworkDescriptor {
            name: "ic".to_string(),
            providers: vec!["https://ic0.app".to_string()],
            is_ic: true,
        }
    }

    fn get_selected_identity_principal(&self) -> Option<Principal> {
        self.pem
            .clone()
            .and_then(move |pem| NanoIdentity::load(pem.clone()).as_ref().sender().ok())
    }
}
