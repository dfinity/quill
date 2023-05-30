//! All the common functionality.

use anyhow::{anyhow, bail, ensure, Context};
use bip39::{Mnemonic, Seed};
use candid::{
    parser::typing::{check_prog, TypeEnv},
    types::Function,
    IDLProg, Principal,
};
use data_encoding::BASE32_NOPAD;
use ic_agent::{
    identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity},
    Agent, Identity,
};
use ic_base_types::PrincipalId;
#[cfg(feature = "hsm")]
use ic_identity_hsm::HardwareIdentity;
use ic_nns_constants::{
    GENESIS_TOKEN_CANISTER_ID, GOVERNANCE_CANISTER_ID, LEDGER_CANISTER_ID, REGISTRY_CANISTER_ID,
    SNS_WASM_CANISTER_ID,
};
use icp_ledger::{AccountIdentifier, Subaccount};
use icrc_ledger_types::icrc1::account::Account;
use itertools::Itertools;
use k256::{elliptic_curve::sec1::ToEncodedPoint, SecretKey};
use pem::{encode, Pem};
use serde_cbor::Value;
use simple_asn1::ASN1Block::{
    BitString, Explicit, Integer, ObjectIdentifier, OctetString, Sequence,
};
use simple_asn1::{oid, to_der, ASN1Class, BigInt, BigUint};
use std::str::FromStr;
#[cfg(feature = "hsm")]
use std::{cell::RefCell, path::PathBuf};
use std::{
    env,
    fmt::{self, Display, Formatter},
    path::Path,
    time::Duration,
};

pub const IC_URL: &str = "https://ic0.app";

pub fn get_ic_url() -> String {
    env::var("IC_URL").unwrap_or_else(|_| IC_URL.to_string())
}

pub mod signing;

pub type AnyhowResult<T = ()> = anyhow::Result<T>;

#[cfg(feature = "hsm")]
#[derive(Debug)]
pub struct HSMInfo {
    pub libpath: PathBuf,
    pub slot: usize,
    pub ident: String,
    pin: RefCell<Option<String>>,
}

#[cfg(all(target_os = "macos", feature = "hsm"))]
const PKCS11_LIBPATH: &str = "/Library/OpenSC/lib/pkcs11/opensc-pkcs11.so";
#[cfg(all(target_os = "linux", feature = "hsm"))]
const PKCS11_LIBPATH: &str = "/usr/lib/x86_64-linux-gnu/opensc-pkcs11.so";
#[cfg(all(target_os = "windows", feature = "hsm"))]
const PKCS11_LIBPATH: &str = r"C:\Program Files\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll";

#[cfg(feature = "hsm")]
impl HSMInfo {
    pub fn new() -> Self {
        HSMInfo {
            libpath: PathBuf::from(
                env::var("QUILL_HSM_LIBPATH")
                    .or_else(|_| env::var("NITROHSM_LIBPATH"))
                    .unwrap_or_else(|_| PKCS11_LIBPATH.to_string()),
            ),
            slot: env::var("QUILL_HSM_SLOT")
                .map(|s| usize::from_str_radix(&s, 16).unwrap())
                .or_else(|_| env::var("NITROHSM_SLOT").map(|s| s.parse().unwrap()))
                .unwrap_or(0),
            ident: env::var("QUILL_HSM_ID")
                .or_else(|_| env::var("NITROHSM_ID"))
                .unwrap_or_else(|_| "01".to_string()),
            pin: RefCell::new(None),
        }
    }
}

