#[cfg(feature = "ledger")]
use crate::lib::ledger::LedgerIdentity;
use crate::lib::{
    get_account_id, get_principal, AnyhowResult, AuthInfo, ParsedAccount, ParsedSubaccount,
};
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use icp_ledger::AccountIdentifier;
use icrc_ledger_types::icrc1::account::Account;
use k256::{elliptic_curve::sec1::ToEncodedPoint, SecretKey};
use sha3::{Digest, Keccak256};

#[derive(Parser)]
/// Prints the principal and the account IDs.
pub struct PublicOpts {
    /// Principal for which to get the account_id.
    #[clap(long)]
    principal_id: Option<String>,
    /// Additionally prints the legacy DFN address for Genesis claims.
    #[clap(long, conflicts_with = "principal-id")]
    genesis_dfn: bool,
    /// If authenticating with a Ledger device, display the public IDs on the device.
    #[cfg_attr(not(feature = "ledger"), clap(hidden = true))]
    #[clap(long, requires = "ledgerhq")]
    display_on_ledger: bool,
    /// Print IDs for the provided subaccount.
    #[clap(long)]
    subaccount: Option<ParsedSubaccount>,
}

/// Prints the account and the principal ids.
pub fn exec(auth: &AuthInfo, opts: PublicOpts) -> AnyhowResult {
    let (principal_id, account_id) = get_public_ids(auth, &opts)?;
    println!("Principal id: {principal_id}");
    println!("Legacy account id: {account_id}");
    if let Some(sub) = opts.subaccount {
        println!(
            "ICRC-1 account id: {}",
            ParsedAccount(Account {
                owner: principal_id,
                subaccount: Some(sub.0 .0)
            })
        );
    }
    if opts.genesis_dfn {
        let AuthInfo::PemFile(pem) = auth else {
            bail!("Must supply a pem or seed file for the DFN address");
        };
        println!("DFN address: {}", get_dfn(pem)?);
    }
    if opts.display_on_ledger {
        #[cfg(feature = "ledger")]
        {
            LedgerIdentity::new()?.display_pk()?;
        }
        #[cfg(not(feature = "ledger"))]
        {
            bail!("This build of quill does not support Ledger functionality.");
        }
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
            Ok((
                principal_id,
                get_account_id(principal_id, opts.subaccount.map(|x| x.0))?,
            ))
        }
        None => {
            if let AuthInfo::NoAuth = auth {
                Err(anyhow!(
                    "public-ids cannot be used without specifying a private key"
                ))
            } else {
                let principal_id = get_principal(auth)?;
                Ok((
                    principal_id,
                    get_account_id(principal_id, opts.subaccount.map(|x| x.0))?,
                ))
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
