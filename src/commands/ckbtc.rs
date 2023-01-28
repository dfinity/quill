use std::str::FromStr;

use anyhow::bail;
use candid::{Nat, Principal};
use clap::Subcommand;
use ic_icrc1::Account;
use openssl::sha::Sha256;
use rust_decimal::Decimal;

use crate::{
    get_auth,
    lib::{ckbtc_minter_canister_id, AnyhowResult},
    BaseOpts,
};

use super::print_vec;

mod balance;
mod retrieve_btc;
mod retrieve_btc_status;
mod transfer;
mod update_balance;
mod withdrawal_address;

/// Commands for chain-key bitcoin (ckBTC)
#[derive(Subcommand)]
pub enum CkbtcCommand {
    Balance(BaseOpts<balance::BalanceOpts>),
    UpdateBalance(BaseOpts<update_balance::UpdateBalanceOpts>),
    Transfer(BaseOpts<transfer::TransferOpts>),
    RetrieveBtc(BaseOpts<retrieve_btc::RetrieveBtcOpts>),
    RetrieveBtcStatus(BaseOpts<retrieve_btc_status::RetrieveBtcStatusOpts>),
    WithdrawalAddress(BaseOpts<withdrawal_address::GetWithdrawalAddressOpts>),
}

pub fn dispatch(command: CkbtcCommand) -> AnyhowResult {
    match command {
        CkbtcCommand::UpdateBalance(opts) => {
            let qr = opts.global_opts.qr;
            let out = update_balance::exec(&get_auth(opts.global_opts)?, opts.command_opts)?;
            print_vec(qr, &out)?;
        }
        CkbtcCommand::Transfer(opts) => {
            let qr = opts.global_opts.qr;
            let out = transfer::exec(&get_auth(opts.global_opts)?, opts.command_opts)?;
            print_vec(qr, &out)?;
        }
        CkbtcCommand::RetrieveBtc(opts) => {
            let qr = opts.global_opts.qr;
            let out = retrieve_btc::exec(&get_auth(opts.global_opts)?, opts.command_opts)?;
            print_vec(qr, &out)?;
        }
        CkbtcCommand::RetrieveBtcStatus(opts) => {
            retrieve_btc_status::exec(opts.command_opts, opts.global_opts.fetch_root_key)?;
        }
        CkbtcCommand::Balance(opts) => {
            let fetch_root_key = opts.global_opts.fetch_root_key;
            balance::exec(
                &get_auth(opts.global_opts)?,
                opts.command_opts,
                fetch_root_key,
            )?;
        }
        CkbtcCommand::WithdrawalAddress(opts) => {
            withdrawal_address::exec(&get_auth(opts.global_opts)?, opts.command_opts)?;
        }
    }
    Ok(())
}

pub struct Btc(pub Nat);

impl FromStr for Btc {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut dec = Decimal::from_str(s)?;
        if dec.scale() > 8 {
            bail!("Bitcoin can only be specified to the 8th decimal.");
        }
        if !dec.is_sign_positive() {
            bail!("Must specify a positive number");
        }
        dec.rescale(8);
        Ok(Self((dec.mantissa() as u128).into()))
    }
}

// Corresponds to ckbtc_minter.get_withdrawal_address(). We do not actually need to make the call
// because the algorithm is considered stable.
fn ckbtc_withdrawal_address(user: &Principal, testnet: bool) -> Account {
    const DOMAIN: &str = "ckbtc";
    let mut hasher = Sha256::new();
    hasher.update(&[DOMAIN.len() as u8]);
    hasher.update(DOMAIN.as_bytes());
    hasher.update(user.as_slice());
    hasher.update(&[0; 8]);
    Account {
        owner: ckbtc_minter_canister_id(testnet).into(),
        subaccount: Some(hasher.finish()),
    }
}

#[cfg(test)]
mod tests {
    use super::Btc;
    use std::str::FromStr;

    #[test]
    fn btc() {
        let btc = Btc::from_str("73.25").unwrap();
        assert_eq!(btc.0, 7_325_000_000_u64)
    }
}