#[derive(Debug)]
pub enum AuthInfo {
    NoAuth, // No authentication details were provided;
    // only unsigned queries are allowed.
    PemFile(String), // --private-pem file specified
    #[cfg(feature = "hsm")]
    Pkcs11Hsm(HSMInfo),
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

pub fn registry_canister_id() -> Principal {
    Principal::from_slice(REGISTRY_CANISTER_ID.as_ref())
}

pub fn sns_wasm_canister_id() -> Principal {
    Principal::from_slice(SNS_WASM_CANISTER_ID.as_ref())
}

pub fn ckbtc_canister_id(testnet: bool) -> Principal {
    if testnet {
        Principal::from_text("mc6ru-gyaaa-aaaar-qaaaq-cai").unwrap()
    } else {
        Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()
    }
}

pub fn ckbtc_minter_canister_id(testnet: bool) -> Principal {
    if testnet {
        Principal::from_text("ml52i-qqaaa-aaaar-qaaba-cai").unwrap()
    } else {
        Principal::from_text("mqygn-kiaaa-aaaar-qaadq-cai").unwrap()
    }
}

pub const ROLE_NNS_GOVERNANCE: &str = "nns:governance";
pub const ROLE_NNS_LEDGER: &str = "nns:ledger";
pub const ROLE_NNS_GTC: &str = "nns:gtc";
pub const ROLE_NNS_REGISTRY: &str = "nns:registry";
pub const ROLE_SNS_WASM: &str = "nns:sns-wasm";
pub const ROLE_ICRC1_LEDGER: &str = "icrc1:ledger";
pub const ROLE_CKBTC_MINTER: &str = "ckbtc:minter";
pub const ROLE_SNS_GOVERNANCE: &str = "sns:governance";
pub const ROLE_SNS_ROOT: &str = "sns:root";
pub const ROLE_SNS_SWAP: &str = "sns:swap";

pub fn get_default_role(canister_id: Principal) -> Option<&'static str> {
    if canister_id == governance_canister_id() {
        Some(ROLE_NNS_GOVERNANCE)
    } else if canister_id == ledger_canister_id() {
        Some(ROLE_NNS_LEDGER)
    } else if canister_id == genesis_token_canister_id() {
        Some(ROLE_NNS_GTC)
    } else if canister_id == registry_canister_id() {
        Some(ROLE_NNS_REGISTRY)
    } else if canister_id == ckbtc_canister_id(false) || canister_id == ckbtc_canister_id(true) {
        Some(ROLE_ICRC1_LEDGER)
    } else if canister_id == ckbtc_minter_canister_id(false)
        || canister_id == ckbtc_minter_canister_id(true)
    {
        Some(ROLE_CKBTC_MINTER)
    } else {
        None
    }
}

pub fn get_local_candid(canister_id: Principal, role: &str) -> AnyhowResult<&'static str> {
    Ok(match role {
        ROLE_NNS_GOVERNANCE => include_str!("../../candid/governance.did"),
        ROLE_NNS_LEDGER => include_str!("../../candid/ledger.did"),
        ROLE_NNS_GTC => include_str!("../../candid/gtc.did"),
        ROLE_NNS_REGISTRY => include_str!("../../candid/registry.did"),
        ROLE_ICRC1_LEDGER => include_str!("../../candid/icrc1.did"),
        ROLE_CKBTC_MINTER => include_str!("../../candid/ckbtc_minter.did"),
        ROLE_SNS_WASM => include_str!("../../candid/snsw.did"),
        ROLE_SNS_GOVERNANCE => include_str!("../../candid/sns-governance.did"),
        ROLE_SNS_ROOT => include_str!("../../candid/sns-root.did"),
        ROLE_SNS_SWAP => include_str!("../../candid/sns-swap.did"),
        _ => bail!(
            "\
Unknown recipient '{role}' in message!
Recipient: {canister_id}
Should be one of: 
- NNS Ledger: {ledger}
- Governance: {governance}
- Genesis: {genesis}
- Registry: {registry}
- ckBTC minter: {ckbtc_minter}
- ckBTC ledger: {ckbtc}
- SNS-WASM: {sns_wasm}
- SNS Governance
- SNS Ledger
- SNS Root
- SNS Swap",
            ledger = ledger_canister_id(),
            governance = governance_canister_id(),
            genesis = genesis_token_canister_id(),
            registry = registry_canister_id(),
            ckbtc_minter = ckbtc_minter_canister_id(false),
            ckbtc = ckbtc_canister_id(false),
            sns_wasm = sns_wasm_canister_id(),
        ),
    })
}

