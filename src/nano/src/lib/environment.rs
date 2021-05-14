use crate::config::dfinity::Config;
use crate::config::dfx_version;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::network::network_descriptor::NetworkDescriptor;

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

    fn is_in_project(&self) -> bool;
    /// Return a temporary directory for configuration if none exists
    /// for the current project or if not in a project. Following
    /// invocations by other processes in the same project should
    /// return the same configuration directory.
    fn get_temp_dir(&self) -> &Path;
    /// Return the directory where state for replica(s) is kept.
    fn get_state_dir(&self) -> PathBuf;
    fn get_version(&self) -> &Version;

    /// This is value of the name passed to dfx `--identity <name>`
    /// Notably, it is _not_ the name of the default identity or selected identity
    fn get_identity_override(&self) -> &Option<String>;

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

    identity_override: Option<String>,
}

impl EnvironmentImpl {
    pub fn new() -> DfxResult<Self> {
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

        // Figure out which version of DFX we should be running. This will use the following
        // fallback sequence:
        //   1. DFX_VERSION environment variable
        //   2. dfx.json "dfx" field
        //   3. this binary's version
        // If any of those are empty string, we stop the fallback and use the current version.
        // If any of those are a valid version, we try to use that directly as is.
        // If any of those are an invalid version, we will show an error to the user.
        let version = match std::env::var("DFX_VERSION") {
            Err(_) => match &config {
                None => dfx_version().clone(),
                Some(c) => match &c.get_config().get_dfx() {
                    None => dfx_version().clone(),
                    Some(v) => Version::parse(&v)?,
                },
            },
            Ok(v) => {
                if v.is_empty() {
                    dfx_version().clone()
                } else {
                    Version::parse(&v)?
                }
            }
        };

        Ok(EnvironmentImpl {
            config: config.map(Arc::new),
            temp_dir,
            version: version.clone(),
            identity_override: None,
        })
    }

    pub fn with_identity_override(mut self, identity: Option<String>) -> Self {
        self.identity_override = identity;
        self
    }
}

impl Environment for EnvironmentImpl {
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

    fn get_identity_override(&self) -> &Option<String> {
        &self.identity_override
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
    identity_manager: IdentityManager,
}

impl<'a> AgentEnvironment<'a> {
    pub fn new(
        backend: &'a dyn Environment,
        network_descriptor: NetworkDescriptor,
        timeout: Duration,
    ) -> DfxResult<Self> {
        let mut identity_manager = IdentityManager::new(backend)?;
        let identity = identity_manager.instantiate_selected_identity()?;

        let agent_url = network_descriptor.providers.first().unwrap();
        Ok(AgentEnvironment {
            backend,
            agent: create_agent(agent_url, identity, timeout).expect("Failed to construct agent."),
            network_descriptor,
            identity_manager,
        })
    }
}

impl<'a> Environment for AgentEnvironment<'a> {
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

    fn get_identity_override(&self) -> &Option<String> {
        self.backend.get_identity_override()
    }

    fn get_agent(&self) -> Option<&Agent> {
        Some(&self.agent)
    }

    fn get_network_descriptor(&self) -> Option<&NetworkDescriptor> {
        Some(&self.network_descriptor)
    }

    fn get_selected_identity(&self) -> Option<&String> {
        Some(self.identity_manager.get_selected_identity_name())
    }

    fn get_selected_identity_principal(&self) -> Option<Principal> {
        self.identity_manager.get_selected_identity_principal()
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
