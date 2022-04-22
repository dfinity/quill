//! All the common functionality.
use crate::SnsCanisterIds;
use anyhow::{anyhow, Context};
use bip39::Mnemonic;
use candid::{
    parser::typing::{check_prog, TypeEnv},
    types::Function,
    CandidType, IDLProg,
};
use ic_agent::{
    identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity},
    Agent, Identity,
};
use ic_base_types::PrincipalId;
use ic_sns_governance::pb::v1::NeuronId;
use ic_types::Principal;
use libsecp256k1::{PublicKey, SecretKey};
use pem::{encode, Pem};
use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use simple_asn1::ASN1Block::{
    BitString, Explicit, Integer, ObjectIdentifier, OctetString, Sequence,
};
use simple_asn1::{oid, to_der, ASN1Class, BigInt, BigUint};

pub const IC_URL: &str = "https://ic0.app";

const EC_PARAMETERS: [u8; 7] = [6, 4, 43, 129, 4, 0, 10];

pub fn get_ic_url() -> String {
    std::env::var("IC_URL").unwrap_or_else(|_| IC_URL.to_string())
}

pub mod qr;
pub mod signing;

pub type AnyhowResult<T = ()> = anyhow::Result<T>;

#[derive(
    Serialize, Deserialize, CandidType, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum TargetCanister {
    Governance(Principal),
    Ledger(Principal),
}

impl From<TargetCanister> for Principal {
    fn from(target_canister: TargetCanister) -> Self {
        match target_canister {
            TargetCanister::Governance(principal) => principal,
            TargetCanister::Ledger(principal) => principal,
        }
    }
}

/// Returns the candid interface definition (i.e. the contents of a .did file)
/// for the target canister, if there is one.
pub fn get_local_candid(target_canister: TargetCanister) -> AnyhowResult<String> {
    match target_canister {
        TargetCanister::Governance(_) => {
            String::from_utf8(include_bytes!("../../candid/governance.did").to_vec())
                .context("Cannot load governance.did")
        }
        TargetCanister::Ledger(_) => {
            String::from_utf8(include_bytes!("../../candid/ledger.did").to_vec())
                .context("Cannot load ledger.did")
        }
    }
}

/// Returns pretty-printed encoding of a candid value.
pub fn get_idl_string(
    serialized_candid: &[u8],
    target_canister: TargetCanister,
    method_name: &str,
    part: &str,
) -> AnyhowResult<String> {
    let spec = get_local_candid(target_canister)?;
    let method_type = get_candid_type(spec, method_name);
    let result = match method_type {
        None => candid::IDLArgs::from_bytes(serialized_candid),
        Some((env, func)) => candid::IDLArgs::from_bytes_with_types(
            serialized_candid,
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

/// Returns the candid type of a specified method and corresponding idl description.
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
        let mut file = std::fs::File::open(&path).context("Cannot open the message file.")?;
        file.read_to_string(&mut content)
            .context("Cannot read the message file.")?;
    }
    Ok(content)
}

/// Returns an agent with an identity derived from a private key if it was provided.
pub fn get_agent(pem: &str) -> AnyhowResult<Agent> {
    let timeout = std::time::Duration::from_secs(60 * 5);
    let builder = Agent::builder()
        .with_transport(
            ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create({
                get_ic_url()
            })?,
        )
        .with_ingress_expiry(Some(timeout));

    builder
        .with_boxed_identity(get_identity(pem))
        .build()
        .map_err(|err| anyhow!(err))
}

/// Returns an identity derived from the private key.
pub fn get_identity(pem: &str) -> Box<dyn Identity + Sync + Send> {
    if pem.is_empty() {
        return Box::new(AnonymousIdentity);
    }
    match Secp256k1Identity::from_pem(pem.as_bytes()) {
        Ok(identity) => Box::new(identity),
        Err(_) => match BasicIdentity::from_pem(pem.as_bytes()) {
            Ok(identity) => Box::new(identity),
            Err(_) => {
                eprintln!("Couldn't load identity from PEM file");
                std::process::exit(1);
            }
        },
    }
}

pub fn require_pem(pem: &Option<String>) -> AnyhowResult<String> {
    match pem {
        None => Err(anyhow!(
            "Cannot use anonymous principal, did you forget --pem-file <pem-file> ?"
        )),
        Some(val) => Ok(val.clone()),
    }
}

pub fn require_canister_ids(
    sns_canister_ids: &Option<SnsCanisterIds>,
) -> AnyhowResult<SnsCanisterIds> {
    match sns_canister_ids {
        None => Err(anyhow!(
            "Cannot sign command without knowing the SNS canister ids, did you forget --canister-ids-file <json-file> ?"
        )),
        Some(ids) => Ok(ids.clone()),
    }
}

pub fn parse_query_response(response: Vec<u8>) -> AnyhowResult<Vec<u8>> {
    let cbor: Value = serde_cbor::from_slice(&response)
        .context("Invalid cbor data in the content of the message.")?;
    if let Value::Map(m) = cbor {
        // Try to decode a rejected response.
        if let (Some(Value::Integer(reject_code)), Some(Value::Text(reject_message))) = (
            m.get(&Value::Text("reject_code".to_string())),
            m.get(&Value::Text("reject_message".to_string())),
        ) {
            return Err(anyhow!(
                "Rejected (code {}): {}",
                reject_code,
                reject_message
            ));
        }

        // Try to decode a successful response.
        if let Some(Value::Map(m)) = m.get(&Value::Text("reply".to_string())) {
            if let Some(Value::Bytes(reply)) = m.get(&Value::Text("arg".to_string())) {
                return Ok(reply.clone());
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

/// Parse a NeuronId from a hex encoded string.
pub fn parse_neuron_id(hex_encoded_id: String) -> AnyhowResult<NeuronId> {
    let id = hex::decode(hex_encoded_id)?;
    Ok(NeuronId { id })
}

/// Converts mnemonic to PEM format
pub fn mnemonic_to_pem(mnemonic: &Mnemonic) -> AnyhowResult<String> {
    fn der_encode_secret_key(public_key: Vec<u8>, secret: Vec<u8>) -> AnyhowResult<Vec<u8>> {
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
        to_der(&data).context("Failed to encode secp256k1 secret key to DER")
    }

    let seed = mnemonic.to_seed("");
    let ext = tiny_hderive::bip32::ExtendedPrivKey::derive(&seed, "m/44'/223'/0'/0/0")
        .map_err(|err| anyhow!("{:?}", err))
        .context("Failed to derive BIP32 extended private key")?;
    let secret = ext.secret();
    let secret_key = SecretKey::parse(&secret).context("Failed to parse secret key")?;
    let public_key = PublicKey::from_secret_key(&secret_key);
    let der = der_encode_secret_key(public_key.serialize().to_vec(), secret.to_vec())?;
    let pem = Pem {
        tag: String::from("EC PARAMETERS"),
        contents: EC_PARAMETERS.to_vec(),
    };
    let parameters_pem = encode(&pem);
    let pem = Pem {
        tag: String::from("EC PRIVATE KEY"),
        contents: der,
    };
    let key_pem = encode(&pem);
    Ok((parameters_pem + &key_pem)
        .replace('\r', "")
        .replace("\n\n", "\n"))
}
