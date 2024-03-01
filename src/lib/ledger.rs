use std::{
    cell::Cell,
    fmt,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

use anyhow::{ensure, Context};
use bip32::DerivationPath;
use candid::Principal;
use hidapi::HidApi;
use ic_agent::{agent::EnvelopeContent, Identity, Signature};
use indicatif::ProgressBar;
use ledger_apdu::{APDUAnswer, APDUCommand, APDUErrorCode};
use ledger_transport_hid::TransportNativeHID;
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_cbor::Serializer;

use super::{
    derivation_path, genesis_token_canister_id, governance_canister_id, ledger_canister_id,
    AnyhowResult,
};

const CLA: u8 = 0x11;
const GET_VERSION: u8 = 0x00;
const GET_ADDR_SECP256K1: u8 = 0x01;
const SIGN_SECP256K1: u8 = 0x02;
const P1_ONLY_RETRIEVE: u8 = 0x00;
const P1_SHOW_ADDRESS_IN_DEVICE: u8 = 0x01;
const TX_NORMAL: u8 = 0x00;
const TX_STAKING: u8 = 0x01;

const PAYLOAD_INIT: u8 = 0x00;
const PAYLOAD_ADD: u8 = 0x01;
const PAYLOAD_LAST: u8 = 0x02;

const PK_OFFSET: usize = 0;
const PK_LEN: usize = 65;
const PRINCIPAL_OFFSET: usize = PK_LEN;
const PRINCIPAL_LEN: usize = 29;

const SIG_OFFSET: usize = 43;
const SIG_LEN: usize = 64;

const CHUNK_SIZE: usize = 250;

// necessary due to HidApi being a singleton
static GLOBAL_HANDLE: Lazy<Mutex<Weak<LedgerIdentityInner>>> =
    Lazy::new(|| Mutex::new(Weak::new()));

// necessary due to Identity::sign not providing other ways to figure this out
thread_local! {
    static NEXT_STAKE: Cell<bool> = Cell::new(false);
}

struct LedgerIdentityInner {
    transport: Mutex<TransportNativeHID>,
}

/// An [`Identity`] backed by a Ledger device.
pub struct LedgerIdentity {
    inner: Arc<LedgerIdentityInner>,
}

impl LedgerIdentity {
    /// Creates a new ledger-device-backed identity.
    pub fn new() -> AnyhowResult<Self> {
        let mut global = GLOBAL_HANDLE.lock().unwrap();
        if let Some(existing) = global.upgrade() {
            Ok(Self { inner: existing })
        } else {
            let inner = Arc::new(LedgerIdentityInner {
                transport: Mutex::new(TransportNativeHID::new(&HidApi::new().unwrap())?),
            });
            *global = Arc::downgrade(&inner);
            Ok(Self { inner })
        }
    }
    /// Within the provided scope, transfers will be marked as 'staking' transactions.
    /// The IC app will refuse transfers to the governance canister unless this is used.
    pub fn with_staking<T>(f: impl FnOnce() -> T) -> T {
        // This is designed the way it is because Ledger signing has two modes and Identity::sign has one.
        // Short of re-parsing the transaction to figure out if it's transferring to the governance canister,
        // one must use a way of communicating this magic staking flag other than an Identity::sign parameter.
        // Global contextual state was judged the simplest way. This value is thread-local and only accessible in a closure,
        // so it should affect neither other threads nor other async tasks.
        // A scope-guard is used to reset it as destructors are still run during panics.
        NEXT_STAKE.with(|next_stake| {
            let _guard = scopeguard::guard((), |_| next_stake.set(false));
            next_stake.set(true);
            f()
        })
    }
    /// Gets the version of the IC app.
    #[allow(unused)]
    pub fn version(&self) -> AnyhowResult<LedgerVersion> {
        get_version(&self.inner.transport.lock().unwrap())
    }
    /// Displays the principal and legacy account ID on the Ledger, and asks the user to confirm it.
    pub fn display_pk(&self) -> AnyhowResult<()> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Confirm principal on Ledger device...");
        spinner.enable_steady_tick(Duration::from_millis(100));
        display_pk(&self.inner.transport.lock().unwrap(), &derivation_path())?;
        spinner.finish_and_clear();
        Ok(())
    }
    /// Gets the public key from the ledger that [`sender`](Self::sender) will return a principal derived from.
    pub fn public_key(&self) -> AnyhowResult<(Principal, Vec<u8>)> {
        get_identity(&self.inner.transport.lock().unwrap(), &derivation_path())
            .map_err(anyhow::Error::msg)
    }
}

