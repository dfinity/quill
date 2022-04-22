use crate::commands::send::send_unsigned_ingress;
use crate::lib::TargetCanister;
use crate::{AnyhowResult, SnsCanisterIds};
use anyhow::Error;
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ledger_canister::{AccountIdentifier, BinaryAccountBalanceArgs};

/// Signs a ledger account-balance query call.
#[derive(Parser)]
pub struct AccountBalanceOpts {
    /// The AccountIdentifier of the account to query. For example: d5662fbce449fbd4adb4b9aff6c59035bd93e7c2eff5010a446ebc3dd81007f8
    account_id: String,

    /// Will display the query, but not send it
    #[clap(long)]
    dry_run: bool,
}

pub async fn exec(sns_canister_ids: &SnsCanisterIds, opts: AccountBalanceOpts) -> AnyhowResult {
    let account_identifier = AccountIdentifier::from_hex(&opts.account_id).map_err(Error::msg)?;
    let ledger_canister_id = PrincipalId::from(sns_canister_ids.ledger_canister_id).0;

    let args = Encode!(&BinaryAccountBalanceArgs {
        account: account_identifier.to_address()
    })?;

    send_unsigned_ingress(
        "account_balance",
        args,
        opts.dry_run,
        TargetCanister::Ledger(ledger_canister_id),
    )
    .await?;

    Ok(())
}
