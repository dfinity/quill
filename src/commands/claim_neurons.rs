#[cfg(feature = "ledger")]
use crate::lib::ledger::LedgerIdentity;
use crate::lib::{
    genesis_token_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_NNS_GTC,
};
use anyhow::{anyhow, Context};
use candid::Encode;
use clap::Parser;
use k256::{elliptic_curve::sec1::ToEncodedPoint, SecretKey};

/// Claim seed neurons from the Genesis Token Canister.
#[derive(Parser)]
pub struct ClaimNeuronOpts;

pub fn exec(auth: &AuthInfo) -> AnyhowResult<Vec<IngressWithRequestId>> {
    if let AuthInfo::PemFile(pem) = auth {
        let keyinfo = pem::parse_many(pem)?
            .into_iter()
            .find(|p| p.tag == "EC PRIVATE KEY")
            .context("Pem file did not contain sec1 key")?;
        let point = SecretKey::from_sec1_der(&keyinfo.contents)
            .map_err(|e| anyhow!("could not load pem file: {e}"))?
            .public_key()
            .to_encoded_point(false);
        let sig = Encode!(&hex::encode(point.as_bytes()))?;

        Ok(vec![sign_ingress_with_request_status_query(
            auth,
            genesis_token_canister_id(),
            ROLE_NNS_GTC,
            "claim_neurons",
            sig,
        )?])
    } else {
        #[cfg(feature = "ledger")]
        if let AuthInfo::Ledger = auth {
            let (_, pk) = LedgerIdentity::new()?.public_key()?;
            let sig = Encode!(&hex::encode(pk))?;
            Ok(vec![sign_ingress_with_request_status_query(
                auth,
                genesis_token_canister_id(),
                ROLE_NNS_GTC,
                "claim_neurons",
                sig,
            )?])
        } else {
            Err(anyhow!(
                "claim-neurons command requires --pem-file or --ledger to be specified"
            ))
        }
        #[cfg(not(feature = "ledger"))]
        Err(anyhow!(
            "claim-neurons command requires --pem-file to be specified"
        ))
    }
}
