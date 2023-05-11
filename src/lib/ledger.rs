use std::{
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::Context;
use bip32::DerivationPath;
use candid::Principal;
use hidapi::HidApi;
use ic_agent::{Identity, Signature};
use ledger_apdu::{APDUAnswer, APDUCommand, APDUErrorCode};
use ledger_transport_hid::TransportNativeHID;

use super::{derivation_path, AnyhowResult};

const CLA: u8 = 0x11;
const GET_VERSION: u8 = 0x00;
const GET_ADDR_SECP256K1: u8 = 0x01;
const SIGN_SECP256K1: u8 = 0x02;
const P1_ONLY_RETRIEVE: u8 = 0x00;
const P1_SHOW_ADDRESS_IN_DEVICE: u8 = 0x01;

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

pub struct LedgerIdentity {
    transport: TransportNativeHID,
    next_stake: AtomicBool,
}

impl LedgerIdentity {
    pub fn new() -> AnyhowResult<Self> {
        Ok(Self {
            transport: TransportNativeHID::new(&HidApi::new().unwrap())?,
            next_stake: AtomicBool::new(false),
        })
    }
    pub fn next_stake(&self) {
        self.next_stake.store(true, Ordering::Release);
    }
    pub fn version(&self) -> AnyhowResult<LedgerVersion> {
        get_version(&self.transport)
    }
    pub fn display_pk(&self) -> AnyhowResult<()> {
        display_pk(&self.transport, &derivation_path())
    }
}

impl Identity for LedgerIdentity {
    fn sender(&self) -> Result<Principal, String> {
        let (principal, _) = get_identity(&self.transport, &derivation_path())?;
        Ok(principal)
    }
    #[allow(clippy::bool_to_int_with_if)]
    fn sign(&self, blob: &[u8]) -> Result<Signature, String> {
        let path = derivation_path();
        let (_, pk) = get_identity(&self.transport, &path)?;
        let is_stake = self.next_stake.swap(false, Ordering::AcqRel);
        let sig = sign_blob(&self.transport, blob, if is_stake { 1 } else { 0 }, &path)?;
        Ok(Signature {
            public_key: Some(pk),
            signature: Some(sig),
        })
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
    let response = interpret_response(&response, "fetching principal from Ledger")?;
    let pk = response[PK_OFFSET..PK_OFFSET + PK_LEN].to_vec();
    let principal =
        Principal::try_from_slice(&response[PRINCIPAL_OFFSET..PRINCIPAL_OFFSET + PRINCIPAL_LEN])
            .map_err(|e| format!("Error interpreting principal from Ledger: {e}"))?;
    Ok((principal, pk))
}

fn interpret_response<'a>(
    response: &'a APDUAnswer<Vec<u8>>,
    action: &str,
) -> Result<&'a [u8], String> {
    if let Ok(errcode) = response.error_code() {
        if errcode == APDUErrorCode::NoError {
            Ok(response.apdu_data())
        } else {
            Err(format!("Error {action}: {errcode:?}"))
        }
    } else {
        Err(format!("Error {action}: {:#X}", response.retcode()))
    }
}

fn sign_blob(
    transport: &TransportNativeHID,
    blob: &[u8],
    txtype: u8,
    path: &DerivationPath,
) -> Result<Vec<u8>, String> {
    sign_chunk(transport, PAYLOAD_INIT, &serialize_path(path), txtype)?;
    let chunks = blob.chunks(CHUNK_SIZE);
    let len = chunks.len();
    for (i, chunk) in chunks.enumerate() {
        let res = sign_chunk(
            transport,
            if i == len { PAYLOAD_ADD } else { PAYLOAD_LAST },
            chunk,
            txtype,
        )?;
        if i == len {
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
    let response = interpret_response(&response, "signing message with Ledger")?;
    if !response.is_empty() {
        Ok(Some(response[SIG_OFFSET..SIG_OFFSET + SIG_LEN].to_vec()))
    } else {
        Ok(None)
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
    let response = interpret_response(&response, "fetching version from Ledger")
        .map_err(anyhow::Error::msg)?;
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
    interpret_response(&response, "displaying public key on Ledger").map_err(anyhow::Error::msg)?;
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
