// Copied from https://raw.githubusercontent.com/dfinity/ic/master/rs/rosetta-api/ledger_canister/src/icpts.rs
// Commit: 779549eccfcf61ac702dfc2ee6d76ffdc2db1f7f

use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(
    Serialize,
    Deserialize,
    CandidType,
    Clone,
    Copy,
    Hash,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct ICPTs {
    /// Number of 10^-8 ICPs.
    /// Named because the equivalent part of a Bitcoin is called a Satoshi
    e8s: u64,
}

/// How many times can ICPs be divided
pub const ICP_SUBDIVIDABLE_BY: u64 = 100_000_000;

/// This is 1/10,000th of an ICP, this is probably more than it costs us to
/// store a transaction so it will likely come down in the future
pub const TRANSACTION_FEE: ICPTs = ICPTs { e8s: 10_000 };

impl ICPTs {
    /// Construct a new instance of ICPTs.
    /// This function will not allow you use more than 1 ICPTs worth of E8s.
    pub fn new(icpt: u64, e8s: u64) -> Result<Self, String> {
        static CONSTRUCTION_FAILED: &str =
            "Constructing ICP failed because the underlying u64 overflowed";

        let icp_part = icpt
            .checked_mul(ICP_SUBDIVIDABLE_BY)
            .ok_or_else(|| CONSTRUCTION_FAILED.to_string())?;
        if e8s >= ICP_SUBDIVIDABLE_BY {
            return Err(format!(
                "You've added too many E8s, make sure there are less than {}",
                ICP_SUBDIVIDABLE_BY
            ));
        }
        let e8s = icp_part
            .checked_add(e8s)
            .ok_or_else(|| CONSTRUCTION_FAILED.to_string())?;
        Ok(Self { e8s })
    }

    /// Gets the total number of whole ICPTs
    /// ```
    /// # use ledger_canister::ICPTs;
    /// let icpt = ICPTs::new(12, 200).unwrap();
    /// assert_eq!(icpt.get_icpts(), 12)
    /// ```
    pub fn get_icpts(self) -> u64 {
        self.e8s / ICP_SUBDIVIDABLE_BY
    }

    /// Gets the total number of E8s not part of a whole ICPT
    /// The returned amount is always in the half-open interval [0, 1 ICP).
    /// ```
    /// # use ledger_canister::ICPTs;
    /// let icpt = ICPTs::new(12, 200).unwrap();
    /// assert_eq!(icpt.get_remainder_e8s(), 200)
    /// ```
    pub fn get_remainder_e8s(self) -> u64 {
        self.e8s % ICP_SUBDIVIDABLE_BY
    }
}

/// ```
/// # use ledger_canister::ICPTs;
/// let icpt = ICPTs::new(12, 200).unwrap();
/// let s = format!("{}", icpt);
/// assert_eq!(&s[..], "12.00000200 ICP")
/// ```
impl fmt::Display for ICPTs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}.{:08} ICP",
            self.get_icpts(),
            self.get_remainder_e8s()
        )
    }
}
