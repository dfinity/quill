//! All the common functionality.

use anyhow::anyhow;
use bip39::Mnemonic;
use candid::{
    parser::typing::{check_prog, TypeEnv},
    types::Function,
    IDLProg,
};
use ic_agent::{
    identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity},
    Agent, Identity,
};
use ic_base_types::PrincipalId;
use ic_nns_constants::{GENESIS_TOKEN_CANISTER_ID, GOVERNANCE_CANISTER_ID, LEDGER_CANISTER_ID};
use ic_types::Principal;
use libsecp256k1::{PublicKey, SecretKey};
use pem::{encode, Pem};
use serde_cbor::Value;
use simple_asn1::ASN1Block::{
    BitString, Explicit, Integer, ObjectIdentifier, OctetString, Sequence,
};
use simple_asn1::{oid, to_der, ASN1Class, BigInt, BigUint};
use std::path::PathBuf;

pub const IC_URL: &str = "https://ic0.app";

pub fn get_ic_url() -> String {
    std::env::var("IC_URL").unwrap_or_else(|_| IC_URL.to_string())
}

pub mod hsm;
pub mod signing;

pub type AnyhowResult<T = ()> = anyhow::Result<T>;

#[derive(Debug)]
pub struct HSMInfo {
    pub libpath: PathBuf,
    pub slot: usize,
    pub ident: String,
    pin: std::cell::RefCell<Option<String>>,
}

#[cfg(target_os = "macos")]
const PKCS11_LIBPATH: &str = "/Library/OpenSC/lib/pkcs11/opensc-pkcs11.so";
#[cfg(target_os = "linux")]
const PKCS11_LIBPATH: &str = "/usr/lib/x86_64-linux-gnu/opensc-pkcs11.so";
#[cfg(target_os = "windows")]
const PKCS11_LIBPATH: &str = "C:/Program Files/OpenSC Project/OpenSC/pkcs11/opensc-pkcs11.dll";

impl HSMInfo {
    pub fn new() -> Self {
        HSMInfo {
            libpath: std::path::PathBuf::from(
                std::env::var("NITROHSM_LIBPATH").unwrap_or_else(|_| PKCS11_LIBPATH.to_string()),
            ),
            slot: std::env::var("NITROHSM_SLOT").map_or(0, |s| s.parse().unwrap()),
            ident: std::env::var("NITROHSM_ID").unwrap_or_else(|_| "01".to_string()),
            pin: std::cell::RefCell::new(None),
        }
    }
}

#[derive(Debug)]
pub enum AuthInfo {
    NoAuth, // No authentication details were provided;
    // only unsigned queries are allowed.
    PemFile(String), // --private-pem file specified
    NitroHsm(HSMInfo),
}

pub fn ledger_canister_id() -> Principal {
    Principal::from_slice(LEDGER_CANISTER_ID.as_ref())
}

pub fn governance_canister_id() -> Principal {
    Principal::from_slice(GOVERNANCE_CANISTER_ID.as_ref())
}

pub fn genesis_token_canister_id() -> Principal {
    Principal::from_slice(GENESIS_TOKEN_CANISTER_ID.as_ref())
}

// Returns the candid for the specified canister id, if there is one.
pub fn get_local_candid(canister_id: Principal) -> AnyhowResult<String> {
    if canister_id == governance_canister_id() {
        String::from_utf8(include_bytes!("../../candid/governance.did").to_vec())
            .map_err(|e| anyhow!(e))
    } else if canister_id == ledger_canister_id() {
        String::from_utf8(include_bytes!("../../candid/ledger.did").to_vec())
            .map_err(|e| anyhow!(e))
    } else if canister_id == genesis_token_canister_id() {
        String::from_utf8(include_bytes!("../../candid/gtc.did").to_vec()).map_err(|e| anyhow!(e))
    } else {
        unreachable!()
    }
}

/// Returns pretty-printed encoding of a candid value.
pub fn get_idl_string(
    blob: &[u8],
    canister_id: Principal,
    method_name: &str,
    part: &str,
) -> AnyhowResult<String> {
    let spec = get_local_candid(canister_id)?;
    let method_type = get_candid_type(spec, method_name);
    let result = match method_type {
        None => candid::IDLArgs::from_bytes(blob),
        Some((env, func)) => candid::IDLArgs::from_bytes_with_types(
            blob,
            &env,
            if part == "args" {
                &func.args
            } else {
                &func.rets
            },
        ),
    };
    Ok(format!("{}", result?))
}

/// Returns the candid type of a specifed method and correspondig idl
/// description.
pub fn get_candid_type(idl: String, method_name: &str) -> Option<(TypeEnv, Function)> {
    let ast = candid::pretty_parse::<IDLProg>("/dev/null", &idl).ok()?;
    let mut env = TypeEnv::new();
    let actor = check_prog(&mut env, &ast).ok()?;
    let method = env.get_method(&actor?, method_name).ok()?.clone();
    Some((env, method))
}

/// Reads from the file path or STDIN and returns the content.
pub fn read_from_file(path: &str) -> AnyhowResult<String> {
    use std::io::Read;
    let mut content = String::new();
    if path == "-" {
        std::io::stdin().read_to_string(&mut content)?;
    } else {
        let path = std::path::Path::new(&path);
        let mut file =
            std::fs::File::open(&path).map_err(|_| anyhow!("Message file doesn't exist"))?;
        file.read_to_string(&mut content)
            .map_err(|_| anyhow!("Cannot read the message file."))?;
    }
    Ok(content)
}

