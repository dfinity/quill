use crate::{
    commands::{send::submit_unsigned_ingress, SendingOpts},
    lib::{
        ledger_canister_id, AnyhowResult, AuthInfo, ParsedNnsAccount, ROLE_ICRC1_LEDGER,
        ROLE_NNS_LEDGER,
    },
    AUTH_FLAGS,
};
use candid::Encode;
use clap::Parser;
use icp_ledger::BinaryAccountBalanceArgs;
use icrc_ledger_types::icrc1::account::Account;

use super::get_principal;

/// Queries a ledger account balance.
#[derive(Parser)]
pub struct AccountBalanceOpts {
    /// The id of the account to query. Optional if a key is used.
    #[arg(required_unless_present_any = AUTH_FLAGS)]
    account_id: Option<ParsedNnsAccount>,

    #[command(flatten)]
    sending_opts: SendingOpts,
}

// We currently only support a subset of the functionality.
#[tokio::main]
pub async fn exec(auth: &AuthInfo, opts: AccountBalanceOpts, fetch_root_key: bool) -> AnyhowResult {
    let account_id = if let Some(id) = opts.account_id {
        id
    } else {
        let account = Account {
            owner: get_principal(auth)?,
            subaccount: None,
        };
        ParsedNnsAccount::Icrc1(account)
    };
    match account_id {
        ParsedNnsAccount::Original(id) => {
            let args = Encode!(&BinaryAccountBalanceArgs {
                account: id.to_address()
            })?;
            submit_unsigned_ingress(
                ledger_canister_id(),
                ROLE_NNS_LEDGER,
                "account_balance",
                args,
                opts.sending_opts,
                fetch_root_key,
            )
            .await
        }
        ParsedNnsAccount::Icrc1(id) => {
            let args = Encode!(&id)?;
            submit_unsigned_ingress(
                ledger_canister_id(),
                ROLE_ICRC1_LEDGER,
                "icrc1_balance_of",
                args,
                opts.sending_opts,
                fetch_root_key,
            )
            .await
        }
    }
}
