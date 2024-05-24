use std::path::PathBuf;

use anyhow::bail;
use clap::Parser;
use sec1::LineEnding;

use crate::{
    lib::{AnyhowResult, AuthInfo},
    write_file,
};

/// Decrypts an encrypted PEM file for use with other tools.
#[derive(Parser)]
pub struct DecryptPemOpts {
    /// The path to write the decrypted PEM to, or "-" for STDOUT
    output_path: PathBuf,
}

pub fn exec(auth: &AuthInfo, opts: DecryptPemOpts) -> AnyhowResult<()> {
    let AuthInfo::K256Key(pk) = auth else {
        bail!("--pem-file was not set to an encrypted PEM file")
    };
    // technically this permits an unencrypted secp256k1 key, which will just be re-encoded directly
    // but who cares
    write_file(
        &opts.output_path,
        "PEM",
        pk.to_sec1_pem(LineEnding::default())?.as_bytes(),
    )?;
    eprintln!("Wrote PEM file to {}", opts.output_path.display());
    Ok(())
}