/// Returns an agent with an identity derived from a private key if it was
/// provided.
pub fn get_agent(auth: &AuthInfo) -> AnyhowResult<Agent> {
    let timeout = std::time::Duration::from_secs(60 * 5);
    let builder = Agent::builder()
        .with_transport(
            ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create({
                get_ic_url()
            })?,
        )
        .with_ingress_expiry(Some(timeout));

    builder
        .with_boxed_identity(get_identity(auth))
        .build()
        .map_err(|err| anyhow!(err))
}

/// Returns an identity derived from the private key.
pub fn get_identity(auth: &AuthInfo) -> Box<dyn Identity + Sync + Send> {
    match auth {
        AuthInfo::NoAuth => Box::new(AnonymousIdentity),
        AuthInfo::PemFile(pem) => match Secp256k1Identity::from_pem(pem.as_bytes()) {
            Ok(identity) => Box::new(identity),
            Err(_) => match BasicIdentity::from_pem(pem.as_bytes()) {
                Ok(identity) => Box::new(identity),
                Err(_) => match BasicIdentity::from_pem(pem.as_bytes()) {
                    Ok(identity) => Box::new(identity),
                    Err(_) => {
                        eprintln!("Couldn't load identity from PEM file");
                        std::process::exit(1);
                    }
                },
            },
        },
        AuthInfo::NitroHsm(info) => Box::new(
            hsm::HardwareIdentity::new(&info.libpath, info.slot, &info.ident, || {
                let pin = info.pin.borrow().clone();
                Ok(pin.unwrap_or_else(|| {
                    std::env::var("NITROHSM_PIN").unwrap_or_else(|_| {
                        let pin = rpassword::read_password_from_tty(Some("NitroHSM PIN: "))
                            .map_err(|_| {
                                eprintln!("NITROHSM_PIN not set");
                                std::process::exit(1);
                            })
                            .unwrap();
                        *info.pin.borrow_mut() = Some(pin.clone());
                        pin
                    })
                }))
            })
            .unwrap(),
        ),
    }
}

pub fn parse_query_response(
    response: Vec<u8>,
    canister_id: Principal,
    method_name: &str,
) -> AnyhowResult<String> {
    let cbor: Value = serde_cbor::from_slice(&response)
        .map_err(|_| anyhow!("Invalid cbor data in the content of the message."))?;
    if let Value::Map(m) = cbor {
        // Try to decode a rejected response.
        if let (_, Some(Value::Integer(reject_code)), Some(Value::Text(reject_message))) = (
            m.get(&Value::Text("status".to_string())),
            m.get(&Value::Text("reject_code".to_string())),
            m.get(&Value::Text("reject_message".to_string())),
        ) {
            return Ok(format!(
                "Rejected (code {}): {}",
                reject_code, reject_message
            ));
        }

        // Try to decode a successful response.
        if let (_, Some(Value::Map(m))) = (
            m.get(&Value::Text("status".to_string())),
            m.get(&Value::Text("reply".to_string())),
        ) {
            if let Some(Value::Bytes(reply)) = m.get(&Value::Text("arg".to_string())) {
                return get_idl_string(reply, canister_id, method_name, "rets");
            }
        }
    }
    Err(anyhow!("Invalid cbor content"))
}

pub fn get_account_id(principal_id: Principal) -> AnyhowResult<ledger_canister::AccountIdentifier> {
    use std::convert::TryFrom;
    let base_types_principal =
        PrincipalId::try_from(principal_id.as_slice()).map_err(|err| anyhow!(err))?;
    Ok(ledger_canister::AccountIdentifier::new(
        base_types_principal,
        None,
    ))
}

/// Converts menmonic to PEM format
pub fn mnemonic_to_pem(mnemonic: &Mnemonic) -> String {
    fn der_encode_secret_key(public_key: Vec<u8>, secret: Vec<u8>) -> Vec<u8> {
        let secp256k1_id = ObjectIdentifier(0, oid!(1, 3, 132, 0, 10));
        let data = Sequence(
            0,
            vec![
                Integer(0, BigInt::from(1)),
                OctetString(32, secret.to_vec()),
                Explicit(
                    ASN1Class::ContextSpecific,
                    0,
                    BigUint::from(0u32),
                    Box::new(secp256k1_id),
                ),
                Explicit(
                    ASN1Class::ContextSpecific,
                    0,
                    BigUint::from(1u32),
                    Box::new(BitString(0, public_key.len() * 8, public_key)),
                ),
            ],
        );
        to_der(&data).expect("Cannot encode secret key.")
    }

    let seed = mnemonic.to_seed("");
    let ext = tiny_hderive::bip32::ExtendedPrivKey::derive(&seed, "m/44'/223'/0'/0/0").unwrap();
    let secret = ext.secret();
    let secret_key = SecretKey::parse(&secret).unwrap();
    let public_key = PublicKey::from_secret_key(&secret_key);
    let der = der_encode_secret_key(public_key.serialize().to_vec(), secret.to_vec());
    let pem = Pem {
        tag: String::from("EC PRIVATE KEY"),
        contents: der,
    };
    encode(&pem).replace("\r", "").replace("\n\n", "\n")
}
