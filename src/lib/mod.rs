//! All the common functionality.

use anyhow::anyhow;
use candid::{
    parser::typing::{check_prog, TypeEnv},
    types::Function,
    IDLProg,
};
use ic_agent::{
    identity::{BasicIdentity, Secp256k1Identity},
    Agent, Identity,
};
use ic_nns_constants::{GOVERNANCE_CANISTER_ID, LEDGER_CANISTER_ID};
use ic_types::Principal;
use std::path::PathBuf;

pub const IC_URL: &str = "https://ic0.app";

pub mod hsm;
pub mod sign;

pub type AnyhowResult<T = ()> = anyhow::Result<T>;

pub struct HSMInfo {
    pub libpath: PathBuf,
    pub slot: usize,
    pub ident: String,
}

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

// Returns the candid for the specified canister id, if there is one.
pub fn get_local_candid(canister_id: Principal) -> AnyhowResult<String> {
    if canister_id == governance_canister_id() {
        String::from_utf8(include_bytes!("../../candid/governance.did").to_vec())
            .map_err(|e| anyhow!(e))
    } else if canister_id == ledger_canister_id() {
        String::from_utf8(include_bytes!("../../candid/ledger.did").to_vec())
            .map_err(|e| anyhow!(e))
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
            ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create(
                IC_URL.to_string(),
            )?,
        )
        .with_ingress_expiry(Some(timeout));

    match auth {
        AuthInfo::NoAuth => builder,
        _ => builder.with_boxed_identity(get_identity(auth)),
    }
    .build()
    .map_err(|err| anyhow!(err))
}

/// Returns an identity derived from the private key.
pub fn get_identity(auth: &AuthInfo) -> Box<dyn Identity + Sync + Send> {
    match auth {
        AuthInfo::PemFile(pem) => match Secp256k1Identity::from_pem(pem.as_bytes()) {
            Ok(identity) => Box::new(identity),
            Err(_) => match BasicIdentity::from_pem(pem.as_bytes()) {
                Ok(identity) => Box::new(identity),
                Err(_) => {
                    eprintln!("Couldn't load identity from PEM file");
                    std::process::exit(1);
                }
            },
        },
        AuthInfo::NitroHsm(info) => Box::new(
            hsm::HardwareIdentity::new(&info.libpath, info.slot, &info.ident, || {
                Ok(std::env::var("NITROHSM_PIN").unwrap_or_else(|_| {
                    rpassword::read_password_from_tty(Some("NitroHSM PIN: "))
                        .map_err(|_| {
                            eprintln!("NITROHSM_PIN not set");
                            std::process::exit(1);
                        })
                        .unwrap()
                }))
            })
            .unwrap(),
        ),
        AuthInfo::NoAuth => panic!("AuthInfo::NoAuth has no identity"),
    }
}
