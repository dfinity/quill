use crate::lib::qr::print_qr;
use crate::lib::{read_from_file, AnyhowResult};
use clap::Parser;

#[derive(Parser)]
pub struct QRCodeOpts {
    /// File the contents of which to be output as a QRCode.
    #[clap(long)]
    file: Option<String>,

    // String to be output as a QRCode.
    #[clap(long)]
    string: Option<String>,
}

/// Prints the account and the principal ids.
pub fn exec(opts: QRCodeOpts) -> AnyhowResult {
    if let Some(file) = opts.file {
        let data = read_from_file(&file)?;
        print_qr(&data);
    }
    if let Some(string) = opts.string {
        print_qr(&string);
    }
    Ok(())
}
