// DISCLAIMER:
// Do not modify this file arbitrarily.
// The contents are borrowed from:
// dfinity-lab/dfinity@25999dd54d29c24edb31483801bddfd8c1d780c8

use candid::CandidType;
use ic_types::principal::Principal;
use serde::{Deserialize, Serialize};

pub mod account_identifier;
pub mod icpts;

pub const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
pub const GOVERNANCE_CANISTER_ID: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";

#[derive(
    Serialize, Deserialize, CandidType, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Memo(pub u64);

impl Default for Memo {
    fn default() -> Memo {
        Memo(0)
    }
}

#[derive(CandidType)]
pub struct AccountBalanceArgs {
    pub account: String,
}

#[derive(CandidType)]
pub struct TimeStamp {
    pub timestamp_nanos: u64,
}

#[derive(CandidType)]
pub struct ClaimOrRefreshNeuronFromAccount {
    pub memo: Memo,
    pub controller: Option<Principal>,
}

#[derive(CandidType)]
pub struct SendArgs {
    pub memo: Memo,
    pub amount: icpts::ICPTs,
    pub fee: icpts::ICPTs,
    pub from_subaccount: Option<account_identifier::Subaccount>,
    pub to: account_identifier::AccountIdentifier,
    pub created_at_time: Option<TimeStamp>,
}
