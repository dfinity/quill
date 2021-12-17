use crate::lib::AnyhowResult;
use anyhow::anyhow;
use bip39::{Language, Mnemonic};
use clap::Parser;
use libsecp256k1::{PublicKey, SecretKey};
use pem::{encode, Pem};
use rand::{rngs::OsRng, RngCore};
use simple_asn1::ASN1Block::{
    BitString, Explicit, Integer, ObjectIdentifier, OctetString, Sequence,
};
use simple_asn1::{oid, to_der, ASN1Class, BigInt, BigUint};
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct GenerateOpts {
    /// Number of words: 12 or 24.
    #[clap(long, default_value = "12")]
    words: u32,

    /// File to write the seed phrase to.
    #[clap(long, default_value = "seed.txt")]
    seed_file: String,

    /// File to write the PEM to.
    #[clap(long, default_value = "identity.pem")]
    pem_file: String,

    /// A seed phrase in quotes to use to generate the PEM file.
    #[clap(long)]
    phrase: Option<String>,

    /// Overwrite any existing seed file.
    #[clap(long)]
    overwrite_seed_file: bool,

    /// Overwrite any existing PEM file.
    #[clap(long)]
    overwrite_pem_file: bool,
}

pub fn der_encode_secret_key(public_key: Vec<u8>, secret: Vec<u8>) -> Vec<u8> {
    let secp256k1_id = ObjectIdentifier(0, oid!(1, 3, 132, 0, 10));
    let data = Sequence(
        0,
        vec![
            Integer(0, BigInt::from(1)),
            OctetString(32, secret.to_vec()),
            Explicit(
                ASN1Class::ContextSpecific,
                0,
                BigUint::from(0u32),
                Box::new(secp256k1_id),
            ),
            Explicit(
                ASN1Class::ContextSpecific,
                0,
                BigUint::from(1u32),
                Box::new(BitString(0, public_key.len() * 8, public_key)),
            ),
        ],
    );
    to_der(&data).expect("Cannot encode secret key.")
}

/// Generate or recover mnemonic seed phrase and/or PEM file.
pub fn exec(opts: GenerateOpts) -> AnyhowResult {
    if Path::new(&opts.seed_file).exists() && !opts.overwrite_seed_file {
        return Err(anyhow!("Seed file exists and overwrite is not set."));
    }
    if Path::new(&opts.pem_file).exists() && !opts.overwrite_pem_file {
        return Err(anyhow!("PEM file exists and overwrite is not set."));
    }
    let bytes = match opts.words {
        12 => 16,
        24 => 32,
        _ => return Err(anyhow!("Words must be 12 or 24.")),
    };
    let mnemonic = match opts.phrase {
        Some(phrase) => Mnemonic::parse(phrase).unwrap(),
        None => {
            let mut key = vec![0u8; bytes];
            OsRng.fill_bytes(&mut key);
            Mnemonic::from_entropy_in(Language::English, &key).unwrap()
        }
    };
    let pem = mnemonic_to_pem(&mnemonic);
    let mut phrase = mnemonic
        .word_iter()
        .collect::<Vec<&'static str>>()
        .join(" ");
    phrase.push('\n');
    std::fs::write(opts.seed_file, phrase)?;
    std::fs::write(opts.pem_file, pem.clone())?;
    let (principal_id, account_id) = crate::commands::public::get_ids(&Some(pem))?;
    println!("Principal id: {}", principal_id);
    println!("Account id: {}", account_id);
    Ok(())
}

/// Converts menmonic to PEM format
pub fn mnemonic_to_pem(mnemonic: &Mnemonic) -> String {
    let seed = mnemonic.to_seed("");
    let ext = tiny_hderive::bip32::ExtendedPrivKey::derive(&seed, "m/44'/223'/0'/0/0").unwrap();
    let secret = ext.secret();
    let secret_key = SecretKey::parse(&secret).unwrap();
    let public_key = PublicKey::from_secret_key(&secret_key);
    let der = der_encode_secret_key(public_key.serialize().to_vec(), secret.to_vec());
    let pem = Pem {
        tag: String::from("EC PRIVATE KEY"),
        contents: der,
    };
    encode(&pem).replace("\r", "").replace("\n\n", "\n")
}
