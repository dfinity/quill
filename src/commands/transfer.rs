use crate::commands::{
    request_status,
    send::{Memo, SendArgs},
    sign::sign,
};
use crate::lib::{
    nns_types::account_identifier::AccountIdentifier,
    nns_types::icpts::{ICPTs, TRANSACTION_FEE},
    AnyhowResult, LEDGER_CANISTER_ID,
};
use anyhow::anyhow;
use candid::Encode;
use clap::Clap;
use ic_types::principal::Principal;
use std::str::FromStr;

const SEND_METHOD: &str = "send_dfx";

/// Signs an ICP transfer transaction.
#[derive(Default, Clap)]
pub struct TransferOpts {
    /// Destination account.
    pub to: String,

    /// Amount of ICPs to transfer (with up to 8 decimal digits after comma).
    #[clap(long, validator(icpts_amount_validator))]
    pub amount: Option<String>,

    /// Reference number.
    #[clap(long, validator(memo_validator))]
    pub memo: Option<String>,

    /// Transaction fee, default is 10000 e8s.
    #[clap(long, validator(icpts_amount_validator))]
    pub fee: Option<String>,
}

pub async fn exec(pem: &Option<String>, opts: TransferOpts) -> AnyhowResult<String> {
    let amount = parse_icpts(&opts.amount.unwrap())
        .map_err(|err| anyhow!("Could not add ICPs and e8s: {}", err))?;
    let fee = opts.fee.map_or(Ok(TRANSACTION_FEE), |v| {
        parse_icpts(&v).map_err(|err| anyhow!(err))
    })?;
    let memo = Memo(opts.memo.unwrap_or("0".to_string()).parse::<u64>().unwrap());
    let to = AccountIdentifier::from_str(&opts.to).map_err(|err| anyhow!(err))?;
    let canister_id = Principal::from_text(LEDGER_CANISTER_ID)?;

    let args = Encode!(&SendArgs {
        memo,
        amount,
        fee,
        from_subaccount: None,
        to,
        created_at_time: None,
    })?;

    let msg_with_req_id = sign(pem, canister_id.clone(), SEND_METHOD, args).await?;
    let request_id = msg_with_req_id
        .request_id
        .expect("No request id for transfer call found");
    let req_status_signed_msg = request_status::sign(pem, request_id, canister_id).await?;

    let mut out = String::new();
    out.push_str("{ \"ingress\": ");
    out.push_str(&msg_with_req_id.buffer);
    out.push_str(", \"request_status\": ");
    out.push_str(&req_status_signed_msg);
    out.push_str("}");

    Ok(out)
}

fn parse_icpts(amount: &str) -> Result<ICPTs, String> {
    let mut it = amount.split(".");
    let icpts = it
        .next()
        .unwrap_or("0")
        .parse::<u64>()
        .map_err(|err| format!("Couldn't parse icpts: {:?}", err))?;

    let mut e8s = it.next().unwrap_or("0").to_string();
    while e8s.len() < 8 {
        e8s.push_str("0");
    }
    let e8s = e8s
        .parse::<u64>()
        .map_err(|err| format!("Couldn't parse e8s: {:?}", err))?;

    ICPTs::new(icpts, e8s)
}

fn icpts_amount_validator(icpts: &str) -> Result<(), String> {
    parse_icpts(icpts).map(|_| ())
}

fn memo_validator(memo: &str) -> Result<(), String> {
    if memo.parse::<u64>().is_ok() {
        return Ok(());
    }
    Err("Memo must be an unsigned integer".to_string())
}
