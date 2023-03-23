use std::str::FromStr;

use anyhow::bail;
use candid::{Nat, Principal};
use clap::Subcommand;
use icrc_ledger_types::Account;
use openssl::sha::Sha256;
use rust_decimal::Decimal;

use crate::lib::{ckbtc_minter_canister_id, AnyhowResult, AuthInfo};

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
    Balance(balance::BalanceOpts),
    UpdateBalance(update_balance::UpdateBalanceOpts),
    Transfer(transfer::TransferOpts),
    RetrieveBtc(retrieve_btc::RetrieveBtcOpts),
    RetrieveBtcStatus(retrieve_btc_status::RetrieveBtcStatusOpts),
    WithdrawalAddress(withdrawal_address::GetWithdrawalAddressOpts),
}

pub fn dispatch(
    auth: &AuthInfo,
    command: CkbtcCommand,
    qr: bool,
    fetch_root_key: bool,
) -> AnyhowResult {
    match command {
        CkbtcCommand::UpdateBalance(opts) => {
            let out = update_balance::exec(auth, opts)?;
            print_vec(qr, &out)?;
        }
        CkbtcCommand::Transfer(opts) => {
            let out = transfer::exec(auth, opts)?;
            print_vec(qr, &out)?;
        }
        CkbtcCommand::RetrieveBtc(opts) => {
            let out = retrieve_btc::exec(auth, opts)?;
            print_vec(qr, &out)?;
        }
        CkbtcCommand::RetrieveBtcStatus(opts) => {
            retrieve_btc_status::exec(opts, fetch_root_key)?;
        }
        CkbtcCommand::Balance(opts) => {
            balance::exec(auth, opts, fetch_root_key)?;
        }
        CkbtcCommand::WithdrawalAddress(opts) => {
            withdrawal_address::exec(auth, opts)?;
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
        owner: ckbtc_minter_canister_id(testnet),
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
