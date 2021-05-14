//! Identity type and module.
//!
//! Wallets are a map of network-identity, but don't have their own types or manager
//! type.
use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult, IdentityError};
use anyhow::{anyhow, bail, Context};
use ic_agent::identity::{BasicIdentity, Secp256k1Identity};
use ic_agent::Signature;
use ic_identity_hsm::HardwareIdentity;
use ic_types::Principal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;

pub mod identity_manager;
pub use identity_manager::{
    HardwareIdentityConfiguration, IdentityConfiguration, IdentityCreationParameters,
    IdentityManager,
};

const IDENTITY_PEM: &str = "identity.pem";
const WALLET_CONFIG_FILENAME: &str = "wallets.json";
const HSM_SLOT_INDEX: usize = 0;

#[derive(Debug, Serialize, Deserialize)]
struct WalletNetworkMap {
    #[serde(flatten)]
    pub networks: BTreeMap<String, Principal>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WalletGlobalConfig {
    pub identities: BTreeMap<String, WalletNetworkMap>,
}

pub struct Identity {
    /// The name of this Identity.
    name: String,

    /// Inner implementation of this identity.
    inner: Box<dyn ic_agent::Identity + Sync + Send>,

    /// The root directory for this identity.
    pub dir: PathBuf,
}

impl Identity {
    pub fn create(
        manager: &IdentityManager,
        name: &str,
        parameters: IdentityCreationParameters,
    ) -> DfxResult {
        let identity_dir = manager.get_identity_dir_path(name);
        if identity_dir.exists() {
            bail!("Identity already exists.");
        }
        fn create(identity_dir: PathBuf) -> DfxResult {
            std::fs::create_dir_all(&identity_dir).context(format!(
                "Cannot create identity directory at '{0}'.",
                identity_dir.display(),
            ))
        };
        match parameters {
            IdentityCreationParameters::Pem() => {
                create(identity_dir)?;
                let pem_file = manager.get_identity_pem_path(name);
                identity_manager::generate_key(&pem_file)
            }
            IdentityCreationParameters::PemFile(src_pem_file) => {
                identity_manager::validate_pem_file(&src_pem_file)?;
                create(identity_dir)?;
                let dst_pem_file = manager.get_identity_pem_path(name);
                identity_manager::import_pem_file(&src_pem_file, &dst_pem_file)
            }
            IdentityCreationParameters::Hardware(parameters) => {
                create(identity_dir)?;
                let identity_configuration = IdentityConfiguration {
                    hsm: Some(parameters),
                };
                let json_file = manager.get_identity_json_path(name);
                identity_manager::write_identity_configuration(&json_file, &identity_configuration)
            }
        }
    }

    fn load_basic_identity(manager: &IdentityManager, name: &str) -> DfxResult<Self> {
        let dir = manager.get_identity_dir_path(name);
        let pem_path = dir.join(IDENTITY_PEM);
        let inner = Box::new(BasicIdentity::from_pem_file(&pem_path).map_err(|e| {
            DfxError::new(IdentityError::CannotReadIdentityFile(
                pem_path.clone(),
                Box::new(DfxError::new(e)),
            ))
        })?);

        Ok(Self {
            name: name.to_string(),
            inner,
            dir: manager.get_identity_dir_path(name),
        })
    }

    fn load_secp256k1_identity(manager: &IdentityManager, name: &str) -> DfxResult<Self> {
        let dir = manager.get_identity_dir_path(name);
        let pem_path = dir.join(IDENTITY_PEM);
        let inner = Box::new(Secp256k1Identity::from_pem_file(&pem_path).map_err(|e| {
            DfxError::new(IdentityError::CannotReadIdentityFile(
                pem_path.clone(),
                Box::new(DfxError::new(e)),
            ))
        })?);

        Ok(Self {
            name: name.to_string(),
            inner,
            dir: manager.get_identity_dir_path(name),
        })
    }

    fn load_hardware_identity(
        manager: &IdentityManager,
        name: &str,
        hsm: HardwareIdentityConfiguration,
    ) -> DfxResult<Self> {
        let inner = Box::new(
            HardwareIdentity::new(
                hsm.pkcs11_lib_path,
                HSM_SLOT_INDEX,
                &hsm.key_id,
                identity_manager::get_dfx_hsm_pin,
            )
            .map_err(DfxError::new)?,
        );
        Ok(Self {
            name: name.to_string(),
            inner,
            dir: manager.get_identity_dir_path(name),
        })
    }

    pub fn load(manager: &IdentityManager, name: &str) -> DfxResult<Self> {
        let json_path = manager.get_identity_json_path(name);
        if json_path.exists() {
            let hsm = identity_manager::read_identity_configuration(&json_path)?
                .hsm
                .ok_or_else(|| {
                    anyhow!("No HardwareIdentityConfiguration for IdentityConfiguration.")
                })?;
            Identity::load_hardware_identity(manager, name, hsm)
        } else {
            Identity::load_secp256k1_identity(manager, name)
                .or_else(|_| Identity::load_basic_identity(manager, name))
        }
    }

    /// Get the name of this identity.
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }

    fn rename_wallet_global_config_key(
        original_identity: &str,
        renamed_identity: &str,
        wallet_path: PathBuf,
    ) -> DfxResult {
        let mut buffer = Vec::new();
        std::fs::File::open(&wallet_path)?.read_to_end(&mut buffer)?;
        let mut config = serde_json::from_slice::<WalletGlobalConfig>(&buffer)?;
        let identities = &mut config.identities;
        let v = identities
            .remove(original_identity)
            .unwrap_or(WalletNetworkMap {
                networks: BTreeMap::new(),
            });
        identities.insert(renamed_identity.to_string(), v);
        std::fs::create_dir_all(wallet_path.parent().unwrap())?;
        std::fs::write(&wallet_path, &serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    // used for dfx identity rename foo bar
    pub fn map_wallets_to_renamed_identity(
        env: &dyn Environment,
        original_identity: &str,
        renamed_identity: &str,
    ) -> DfxResult {
        let persistent_wallet_path = get_config_dfx_dir_path()?
            .join("identity")
            .join(original_identity)
            .join(WALLET_CONFIG_FILENAME);
        if persistent_wallet_path.exists() {
            Identity::rename_wallet_global_config_key(
                original_identity,
                renamed_identity,
                persistent_wallet_path,
            )?;
        }
        let local_wallet_path = env
            .get_temp_dir()
            .join("local")
            .join(WALLET_CONFIG_FILENAME);
        if local_wallet_path.exists() {
            Identity::rename_wallet_global_config_key(
                original_identity,
                renamed_identity,
                local_wallet_path,
            )?;
        }
        Ok(())
    }
}

impl ic_agent::Identity for Identity {
    fn sender(&self) -> Result<Principal, String> {
        self.inner.sender()
    }

    fn sign(&self, blob: &[u8]) -> Result<Signature, String> {
        self.inner.sign(blob)
    }
}

impl AsRef<Identity> for Identity {
    fn as_ref(&self) -> &Identity {
        self
    }
}
