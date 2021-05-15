//! Identity type and module.
//!
//! Wallets are a map of network-identity, but don't have their own types or manager
//! type.
use ic_agent::identity::{BasicIdentity, Secp256k1Identity};
use ic_agent::Signature;
use ic_types::Principal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

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
    /// Inner implementation of this identity.
    inner: Box<dyn ic_agent::Identity + Sync + Send>,
}

impl Identity {
    fn load_basic_identity(pem_path: &PathBuf) -> Option<Self> {
        let inner = Box::new(BasicIdentity::from_pem_file(&pem_path).ok()?);
        Some(Self { inner })
    }

    fn load_secp256k1_identity(pem_path: &PathBuf) -> Option<Self> {
        let inner = Box::new(Secp256k1Identity::from_pem_file(&pem_path).ok()?);
        Some(Self { inner })
    }

    pub fn load(pem_path: &PathBuf) -> Self {
        Identity::load_secp256k1_identity(pem_path)
            .or_else(|| Identity::load_basic_identity(pem_path))
            .expect("Couldn't load identity from file")
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
