use crate::lib::{
    genesis_token_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_NNS_GTC,
};
use anyhow::anyhow;
use candid::Encode;
use openssl::bn::BigNumContext;
use openssl::ec::{EcKey, PointConversionForm};

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
        Err(anyhow!(
            "claim-neurons command requires a --pem-file to be specified"
        ))
    }
}
