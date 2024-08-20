//! All the common functionality.

use anyhow::{anyhow, bail, ensure, Context};
use bigdecimal::BigDecimal;
use bip32::DerivationPath;
use bip39::{Mnemonic, Seed};
use candid::{types::Function, Nat, Principal, TypeEnv};
use candid_parser::{typing::check_prog, utils::CandidSource, IDLProg};
use crc32fast::Hasher;
use data_encoding::BASE32_NOPAD;
use ic_agent::{
    identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity},
    Agent, Identity,
};
use ic_base_types::{CanisterId, PrincipalId};
#[cfg(feature = "hsm")]
use ic_identity_hsm::HardwareIdentity;
use ic_nns_constants::{
    GENESIS_TOKEN_CANISTER_ID, GOVERNANCE_CANISTER_ID, LEDGER_CANISTER_ID, REGISTRY_CANISTER_ID,
    SNS_WASM_CANISTER_ID,
    canister_id_to_nns_canister_name,
};
use icp_ledger::{AccountIdentifier, Subaccount};
use icrc_ledger_types::icrc1::account::Account;
use k256::SecretKey;
use pkcs8::pkcs5::{pbes2::Parameters, scrypt::Params};
use ring::signature::Ed25519KeyPair;
use serde_cbor::Value;

use std::{
    env,
    fmt::{self, Display, Formatter},
    path::Path,
    rc::Rc,
    str::FromStr,
    time::{Duration, SystemTime},
};

#[cfg(feature = "hsm")]
use std::{cell::RefCell, path::PathBuf};

#[cfg(feature = "ledger")]
use self::ledger::LedgerIdentity;

pub const IC_URL: &str = "https://ic0.app";

pub fn get_ic_url() -> String {
    env::var("IC_URL").unwrap_or_else(|_| IC_URL.to_string())
}

pub mod format;
#[cfg(feature = "ledger")]
pub mod ledger;
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
    pub fn new() -> AnyhowResult<Self> {
        let libpath_var = env::var("QUILL_HSM_LIBPATH").or_else(|_| env::var("NITROHSM_LIBPATH"));
        let libpath = libpath_var.unwrap_or_else(|_| PKCS11_LIBPATH.to_string());
        let slot = if let Ok(hex_slot) = env::var("QUILL_HSM_SLOT") {
            Some(usize::from_str_radix(&hex_slot, 16)?)
        } else {
            env::var("NITROHSM_SLOT")
                .ok()
                .map(|s| s.parse())
                .transpose()?
        }
        .unwrap_or(0);
        let hsm_id_var = env::var("QUILL_HSM_ID").or_else(|_| env::var("NITROHSM_ID"));
        let hsm_id = hsm_id_var.unwrap_or_else(|_| "01".to_string());
        Ok(HSMInfo {
            libpath: libpath.into(),
            slot,
            ident: hsm_id,
            pin: RefCell::new(None),
        })
    }
}