/// Returns pretty-printed encoding of a candid value.
pub fn get_idl_string(
    blob: &[u8],
    canister_id: Principal,
    role: &str,
    method_name: &str,
    part: &str,
) -> AnyhowResult<String> {
    let spec = get_local_candid(canister_id, role)?;
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
pub fn get_candid_type(idl: &str, method_name: &str) -> Option<(TypeEnv, Function)> {
    let ast = candid::pretty_parse::<IDLProg>("/dev/null", idl).ok()?;
    let mut env = TypeEnv::new();
    let actor = check_prog(&mut env, &ast).ok()?;
    let method = env.get_method(&actor?, method_name).ok()?.clone();
    Some((env, method))
}

/// Reads from the file path or STDIN and returns the content.
pub fn read_from_file(path: impl AsRef<Path>) -> AnyhowResult<String> {
    use std::io::Read;
    let path = path.as_ref();
    let mut content = String::new();
    if path == Path::new("-") {
        std::io::stdin().read_to_string(&mut content)?;
    } else {
        let mut file = std::fs::File::open(path).context("Cannot open the message file.")?;
        file.read_to_string(&mut content)
            .context("Cannot read the message file.")?;
    }
    Ok(content)
}

/// Returns an agent with an identity derived from a private key if it was
/// provided.
pub fn get_agent(auth: &AuthInfo) -> AnyhowResult<Agent> {
    let timeout = Duration::from_secs(60 * 5);
    let builder = Agent::builder()
        .with_transport(
            ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create({
                get_ic_url()
            })?,
        )
        .with_ingress_expiry(Some(timeout));

    let identity = get_identity(auth)?;
    builder
        .with_boxed_identity(identity)
        .build()
        .map_err(|err| anyhow!(err))
}

#[cfg(feature = "hsm")]
fn ask_pkcs11_pin_via_tty() -> Result<String, String> {
    rpassword::prompt_password("HSM PIN: ")
        .context("Cannot read HSM PIN from tty")
        // TODO: better error string
        .map_err(|e| e.to_string())
}

#[cfg(feature = "hsm")]
fn read_pkcs11_pin_env_var() -> Result<Option<String>, String> {
    match env::var("QUILL_HSM_PIN").or_else(|_| env::var("NITROHSM_PIN")) {
        Ok(val) => Ok(Some(val)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(e) => Err(format!("{}", e)),
    }
}

/// Returns an identity derived from the private key.
pub fn get_identity(auth: &AuthInfo) -> AnyhowResult<Box<dyn Identity>> {
    match auth {
        AuthInfo::NoAuth => Ok(Box::new(AnonymousIdentity) as _),
        AuthInfo::PemFile(pem) => match Secp256k1Identity::from_pem(pem.as_bytes()) {
            Ok(id) => Ok(Box::new(id) as _),
            Err(_) => match BasicIdentity::from_pem(pem.as_bytes()) {
                Ok(id) => Ok(Box::new(id) as _),
                Err(e) => Err(e).context("couldn't load identity from PEM file"),
            },
        },
        #[cfg(feature = "hsm")]
        AuthInfo::Pkcs11Hsm(info) => {
            let pin_fn = || {
                let user_set_pin = { info.pin.borrow().clone() };
                match user_set_pin {
                    None => match read_pkcs11_pin_env_var() {
                        Ok(Some(pin)) => Ok(pin),
                        Ok(None) => {
                            let pin = ask_pkcs11_pin_via_tty()?;
                            *info.pin.borrow_mut() = Some(pin.clone());
                            Ok(pin)
                        }
                        Err(e) => Err(e),
                    },
                    Some(pin) => Ok(pin),
                }
            };
            let identity = HardwareIdentity::new(&info.libpath, info.slot, &info.ident, pin_fn)
                .context("Unable to use your hardware key")?;
            Ok(Box::new(identity) as _)
        }
    }
}

pub fn parse_query_response(
    response: Vec<u8>,
    canister_id: Principal,
    role: &str,
    method_name: &str,
) -> AnyhowResult<String> {
    let cbor: Value = serde_cbor::from_slice(&response)
        .context("Invalid cbor data in the content of the message.")?;
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
                return get_idl_string(reply, canister_id, role, method_name, "rets");
            }
        }
    }
    Err(anyhow!("Invalid cbor content"))
}

