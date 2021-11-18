use crate::{
    commands::sign::sign_ingress_with_request_status_query,
    lib::{genesis_token_canister_id, sign::signed_message::IngressWithRequestId, AnyhowResult},
};
use candid::Encode;
use openssl::bn::BigNumContext;
use openssl::ec::{EcKey, PointConversionForm};

pub async fn exec(pem: &Option<String>) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let private_key = EcKey::private_key_from_pem(
        &pem.as_ref()
            .expect("claim-neurons requires a PEM private key")
            .as_bytes(),
    )?;
    let group = private_key.group();
    let public_key = EcKey::from_public_key(group, private_key.public_key())?;
    let mut context = BigNumContext::new()?;
    let bytes = public_key.public_key().to_bytes(
        public_key.group(),
        PointConversionForm::UNCOMPRESSED,
        &mut context,
    )?;
    let sig = Encode!(&hex::encode(&bytes))?;

    let mut messages = Vec::new();
    messages.push(
        sign_ingress_with_request_status_query(
            pem,
            genesis_token_canister_id(),
            "claim_neurons",
            sig,
        )
        .await?,
    );
    Ok(messages)
}
