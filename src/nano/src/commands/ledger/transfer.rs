use crate::commands::ledger::get_icpts_from_args;
use crate::commands::sign;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::account_identifier::AccountIdentifier;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::nns_types::{Memo, SendArgs, LEDGER_CANISTER_ID};
use crate::lib::operations::canister::get_local_candid_path;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::validators::{e8s_validator, icpts_amount_validator, memo_validator};
use crate::util::{get_candid_type, get_idl_string};
use anyhow::anyhow;
use candid::Encode;
use clap::Clap;
use ic_types::principal::Principal;
use std::str::FromStr;

const SEND_METHOD: &str = "send_dfx";

/// Transfer ICP from the user to the destination AccountIdentifier
#[derive(Clap)]
pub struct TransferOpts {
    /// AccountIdentifier of transfer destination.
    to: String,

    /// ICPs to transfer to the destination AccountIdentifier
    /// Can be specified as a Decimal with the fractional portion up to 8 decimal places
    /// i.e. 100.012
    #[clap(long, validator(icpts_amount_validator))]
    amount: Option<String>,

    /// Specify ICP as a whole number, helpful for use in conjunction with `--e8s`
    #[clap(long, validator(e8s_validator), conflicts_with("amount"))]
    icp: Option<String>,

    /// Specify e8s as a whole number, helpful for use in conjunction with `--icp`
    #[clap(long, validator(e8s_validator), conflicts_with("amount"))]
    e8s: Option<String>,

    /// Specify a numeric memo for this transaction.
    #[clap(long, validator(memo_validator))]
    memo: String,

    /// Transaction fee, default is 10000 e8s.
    #[clap(long, validator(icpts_amount_validator))]
    fee: Option<String>,

    /// Sign the transaction and save to the file.
    #[clap(long)]
    file: String,
}

pub async fn exec(env: &dyn Environment, opts: TransferOpts) -> DfxResult {
    let amount = get_icpts_from_args(opts.amount, opts.icp, opts.e8s)?;

    let fee = opts.fee.map_or(Ok(TRANSACTION_FEE), |v| {
        ICPTs::from_str(&v).map_err(|err| anyhow!(err))
    })?;

    // validated by memo_validator
    let memo = Memo(opts.memo.parse::<u64>().unwrap());

    let to = AccountIdentifier::from_str(&opts.to).map_err(|err| anyhow!(err))?;

    fetch_root_key_if_needed(env).await?;

    let canister_id = Principal::from_text(LEDGER_CANISTER_ID)?;

    let args = Encode!(&SendArgs {
        memo,
        amount,
        fee,
        from_subaccount: None,
        to,
        created_at_time: None,
    })?;

    let path = get_local_candid_path(canister_id.clone());
    let method_type = path.and_then(|path| get_candid_type(&path, &SEND_METHOD));
    let argument = Some(get_idl_string(&args, Some("raw"), &method_type)?);
    let opts = sign::SignOpts {
        canister_name: canister_id.to_string(),
        method_name: SEND_METHOD.to_string(),
        query: false,
        update: true,
        argument,
        random: None,
        r#type: Some("raw".to_string()),
        expire_after: "5m".to_string(),
        file: opts.file,
    };
    sign::exec(env, opts).await
}
