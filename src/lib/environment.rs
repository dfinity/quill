use crate::config::Config;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity as NanoIdentity;
use crate::lib::network::NetworkDescriptor;
use anyhow::anyhow;
use ic_agent::{Agent, Identity};
use ic_types::Principal;
use semver::Version;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

pub trait Environment {
    fn get_config(&self) -> Option<Arc<Config>>;
    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>>;

    fn get_pem_file(&self) -> &PathBuf;

    fn is_in_project(&self) -> bool;
    /// Return a temporary directory for configuration if none exists
    /// for the current project or if not in a project. Following
    /// invocations by other processes in the same project should
    /// return the same configuration directory.
    fn get_temp_dir(&self) -> &Path;
    /// Return the directory where state for replica(s) is kept.
    fn get_state_dir(&self) -> PathBuf;
    fn get_version(&self) -> &Version;

    // Explicit lifetimes are actually needed for mockall to work properly.
    #[allow(clippy::needless_lifetimes)]
    fn get_agent<'a>(&'a self) -> Option<&'a Agent>;

    #[allow(clippy::needless_lifetimes)]
    fn get_network_descriptor<'a>(&'a self) -> Option<&'a NetworkDescriptor>;

    fn get_selected_identity(&self) -> Option<&String>;

    fn get_selected_identity_principal(&self) -> Option<Principal>;
}

pub struct EnvironmentImpl {
    config: Option<Arc<Config>>,
    temp_dir: PathBuf,
    version: Version,
    pem_file: PathBuf,
}

impl EnvironmentImpl {
    pub fn new(pem_file: PathBuf) -> DfxResult<Self> {
        let config = match Config::from_current_dir() {
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
            Ok(x) => Ok(Some(x)),
        }?;
        let temp_dir = match &config {
            None => tempfile::tempdir()
                .expect("Could not create a temporary directory.")
                .into_path(),
            Some(c) => c.get_path().parent().unwrap().join(".dfx"),
        };
        create_dir_all(&temp_dir)?;

        Ok(EnvironmentImpl {
            config: config.map(Arc::new),
            temp_dir,
            version: Version::parse("0.1.0").unwrap(),
            pem_file,
        })
    }
}

impl Environment for EnvironmentImpl {
    fn get_pem_file(&self) -> &PathBuf {
        &self.pem_file
    }

    fn get_config(&self) -> Option<Arc<Config>> {
        self.config.as_ref().map(|x| Arc::clone(x))
    }

    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>> {
        self.get_config().ok_or_else(|| anyhow!(
            "Cannot find dfx configuration file in the current working directory. Did you forget to create one?"
        ))
    }

    fn is_in_project(&self) -> bool {
        self.config.is_some()
    }

    fn get_temp_dir(&self) -> &Path {
        &self.temp_dir
    }

    fn get_state_dir(&self) -> PathBuf {
        self.get_temp_dir().join("state")
    }

    fn get_version(&self) -> &Version {
        &self.version
    }

    fn get_agent(&self) -> Option<&Agent> {
        // create an AgentEnvironment explicitly, in order to specify network and agent.
        // See install, build for examples.
        None
    }

    fn get_network_descriptor(&self) -> Option<&NetworkDescriptor> {
        // create an AgentEnvironment explicitly, in order to specify network and agent.
        // See install, build for examples.
        None
    }

    fn get_selected_identity(&self) -> Option<&String> {
        None
    }

    fn get_selected_identity_principal(&self) -> Option<Principal> {
        None
    }
}

pub struct AgentEnvironment<'a> {
    backend: &'a dyn Environment,
    agent: Agent,
    network_descriptor: NetworkDescriptor,
    pem_file: PathBuf,
}

impl<'a> AgentEnvironment<'a> {
    pub fn new(
        backend: &'a dyn Environment,
        network_descriptor: NetworkDescriptor,
        pem_file: &PathBuf,
        timeout: Duration,
    ) -> DfxResult<Self> {
        let identity = Box::new(NanoIdentity::load(pem_file));
        let agent_url = network_descriptor.providers.first().unwrap();
        Ok(AgentEnvironment {
            backend,
            agent: create_agent(agent_url, identity, timeout).expect("Failed to construct agent."),
            network_descriptor,
            pem_file: pem_file.clone(),
        })
    }
}

impl<'a> Environment for AgentEnvironment<'a> {
    fn get_pem_file(&self) -> &PathBuf {
        &self.pem_file
    }
    fn get_config(&self) -> Option<Arc<Config>> {
        self.backend.get_config()
    }

    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>> {
        self.get_config().ok_or_else(|| anyhow!(
            "Cannot find dfx configuration file in the current working directory. Did you forget to create one?"
        ))
    }

    fn is_in_project(&self) -> bool {
        self.backend.is_in_project()
    }

    fn get_temp_dir(&self) -> &Path {
        self.backend.get_temp_dir()
    }

    fn get_state_dir(&self) -> PathBuf {
        self.backend.get_state_dir()
    }

    fn get_version(&self) -> &Version {
        self.backend.get_version()
    }

    fn get_agent(&self) -> Option<&Agent> {
        Some(&self.agent)
    }

    fn get_network_descriptor(&self) -> Option<&NetworkDescriptor> {
        Some(&self.network_descriptor)
    }

    fn get_selected_identity(&self) -> Option<&String> {
        unimplemented!()
    }

    fn get_selected_identity_principal(&self) -> Option<Principal> {
        NanoIdentity::load(&self.pem_file).as_ref().sender().ok()
    }
}

pub struct AgentClient {}

impl AgentClient {
    pub fn new() -> DfxResult<AgentClient> {
        Ok(Self {})
    }
}

fn create_agent(
    url: &str,
    identity: Box<dyn Identity + Send + Sync>,
    timeout: Duration,
) -> Option<Agent> {
    AgentClient::new().ok().and_then(|_| {
        Agent::builder()
            .with_transport(
                ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create(url)
                    .unwrap(),
            )
            .with_boxed_identity(identity)
            .with_ingress_expiry(Some(timeout))
            .build()
            .ok()
    })
}
