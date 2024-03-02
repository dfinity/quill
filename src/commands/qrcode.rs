use std::path::PathBuf;

use crate::lib::{read_from_file, AnyhowResult};
use anyhow::Context;
use clap::Parser;
use qrcodegen::{QrCode, QrCodeEcc};

/// Print QR code for data e.g. principal id.
#[derive(Parser)]
pub struct QRCodeOpts {
    /// File the contents of which to be output as a QRCode.
    #[clap(long)]
    file: Option<PathBuf>,

    // String to be output as a QRCode.
    #[clap(long)]
    string: Option<String>,
}

/// Prints the account and the principal ids.
pub fn exec(opts: QRCodeOpts) -> AnyhowResult {
    if let Some(file) = opts.file {
        let data = read_from_file(file)?;
        print_qr(&data)?;
    }
    if let Some(string) = opts.string {
        print_qr(&string)?;
    }
    Ok(())
}

// Prints the given QrCode object to the console.
pub fn print_qr(text: &str) -> AnyhowResult {
    let errcorlvl: QrCodeEcc = QrCodeEcc::Medium; // Error correction level

    // Make and print the QR Code symbol
    let qr: QrCode = QrCode::encode_text(text, errcorlvl)
        .context("Could not encode QR code (data too large)")?;

    let border: i32 = 4;
    for y in -border / 2..=qr.size() / 2 + border / 2 {
        for x in -border..qr.size() + border {
            let c = match (!qr.get_module(x, y * 2), !qr.get_module(x, y * 2 + 1)) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            };
            print!("{c}");
        }
        println!();
    }
    println!();
    Ok(())
}
