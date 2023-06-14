#[cfg(feature = "ledger")]
use crate::lib::ledger::LedgerIdentity;
use crate::lib::{
    genesis_token_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_NNS_GTC,
};
use anyhow::anyhow;
use candid::Encode;
use clap::Parser;
use openssl::bn::BigNumContext;
use openssl::ec::{EcKey, PointConversionForm};

/// Claim seed neurons from the Genesis Token Canister.
#[derive(Parser)]
pub struct ClaimNeuronOpts;

pub fn exec(auth: &AuthInfo) -> AnyhowResult<Vec<IngressWithRequestId>> {
    if let AuthInfo::PemFile(pem) = auth {
        let private_key = EcKey::private_key_from_pem(pem.as_bytes())?;
        let group = private_key.group();
        let public_key = EcKey::from_public_key(group, private_key.public_key())?;
        let mut context = BigNumContext::new()?;
        let bytes = public_key.public_key().to_bytes(
            public_key.group(),
            PointConversionForm::UNCOMPRESSED,
            &mut context,
        )?;
        let sig = Encode!(&hex::encode(bytes))?;

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
