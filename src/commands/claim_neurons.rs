use crate::lib::{
    genesis_token_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_NNS_GTC,
};
use anyhow::anyhow;
use candid::Encode;
use clap::Parser;
use k256::elliptic_curve::sec1::ToEncodedPoint;

/// Claim seed neurons from the Genesis Token Canister.
#[derive(Parser)]
pub struct ClaimNeuronOpts;

pub fn exec(auth: &AuthInfo) -> AnyhowResult<Vec<IngressWithRequestId>> {
    if let AuthInfo::K256Key(pk) = auth {
        let point = pk.public_key().to_encoded_point(false);
        let sig = Encode!(&hex::encode(point.as_bytes()))?;

        Ok(vec![sign_ingress_with_request_status_query(
            auth,
            genesis_token_canister_id(),
            ROLE_NNS_GTC,
            "claim_neurons",
            sig,
        )?])
    } else {
        Err(anyhow!(
            "claim-neurons command requires --pem-file to be specified"
        ))
    }
}