pub fn get_account_id(principal_id: Principal) -> AnyhowResult<AccountIdentifier> {
    let base_types_principal =
        PrincipalId::try_from(principal_id.as_slice()).map_err(|err| anyhow!(err))?;
    Ok(AccountIdentifier::new(base_types_principal, None))
}

/// Converts menmonic to PEM format
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

    let seed = Seed::new(mnemonic, "");
    let ext = bip32::XPrv::derive_from_path(seed, &"m/44'/223'/0'/0/0".parse()?)
        .map_err(|err| anyhow!("{:?}", err))
        .context("Failed to derive BIP32 extended private key")?;
    let secret = ext.private_key();
    let secret_key = SecretKey::from(secret);
    let public_key = secret_key.public_key();
    let der = der_encode_secret_key(
        public_key.to_encoded_point(false).to_bytes().into(),
        secret_key.to_be_bytes().to_vec(),
    )?;
    let pem = Pem {
        tag: String::from("EC PRIVATE KEY"),
        contents: der,
    };
    let key_pem = encode(&pem);
    Ok(key_pem.replace('\r', "").replace("\n\n", "\n"))
}

pub struct ParsedSubaccount(pub Subaccount);

impl FromStr for ParsedSubaccount {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut array = [0; 32];
        ensure!(
            s.len() <= 64,
            "Too long: subaccounts are 64 characters or less"
        );
        hex::decode_to_slice(s, &mut array[32 - s.len() / 2..])?;
        Ok(ParsedSubaccount(Subaccount(array)))
    }
}

pub struct ParsedAccount(pub Account);

impl FromStr for ParsedAccount {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut base32 = s.replace('-', "");
        base32.make_ascii_uppercase();
        let decoded = BASE32_NOPAD.decode(base32.as_bytes())?;
        let (crc_bytes, addr) = decoded.split_at(4);
        let crc = crc32fast::hash(addr);
        if crc.to_be_bytes() != *crc_bytes {
            bail!("Principal CRC doesn't match - was it copied correctly?");
        }
        if addr.last() != Some(&0x7f) {
            let principal = Principal::try_from_slice(addr)?;
            return Ok(Self(Account {
                owner: principal,
                subaccount: None,
            }));
        }
        let subaccount_length = *addr
            .get(addr.len() - 2)
            .context("Invalid ICRC-1 address (subaccount length missing)")?
            as usize;
        ensure!(
            subaccount_length <= 32,
            "Invalid ICRC-1 address (subaccount too long)"
        );
        let subaccount = addr
            .get(addr.len() - subaccount_length - 2..addr.len() - 2)
            .context("Invalid ICRC-1 address (subaccount too small)")?;
        let mut subaccount_padded = [0; 32];
        subaccount_padded[32 - subaccount_length..].copy_from_slice(subaccount);
        let principal = Principal::try_from_slice(&addr[..addr.len() - subaccount_length - 2])?;
        Ok(Self(Account {
            owner: principal,
            subaccount: Some(subaccount_padded),
        }))
    }
}

impl Display for ParsedAccount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt_account(&self.0, f)
    }
}

fn fmt_account(account: &Account, f: &mut Formatter<'_>) -> fmt::Result {
    const EMPTY: [u8; 32] = [0; 32];
    match account.subaccount {
        None | Some(EMPTY) if account.owner.as_slice().last() != Some(&0x7f) => {
            account.owner.fmt(f)
        }
        _ => {
            let mut principal_bytes = account.owner.as_slice().to_owned();
            let subaccount = account.subaccount.unwrap_or_default();
            let first_digit = subaccount.iter().position(|x| *x != 0);
            let shrunk = if let Some(first_digit) = first_digit {
                &subaccount[first_digit..]
            } else {
                &[]
            };
            principal_bytes.extend_from_slice(shrunk);
            principal_bytes.extend_from_slice(&[shrunk.len() as u8, 0x7f]);
            let crc = crc32fast::hash(&principal_bytes);
            principal_bytes.splice(0..0, crc.to_be_bytes());
            let hex_encoding = BASE32_NOPAD.encode(&principal_bytes);
            let chunks = hex_encoding.chars().chunks(5);
            write!(
                f,
                "{}",
                chunks
                    .into_iter()
                    .map(|ck| ck.map(|ch| ch.to_ascii_lowercase()).format(""))
                    .format("-")
            )
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParsedNnsAccount {
    Original(AccountIdentifier),
    Icrc1(Account),
}

impl Display for ParsedNnsAccount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Original(ident) => ident.to_hex().fmt(f),
            Self::Icrc1(account) => fmt_account(account, f),
        }
    }
}