#[derive(Debug)]
pub enum AuthInfo {
    /// No authentication details were provided;
    /// only unsigned queries are allowed.
    NoAuth,
    /// Secp256k1 key provided via --pem-file.
    K256Key(SecretKey),
    /// Ed25519 key provided via --pem-file. Stored in PKCS#8 because Ed25519KeyPair cannot be cloned or unpacked.
    Ed25519Key(Vec<u8>),
    /// PKCS#11 security module, usually Nitrokey
    #[cfg(feature = "hsm")]
    Pkcs11Hsm(HSMInfo),
    /// Ledger Nano with the Internet Computer app installed
    #[cfg(feature = "ledger")]
    Ledger,
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

/// Returns a string representation of init_args.
///
/// Similar to get_idl_string, but that assumes you have a blob that gets passed
/// to, or returned from a method. Whereas, this deals with data that you pass
/// during installation (or upgrade).
///
/// Ideally, the string is human-readable. Otherwise, this falls back to
/// encoding init_args as in hex.
///
/// This only works well in for some NNS canisters, specifically,
///
///    - governance
///    - ledger
///    - gtc
///    - registry
///    - sns-wasm
fn display_init_args(init_args: &[u8], canister_id: PrincipalId) -> String {
    let canister_name = canister_id_to_nns_canister_name(
        CanisterId::unchecked_from_principal(canister_id)
    );

    let main = || {
        let canister_role = get_default_role(canister_id.0)
            .with_context(|| {
                format!(
                    "unable to humanize install args, because the role of {} is unknown.",
                    canister_id,
                )
            })?;

        // Glean supporting information about how to (decode and) interpret args
        // from (embedded) .did file.
        let interface = get_local_candid(canister_id.0, canister_role)
            .with_context(|| {
                format!(
                    "unable to display install args, because we do not have \
                     the interface definition of the {} canister",
                    canister_name,
                )
            })?;

        let (name_to_type, service) = CandidSource::Text(interface)
            .load()
            .with_context(|| {
                format!(
                    "unable to display install args, because we could not \
                     parse the interface definition of the {} canister.",
                    canister_name,
                )
            })?;

        let service = service.with_context(|| {
            format!(
                "unable to display install args, because there seems to \
                 be no service in the interface definition of the {} canister.",
                canister_name,
            )
        })?;
        let service = unwrap_type(service)
            .context("unable to display install args")?;

        let init_args_type = match service {
            candid::types::TypeInner::Class(init_args_type, _methods) => init_args_type,
            not_class => bail!("Somehow, service is not a service??? {:?}", not_class),
        };

        let init_args_type = init_args_type
            .into_iter()
            .map(|arg_type| {
                let arg_type = unwrap_type(arg_type)
                    .context("unable to display install args")?;
                match arg_type {
                    candid::types::TypeInner::Var(name) => {
                        name_to_type.find_type(&name)
                            .context("DO NOT MERGE")
                            .cloned()
                    }
                    arg_type => Ok(candid::types::Type::from(arg_type)),
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .context("DO NOT MERGE")?;

        // Finally, decode and interpret init args.
        candid::IDLArgs::from_bytes_with_types(
            init_args,
            &name_to_type,
            &init_args_type,
        )
        .context("DO NOT MERGE")
    };

    match main() {
        Ok(ok) => format!("{}", ok),
        Err(err) => { // DO NOT MERGE: Log err
            hex::encode(init_args)
        }
    }
}

fn unwrap_type(type_: candid::types::Type) -> anyhow::Result<candid::types::TypeInner> {
    let type_: Rc<_> = match type_ {
        candid::types::Type(ok) => ok,
    };

    Rc::into_inner(type_)
        .context("unable to unwrap type")
}

/// Returns pretty-printed encoding of a candid value.
pub fn display_response(
    blob: &[u8],
    canister_id: Principal,
    role: &str,
    method_name: &str,
    part: &str,
) -> AnyhowResult<String> {
    match role {
        ROLE_NNS_GOVERNANCE => match method_name {
            "get_neuron_info" => format::nns_governance::display_get_neuron_info(blob),
            "manage_neuron" => format::nns_governance::display_manage_neuron(blob),
            "get_neuron_ids" => format::nns_governance::display_neuron_ids(blob),
            "update_node_provider" => format::nns_governance::display_update_node_provider(blob),
            "list_proposals" => format::nns_governance::display_list_proposals(blob),
            "list_neurons" => format::nns_governance::display_list_neurons(blob),
            "get_proposal_info" => format::nns_governance::display_get_proposal(blob),
            "claim_gtc_neurons" => format::nns_governance::display_claim_gtc_neurons(blob),
            "claim_or_refresh_neuron_from_account" => {
                format::nns_governance::display_claim_or_refresh_neuron_from_account(blob)
            }
            _ => get_idl_string(blob, canister_id, role, method_name, part),
        },
        ROLE_SNS_GOVERNANCE => match method_name {
            "manage_neuron" => format::sns_governance::display_manage_neuron(blob),
            _ => get_idl_string(blob, canister_id, role, method_name, part),
        },
        ROLE_NNS_LEDGER => match method_name {
            "transfer" => format::icp_ledger::display_transfer(blob),
            "send_dfx" => format::icp_ledger::display_send_dfx(blob),
            "account_balance" | "account_balance_dfx" => {
                format::icp_ledger::display_account_balance_or_dfx(blob)
            }
            _ => get_idl_string(blob, canister_id, role, method_name, part),
        },
        ROLE_ICRC1_LEDGER => match method_name {
            "icrc1_transfer" => format::icrc1::display_transfer(blob),
            "icrc1_balance_of" => format::icrc1::display_balance(blob),
            _ => get_idl_string(blob, canister_id, role, method_name, part),
        },
        ROLE_SNS_SWAP => match method_name {
            "get_buyer_state" => format::sns_swap::display_get_buyer_state(blob),
            "error_refund_icp" => format::sns_swap::display_refund(blob),
            "new_sale_ticket" => format::sns_swap::display_new_sale_ticket(blob),
            "refresh_buyer_tokens" => format::sns_swap::display_refresh_buyer_tokens(blob),
            _ => get_idl_string(blob, canister_id, role, method_name, part),
        },
        ROLE_CKBTC_MINTER => match method_name {
            "update_balance" => format::ckbtc::display_update_balance(blob),
            "retrieve_btc" => format::ckbtc::display_retrieve_btc(blob),
            "retrieve_btc_status" => format::ckbtc::display_retrieve_btc_status(blob),
            "retrieve_btc_status_v2" => format::ckbtc::display_retrieve_btc_status_v2(blob),
            _ => get_idl_string(blob, canister_id, role, method_name, part),
        },
        ROLE_NNS_GTC => match method_name {
            "claim_neurons" => format::gtc::format_claim_neurons(blob),
            _ => get_idl_string(blob, canister_id, role, method_name, part),
        },
        ROLE_SNS_ROOT => match method_name {
            "get_sns_canisters_summary" => format::sns_root::display_canisters_summary(blob),
            _ => get_idl_string(blob, canister_id, role, method_name, part),
        },
        ROLE_NNS_REGISTRY => match method_name {
            "update_node_operator_config_directly" => {
                format::registry::display_update_node_operator_config_directly(blob)
            }
            _ => get_idl_string(blob, canister_id, role, method_name, part),
        },
        ROLE_SNS_WASM => match method_name {
            "list_deployed_snses" => format::sns_wasm::display_list_snses(blob),
            _ => get_idl_string(blob, canister_id, role, method_name, part),
        },
        _ => get_idl_string(blob, canister_id, role, method_name, part),
    }
}

/// Returns the candid type of a specifed method and correspondig idl
/// description.
pub fn get_candid_type(idl: &str, method_name: &str) -> Option<(TypeEnv, Function)> {
    let ast = candid_parser::pretty_parse::<IDLProg>("/dev/null", idl).ok()?;
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
        .with_url(get_ic_url())
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
        Err(e) => Err(format!("{e}")),
    }
}

/// Returns an identity derived from the private key.
pub fn get_identity(auth: &AuthInfo) -> AnyhowResult<Box<dyn Identity>> {
    match auth {
        AuthInfo::NoAuth => Ok(Box::new(AnonymousIdentity) as _),
        AuthInfo::K256Key(pk) => Ok(Box::new(Secp256k1Identity::from_private_key(pk.clone()))),
        AuthInfo::Ed25519Key(kp) => Ok(Box::new(BasicIdentity::from_key_pair(
            Ed25519KeyPair::from_pkcs8(kp).expect("Ed25519 key previously validated"),
        ))),
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
        #[cfg(feature = "ledger")]
        AuthInfo::Ledger => Ok(Box::new(LedgerIdentity::new()?)),
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
            return Ok(format!("Rejected (code {reject_code}): {reject_message}",));
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

/// Returns the account id and the principal id if the private key was provided.
pub fn get_principal(auth: &AuthInfo) -> AnyhowResult<Principal> {
    let principal_id = get_identity(auth)?.sender().map_err(|e| anyhow!(e))?;
    Ok(principal_id)
}

pub fn get_account_id(
    principal_id: Principal,
    subaccount: Option<Subaccount>,
) -> AnyhowResult<AccountIdentifier> {
    let base_types_principal =
        PrincipalId::try_from(principal_id.as_slice()).map_err(|err| anyhow!(err))?;
    Ok(AccountIdentifier::new(base_types_principal, subaccount))
}

/// Converts menmonic to PEM format
pub fn mnemonic_to_key(mnemonic: &Mnemonic) -> AnyhowResult<SecretKey> {
    let seed = Seed::new(mnemonic, "");
    let ext = bip32::XPrv::derive_from_path(seed, &derivation_path())
        .map_err(|err| anyhow!("{:?}", err))
        .context("Failed to derive BIP32 extended private key")?;
    let secret = ext.private_key();
    let secret_key = SecretKey::from(secret);
    Ok(secret_key)
}

const DERIVATION_PATH: &str = "m/44'/223'/0'/0/0";
fn derivation_path() -> DerivationPath {
    DERIVATION_PATH.parse().unwrap()
}

#[derive(Copy, Clone)]
pub struct ParsedSubaccount(pub Subaccount);

impl FromStr for ParsedSubaccount {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut array = [0; 32];
        ensure!(
            s.len() <= 64,
            "Too long: subaccounts are 64 characters or less"
        );
        let mut padded;
        let mut s = s;
        if s.len() % 2 == 1 {
            padded = String::new();
            padded.push('0');
            padded.push_str(s);
            s = &padded;
        }
        hex::decode_to_slice(s, &mut array[32 - s.len() / 2..])?;
        Ok(ParsedSubaccount(Subaccount(array)))
    }
}

#[derive(Debug, Clone)]
pub struct ParsedAccount(pub Account);

impl FromStr for ParsedAccount {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((rest, subaccount)) = s.split_once('.') else {
            return Ok(Self(Account {
                owner: Principal::from_str(s)
                    .map_err(|e| anyhow!("Invalid ICRC-1 account: missing subaccount, or: {e}"))?,
                subaccount: None,
            }));
        };
        let (principal, crc) = rest
            .rsplit_once('-')
            .context("Invalid ICRC-1 address (no principal)")?;
        let crc = BASE32_NOPAD
            .decode(crc.to_ascii_uppercase().as_bytes())
            .context("Invalid ICRC-1 account: invalid CRC")?;
        let crc = u32::from_be_bytes(
            crc[..]
                .try_into()
                .context("Invalid ICRC-1 account: invalid CRC")?,
        );
        let principal =
            Principal::from_str(principal).context("Invalid ICRC-1 account: invalid principal")?;
        ensure!(
            !subaccount.starts_with('0'),
            "Invalid ICRC-1 account: subaccount started with 0",
        );
        ensure!(
            !subaccount.is_empty(),
            "Invalid ICRC-1 account: empty subaccount despite subaccount separator",
        );
        let subaccount = ParsedSubaccount::from_str(subaccount)
            .context("Invalid ICRC-1 account: invalid subaccount")?;
        let mut hasher = Hasher::new();
        hasher.update(principal.as_slice());
        hasher.update(&subaccount.0 .0);
        ensure!(
            hasher.finalize() == crc,
            "Invalid ICRC-1 account: account ID did not match checksum (was it copied wrong?)"
        );
        Ok(Self(Account {
            owner: principal,
            subaccount: Some(subaccount.0 .0),
        }))
    }
}

impl Display for ParsedAccount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt_account(&self.0, f)
    }
}

fn fmt_account(account: &Account, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", account.owner)?;
    let Some(subaccount) = account.subaccount else {
        return Ok(());
    };
    let Some(first_digit) = subaccount.iter().position(|x| *x != 0) else {
        return Ok(());
    };
    let mut crc = Hasher::new();
    crc.update(account.owner.as_slice());
    crc.update(&subaccount);
    let mut crc = BASE32_NOPAD.encode(&crc.finalize().to_be_bytes());
    crc.make_ascii_lowercase();
    let shrunk = &subaccount[first_digit..];
    let subaccount = hex::encode(shrunk);
    let subaccount = if subaccount.as_bytes()[0] == b'0' {
        &subaccount[1..]
    } else {
        &subaccount
    };
    write!(f, "-{crc}.{subaccount}")
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

pub fn now_nanos() -> u64 {
    if std::env::var("QUILL_TEST_FIXED_TIMESTAMP").is_ok() {
        1_669_073_904_187_044_208
    } else {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

pub fn e8s_to_tokens(e8s: Nat) -> BigDecimal {
    BigDecimal::new(e8s.0.into(), 8)
}

pub fn key_encryption_params<'a>(salt: &'a [u8; 16], iv: &'a [u8; 16]) -> Parameters<'a> {
    let scrypt_params = Params::new(17, 8, 1, 32).expect("valid scrypt Params consts");
    Parameters::scrypt_aes256cbc(scrypt_params, salt, iv)
        .expect("valid PKCS5 encryption parameters")
}

#[cfg(test)]
mod tests {
    use super::{ParsedAccount, ParsedSubaccount, display_init_args};
    use candid::{Encode, Principal};
    use ic_base_types::PrincipalId;
    use ic_nns_governance::pb::v1::Governance as GovernanceProto;
    use ic_nns_constants::GOVERNANCE_CANISTER_ID;
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    #[test]
    fn account() {
        let account = ParsedAccount::from_str("k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae-dfxgiyy.102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20").unwrap();
        assert_eq!(
            account.0.owner,
            Principal::from_str("k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae")
                .unwrap(),
        );
        assert_eq!(account.0.subaccount, Some(*b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f\x20"));
        assert_eq!(account.to_string(), "k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae-dfxgiyy.102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20");
    }

    #[test]
    fn account_short() {
        let account = ParsedAccount::from_str(
            "k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae-6cc627i.1",
        )
        .unwrap();
        assert_eq!(
            account.0.owner,
            Principal::from_text("k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae")
                .unwrap(),
        );
        assert_eq!(account.0.subaccount.unwrap()[..31], [0; 31][..]);
        assert_eq!(account.0.subaccount.unwrap()[31], 1);
        assert_eq!(
            account.to_string(),
            "k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae-6cc627i.1",
        );
    }

    #[test]
    fn account_default_subaccount() {
        let mut account = ParsedAccount::from_str(
            "iooej-vlrze-c5tme-tn7qt-vqe7z-7bsj5-ebxlc-hlzgs-lueo3-3yast-pae",
        )
        .unwrap();
        assert_eq!(
            account.0.owner,
            Principal::from_str("iooej-vlrze-c5tme-tn7qt-vqe7z-7bsj5-ebxlc-hlzgs-lueo3-3yast-pae")
                .unwrap()
        );
        assert_eq!(account.0.subaccount, None);
        assert_eq!(
            account.to_string(),
            "iooej-vlrze-c5tme-tn7qt-vqe7z-7bsj5-ebxlc-hlzgs-lueo3-3yast-pae"
        );
        account.0.subaccount = Some([0; 32]);
        assert_eq!(
            account.to_string(),
            "iooej-vlrze-c5tme-tn7qt-vqe7z-7bsj5-ebxlc-hlzgs-lueo3-3yast-pae"
        );
    }

    #[test]
    fn account_other_principals() {
        let mut account = ParsedAccount::from_str("2vxsx-fae").unwrap();
        assert_eq!(account.0.owner, Principal::anonymous());
        assert_eq!(account.0.subaccount, None);
        assert_eq!(account.to_string(), "2vxsx-fae");
        let mut subacct1 = [0; 32];
        subacct1[31] = 1;
        account.0.subaccount = Some(subacct1);
        assert_eq!(account.to_string(), "2vxsx-fae-22yutvy.1");
    }

    #[test]
    fn account_errors() {
        let not_canonical = ParsedAccount::from_str(
            "k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae-6cc627i.01",
        );
        assert!(not_canonical
            .unwrap_err()
            .to_string()
            .contains("subaccount started with 0"));
        let no_cksum = ParsedAccount::from_str(
            "k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae.1",
        );
        assert!(no_cksum.unwrap_err().to_string().contains("invalid CRC"));
        let bad_cksum = ParsedAccount::from_str(
            "k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae-6cc627j.1",
        );
        assert!(bad_cksum.unwrap_err().to_string().contains("invalid CRC"));
        let wrong_cksum = ParsedAccount::from_str(
            "k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae-7cc627i.1",
        );
        assert!(wrong_cksum
            .unwrap_err()
            .to_string()
            .contains("account ID did not match checksum"));
        let null_subaccount = ParsedAccount::from_str(
            "k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae-q6bn32y.",
        );
        assert!(null_subaccount
            .unwrap_err()
            .to_string()
            .contains("empty subaccount despite subaccount separator"));
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

    #[test]
    fn test_display_init_args() {
        // Step 1: Construct input.
        let governance_proto = GovernanceProto {
            wait_for_quiet_threshold_seconds: 123_456_789,
            ..Default::default()
        };
        let encoded = Encode!(&governance_proto).unwrap();

        // Step 2: Call code under test.
        let decoded = display_init_args(&encoded, PrincipalId::from(GOVERNANCE_CANISTER_ID));

        // Step 3: Inspect results.
        assert_eq!(
            decoded,
            // If you dig deep enough, you will see a line in this string that
            // looks like the field value we choose at the beginning of this
            // test. Other fields are "false-y" (according to their type).
            // Common "false-y" values are null, vec {}, and 0. (This assert
            // needs to be updated every time a field is added to
            // GovernanceProto.)
            "(
  record {
    default_followees = vec {};
    making_sns_proposal = null;
    most_recent_monthly_node_provider_rewards = null;
    maturity_modulation_last_updated_at_timestamp_seconds = null;
    wait_for_quiet_threshold_seconds = 123_456_789 : nat64;
    metrics = null;
    neuron_management_voting_period_seconds = null;
    node_providers = vec {};
    cached_daily_maturity_modulation_basis_points = null;
    economics = null;
    restore_aging_summary = null;
    spawning_neurons = null;
    latest_reward_event = null;
    to_claim_transfers = vec {};
    short_voting_period_seconds = 0 : nat64;
    topic_followee_index = vec {};
    migrations = null;
    proposals = vec {};
    xdr_conversion_rate = null;
    in_flight_commands = vec {};
    neurons = vec {};
    genesis_timestamp_seconds = 0 : nat64;
  },
)",
        );
    }
}
