use crate::lib::{get_account_id, get_identity, ledger::LedgerIdentity, AnyhowResult, AuthInfo};
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use icp_ledger::AccountIdentifier;
use k256::{elliptic_curve::sec1::ToEncodedPoint, SecretKey};
use sha3::{Digest, Keccak256};

#[derive(Parser)]
/// Prints the principal id and the account id.
pub struct PublicOpts {
    /// Principal for which to get the account_id.
    #[clap(long)]
    principal_id: Option<String>,
    /// Additionally prints the legacy DFN address for Genesis claims.
    #[clap(long, conflicts_with = "principal-id")]
    genesis_dfn: bool,
    /// If authenticating with a Ledger device, display the public IDs on the device.
    #[clap(long, requires = "ledgerhq")]
    display_on_ledger: bool,
}

/// Prints the account and the principal ids.
pub fn exec(auth: &AuthInfo, opts: PublicOpts) -> AnyhowResult {
    let (principal_id, account_id) = get_public_ids(auth, &opts)?;
    println!("Principal id: {}", principal_id.to_text());
    println!("Account id: {}", account_id);
    if opts.genesis_dfn {
        let AuthInfo::PemFile(pem) = auth else {
            bail!("Must supply a pem or seed file for the DFN address");
        };
        println!("DFN address: {}", get_dfn(pem)?)
    }
    if opts.display_on_ledger {
        LedgerIdentity::new()?.display_pk()?;
    }
    Ok(())
}

/// Returns the account id and the principal id if the private key was provided.
fn get_public_ids(
    auth: &AuthInfo,
    opts: &PublicOpts,
) -> AnyhowResult<(Principal, AccountIdentifier)> {
    match &opts.principal_id {
        Some(principal_id) => {
            let principal_id = Principal::from_text(principal_id)?;
            Ok((principal_id, get_account_id(principal_id)?))
        }
        None => {
            if let AuthInfo::NoAuth = auth {
                Err(anyhow!(
                    "public-ids cannot be used without specifying a private key"
                ))
            } else {
                get_ids(auth)
            }
        }
    }
}

fn get_dfn(pem: &str) -> AnyhowResult<String> {
    let pk = SecretKey::from_sec1_pem(pem).context("DFN addresses need a secp256k1 key")?;
    let pubk = pk.public_key();
    let uncompressed = pubk.to_encoded_point(false);
    let hash = Keccak256::digest(&uncompressed.as_bytes()[1..]);
    Ok(hex::encode(&hash[12..]))
}

/// Returns the account id and the principal id if the private key was provided.
pub fn get_ids(auth: &AuthInfo) -> AnyhowResult<(Principal, AccountIdentifier)> {
    let principal_id = get_identity(auth)?.sender().map_err(|e| anyhow!(e))?;
    Ok((principal_id, get_account_id(principal_id)?))
}