impl Identity for LedgerIdentity {
    fn sender(&self) -> Result<Principal, String> {
        let (principal, _) =
            get_identity(&self.inner.transport.lock().unwrap(), &derivation_path())?;
        Ok(principal)
    }
    /// Sign a request ID from a content map.
    ///
    /// The behavior of this function is affected by whether it is in a [`with_staking`](Self::with_staking) scope or not.
    #[allow(clippy::bool_to_int_with_if)]
    fn sign(&self, content: &EnvelopeContent) -> Result<Signature, String> {
        let path = derivation_path();
        let next_stake = NEXT_STAKE.with(|next_stake| next_stake.replace(false));
        let transport = self.inner.transport.lock().unwrap();
        let (_, pk) = get_identity(&transport, &path)?;
        // The IC ledger app expects to receive the entire envelope, sans signature.
        #[derive(Serialize)]
        struct Envelope<'a> {
            content: &'a EnvelopeContent,
        }
        let mut blob = vec![];
        let mut serializer = Serializer::new(&mut blob);
        serializer.self_describe().map_err(|e| format!("{e}"))?;
        Envelope { content }
            .serialize(&mut serializer)
            .map_err(|e| format!("{e}"))?;
        let spinner = ProgressBar::new_spinner();
        let message = match content {
            EnvelopeContent::Call { method_name, .. } => format!("`{method_name}` call"),
            EnvelopeContent::Query { method_name, .. } => format!("`{method_name}` query call"),
            EnvelopeContent::ReadState { .. } => "status check".into(),
        };
        spinner.set_message(format!("Confirm {message} on Ledger device..."));
        spinner.enable_steady_tick(Duration::from_millis(100));
        let sig = sign_blob(
            &transport,
            &blob,
            // See with_staking
            if next_stake { TX_STAKING } else { TX_NORMAL },
            &path,
            content,
        )?;
        spinner.finish_with_message(format!("Confirmed {message} on Ledger device"));
        Ok(Signature {
            public_key: Some(pk),
            signature: Some(sig),
            delegations: None,
        })
    }

    fn public_key(&self) -> Option<Vec<u8>> {
        let (_, pk) = get_identity(&self.inner.transport.lock().unwrap(), &derivation_path())
            .map_err(|e| e.to_string())
            .ok()?;
        Some(pk)
    }
}

fn serialize_path(path: &DerivationPath) -> Vec<u8> {
    path.as_ref()
        .iter()
        .flat_map(|x| x.0.to_le_bytes())
        .collect()
}

fn get_identity(
    transport: &TransportNativeHID,
    path: &DerivationPath,
) -> Result<(Principal, Vec<u8>), String> {
    let command = APDUCommand {
        cla: CLA,
        ins: GET_ADDR_SECP256K1,
        p1: P1_ONLY_RETRIEVE,
        p2: 0,
        data: serialize_path(path),
    };
    let response = transport
        .exchange(&command)
        .map_err(|e| format!("Error communicating with Ledger: {e}"))?;
    let response = interpret_response(&response, "fetching principal from Ledger", None)?;
    let pk = response
        .get(PK_OFFSET..PK_OFFSET + PK_LEN)
        .ok_or_else(|| "Ledger message too short".to_string())?
        .to_vec();
    let principal = Principal::try_from_slice(
        response
            .get(PRINCIPAL_OFFSET..PRINCIPAL_OFFSET + PRINCIPAL_LEN)
            .ok_or_else(|| "Ledger message too short".to_string())?,
    )
    .map_err(|e| format!("Error interpreting principal from Ledger: {e}"))?;
    Ok((principal, pk))
}