impl FromStr for ParsedNnsAccount {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.as_bytes()[6] == b'-' {
            Ok(Self::Icrc1(ParsedAccount::from_str(s)?.0))
        } else {
            let intended = AccountIdentifier::from_hex(s);
            match intended {
                Ok(o) => Ok(Self::Original(o)),
                Err(e) => Ok(Self::Icrc1(
                    ParsedAccount::from_str(s).map_err(|_| anyhow!(e))?.0,
                )),
            }
        }
    }
}

impl ParsedNnsAccount {
    pub fn into_identifier(self) -> AccountIdentifier {
        match self {
            Self::Original(ident) => ident,
            Self::Icrc1(account) => {
                AccountIdentifier::new(account.owner.into(), account.subaccount.map(Subaccount))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ParsedAccount, ParsedSubaccount};
    use candid::Principal;
    use std::str::FromStr;

    #[test]
    fn account() {
        let account = ParsedAccount::from_str("q26sl-4iaaa-aaaar-qaadq-cajkb-j33fm-ey45l-omb3j-kujum-vl6ge-wyjtd-vv37j-zkeli-5k5fb-h64qq-h6").unwrap();
        assert_eq!(
            account.0.owner,
            Principal::from_str("mqygn-kiaaa-aaaar-qaadq-cai").unwrap()
        );
        assert_eq!(account.0.subaccount, Some(*b"*\x0aw\xb2\xb0\x98\xe7V\xe6\x07iU\x13FU~1-\x84\xccu\xae\xfe\x9c\xa8\x8bGU\xd2\x84\xfe\xe4"));
        assert_eq!(account.to_string(), "q26sl-4iaaa-aaaar-qaadq-cajkb-j33fm-ey45l-omb3j-kujum-vl6ge-wyjtd-vv37j-zkeli-5k5fb-h64qq-h6")
    }

    #[test]
    fn simple_account() {
        let mut account = ParsedAccount::from_str("2vxsx-fae").unwrap();
        assert_eq!(account.0.owner, Principal::anonymous());
        assert_eq!(account.0.subaccount, None);
        assert_eq!(account.to_string(), "2vxsx-fae");
        let mut subacct1 = [0; 32];
        subacct1[31] = 1;
        account.0.subaccount = Some(subacct1);
        assert_eq!(account.to_string(), "ozcx7-eaeae-ax6");
        let account = ParsedAccount::from_str("ozcx7-eaeae-ax6").unwrap();
        assert_eq!(account.0.owner, Principal::anonymous());
        assert_eq!(account.0.subaccount, Some(subacct1));
    }

    #[test]
    fn subaccount() {
        let subacct = ParsedSubaccount::from_str(
            "2a0a77b2b098e756e60769551346557e312d84cc75aefe9ca88b4755d284fee4",
        )
        .unwrap();
        assert_eq!(subacct.0 .0, *b"\x2a\x0a\x77\xb2\xb0\x98\xe7\x56\xe6\x07\x69\x55\x13\x46\x55\x7e\x31\x2d\x84\xcc\x75\xae\xfe\x9c\xa8\x8b\x47\x55\xd2\x84\xfe\xe4");
        let short = ParsedSubaccount::from_str("0102").unwrap();
        assert_eq!(
            short.0 .0,
            *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\x02"
        );
    }
}
