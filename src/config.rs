use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::default::Default;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE_NAME: &str = "dfx.json";

/// A Canister configuration in the dfx.json config file.
/// It only contains a type; everything else should be infered using the
/// CanisterInfo type.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigCanistersCanister {
    pub r#type: Option<String>,

    #[serde(flatten)]
    pub extras: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigDefaultsBootstrap {
    pub ip: Option<IpAddr>,
    pub port: Option<u16>,
    pub timeout: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigDefaultsBuild {
    pub packtool: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigDefaultsReplica {
    pub message_gas_limit: Option<u64>,
    pub port: Option<u16>,
    pub round_gas_limit: Option<u64>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkType {
    // We store ephemeral canister ids in .dfx/{network}/canister_ids.json
    Ephemeral,

    // We store persistent canister ids in canister_ids.json (adjacent to dfx.json)
    Persistent,
}

impl Default for NetworkType {
    // This is just needed for the Default trait on NetworkType,
    // but nothing will ever call it, due to field defaults.
    fn default() -> Self {
        NetworkType::Ephemeral
    }
}

impl NetworkType {
    fn ephemeral() -> Self {
        NetworkType::Ephemeral
    }
    fn persistent() -> Self {
        NetworkType::Persistent
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigNetworkProvider {
    pub providers: Vec<String>,

    #[serde(default = "NetworkType::persistent")]
    pub r#type: NetworkType,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ConfigLocalProvider {
    pub bind: String,

    #[serde(default = "NetworkType::ephemeral")]
    pub r#type: NetworkType,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ConfigNetwork {
    ConfigNetworkProvider(ConfigNetworkProvider),
    ConfigLocalProvider(ConfigLocalProvider),
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Profile {
    // debug is for development only
    Debug,
    // release is for production
    Release,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigDefaults {
    pub bootstrap: Option<ConfigDefaultsBootstrap>,
    pub build: Option<ConfigDefaultsBuild>,
    pub replica: Option<ConfigDefaultsReplica>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigInterface {
    pub profile: Option<Profile>,
    pub version: Option<u32>,
    pub dfx: Option<String>,
    pub canisters: Option<BTreeMap<String, ConfigCanistersCanister>>,
    pub defaults: Option<ConfigDefaults>,
    pub networks: Option<BTreeMap<String, ConfigNetwork>>,
}

impl ConfigCanistersCanister {}

impl ConfigDefaultsBuild {}

impl ConfigDefaults {}

impl ConfigInterface {}

#[derive(Clone)]
pub struct Config {
    path: PathBuf,
    json: Value,
    // public interface to the config:
    pub config: ConfigInterface,
}

impl Config {
    pub fn resolve_config_path(working_dir: &Path) -> Result<PathBuf, std::io::Error> {
        let mut curr = PathBuf::from(working_dir).canonicalize()?;
        while curr.parent().is_some() {
            if curr.join(CONFIG_FILE_NAME).is_file() {
                return Ok(curr.join(CONFIG_FILE_NAME));
            } else {
                curr.pop();
            }
        }

        // Have to check if the config could be in the root (e.g. on VMs / CI).
        if curr.join(CONFIG_FILE_NAME).is_file() {
            return Ok(curr.join(CONFIG_FILE_NAME));
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Config not found.",
        ))
    }

    pub fn from_file(path: &Path) -> std::io::Result<Config> {
        let content = std::fs::read(&path)?;
        Config::from_slice(path.to_path_buf(), &content)
    }

    pub fn from_dir(working_dir: &Path) -> std::io::Result<Config> {
        let path = Config::resolve_config_path(working_dir)?;
        Config::from_file(&path)
    }

    pub fn from_current_dir() -> std::io::Result<Config> {
        Config::from_dir(&std::env::current_dir()?)
    }

    fn from_slice(path: PathBuf, content: &[u8]) -> std::io::Result<Config> {
        let config = serde_json::from_slice(&content)?;
        let json = serde_json::from_slice(&content)?;
        Ok(Config { path, json, config })
    }

    #[cfg(test)]
    pub fn from_str_and_path(path: PathBuf, content: &str) -> std::io::Result<Config> {
        Config::from_slice(path, content.as_bytes())
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
}