fn interpret_response<'a>(
    response: &'a APDUAnswer<Vec<u8>>,
    action: &str,
    content: Option<&EnvelopeContent>,
) -> Result<&'a [u8], String> {
    if let Ok(errcode) = response.error_code() {
        match errcode {
            APDUErrorCode::NoError => Ok(response.apdu_data()),
            APDUErrorCode::DataInvalid if matches!(content, Some(EnvelopeContent::Call { method_name, .. }) if method_name == "send_dfx") => {
                Err(format!("Error {action}: Must use a principal or ICRC-1 account ID, not a legacy account ID"))
            }
            APDUErrorCode::DataInvalid if matches!(content,
                Some(EnvelopeContent::Call { method_name, canister_id, .. }
                    | EnvelopeContent::Query { method_name, canister_id, .. })
                        if !supported_transaction(canister_id, method_name)
            ) => {
                Err(format!(
                    "Error {action}: The IC app for Ledger only supports transfers and certain neuron management operations"
                ))
            }
            APDUErrorCode::DataInvalid if matches!(content, Some(EnvelopeContent::Call { method_name, .. }) if method_name == "icrc1_transfer") => {
                Err(format!(
                    "Error {action}: The IC app for Ledger only supports transfers to user principals, not canisters or the anonymous principal"
                ))
            }
            APDUErrorCode::ClaNotSupported => Err(format!("Error {action}: IC app not open on device")),
            APDUErrorCode::CommandNotAllowed => Err(format!("Error {action}: Device rejected the message")),
            errcode => match std::str::from_utf8(response.apdu_data()) {
                Ok(s) if !s.trim().is_empty() => Err(format!("Error {action}: {errcode:?}: {s}")),
                _ => Err(format!("Error {action}: {errcode:?}")),
            },
        }
    } else {
        match response.retcode() {
            0x6E01 => Err(format!("Error {action}: IC app not open on device")),
            0x5515 => Err(format!("Error {action}: device is sleeping")),
            retcode => Err(format!("Error {action}: {retcode:#X}")),
        }
    }
}

pub fn supported_transaction(canister_id: &Principal, method_name: &str) -> bool {
    if *canister_id == genesis_token_canister_id() {
        method_name == "claim_neurons"
    } else if *canister_id == governance_canister_id() {
        method_name == "manage_neuron"
            || method_name == "manage_neuron_pb"
            || method_name == "list_neurons"
            || method_name == "list_neurons_pb"
            || method_name == "update_node_provider"
    } else if *canister_id == ledger_canister_id() {
        method_name == "send_pb" || method_name == "icrc1_transfer"
    } else {
        method_name == "icrc1_transfer"
            || method_name == "manage_neuron"
            || method_name == "list_neurons"
    }
}

fn sign_blob(
    transport: &TransportNativeHID,
    blob: &[u8],
    txtype: u8,
    path: &DerivationPath,
    content: &EnvelopeContent,
) -> Result<Vec<u8>, String> {
    sign_chunk(
        transport,
        PAYLOAD_INIT,
        &serialize_path(path),
        txtype,
        content,
    )?;
    let chunks = blob.chunks(CHUNK_SIZE);
    let end = chunks.len() - 1;
    for (i, chunk) in chunks.enumerate() {
        let res = sign_chunk(
            transport,
            if i == end { PAYLOAD_LAST } else { PAYLOAD_ADD },
            chunk,
            txtype,
            content,
        )?;
        if i == end {
            return Ok(res.ok_or("Error signing message with Ledger: No signature returned")?);
        }
    }
    unreachable!()
}

fn sign_chunk(
    transport: &TransportNativeHID,
    kind: u8,
    chunk: &[u8],
    txtype: u8,
    content: &EnvelopeContent,
) -> Result<Option<Vec<u8>>, String> {
    let command = APDUCommand {
        cla: CLA,
        ins: SIGN_SECP256K1,
        p1: kind,
        p2: txtype,
        data: chunk,
    };
    let response = transport
        .exchange(&command)
        .map_err(|e| format!("Error communicating with Ledger: {e}"))?;
    let response = interpret_response(&response, "signing message with Ledger", Some(content))?;
    if response.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            response
                .get(SIG_OFFSET..SIG_OFFSET + SIG_LEN)
                .ok_or_else(|| "Ledger message too short".to_string())?
                .to_vec(),
        ))
    }
}

fn get_version(transport: &TransportNativeHID) -> AnyhowResult<LedgerVersion> {
    let command = APDUCommand {
        cla: CLA,
        ins: GET_VERSION,
        p1: 0,
        p2: 0,
        data: &[][..],
    };
    let response = transport
        .exchange(&command)
        .context("Error communicating with ledger")?;
    let response = interpret_response(&response, "fetching version from Ledger", None)
        .map_err(anyhow::Error::msg)?;
    ensure!(response.len() >= 4, "Ledger message too short");
    Ok(LedgerVersion {
        major: response[1],
        minor: response[2],
        patch: response[3],
    })
}

fn display_pk(transport: &TransportNativeHID, path: &DerivationPath) -> AnyhowResult<()> {
    let command = APDUCommand {
        cla: CLA,
        ins: GET_ADDR_SECP256K1,
        p1: P1_SHOW_ADDRESS_IN_DEVICE,
        p2: 0,
        data: serialize_path(path),
    };
    let response = transport
        .exchange(&command)
        .context("Error communicating with ledger")?;
    interpret_response(&response, "displaying public key on Ledger", None)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

#[derive(Debug)]
pub struct LedgerVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl fmt::Display for LedgerVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
