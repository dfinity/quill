use crate::commands::send::{Memo, SendArgs};
use crate::lib::{
    ledger_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo,
};
use anyhow::{anyhow, bail, Context};
use candid::Encode;
use clap::Parser;
use ledger_canister::{Tokens, DEFAULT_TRANSFER_FEE};

/// Signs an ICP transfer transaction.
#[derive(Default, Parser)]
pub struct TransferOpts {
    /// Destination account.
    pub to: String,

    /// Amount of ICPs to transfer (with up to 8 decimal digits after comma).
    #[clap(long, validator(token_amount_validator))]
    pub amount: String,

    /// Reference number, default is 0.
    #[clap(long, validator(memo_validator))]
    pub memo: Option<String>,

    /// Transaction fee, default is 10000 e8s.
    #[clap(long, validator(token_amount_validator))]
    pub fee: Option<String>,
}

pub fn exec(auth: &AuthInfo, opts: TransferOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let amount = parse_tokens(&opts.amount).context("Cannot parse amount")?;
    let fee = opts.fee.map_or(Ok(DEFAULT_TRANSFER_FEE), |v| {
        parse_tokens(&v).context("Cannot parse fee")
    })?;
    let memo = Memo(
        opts.memo
            .unwrap_or_else(|| "0".to_string())
            .parse::<u64>()
            .context("Failed to parse memo as unsigned integer")?,
    );
    let to = opts.to;

    let args = Encode!(&SendArgs {
        memo,
        amount,
        fee,
        from_subaccount: None,
        to,
        created_at_time: None,
    })?;

    let msg = sign_ingress_with_request_status_query(auth, ledger_canister_id(), "send_dfx", args)?;
    Ok(vec![msg])
}

fn new_tokens(tokens: u64, e8s: u64) -> AnyhowResult<Tokens> {
    Tokens::new(tokens, e8s)
        .map_err(|err| anyhow!(err))
        .context("Cannot create new tokens structure")
}

fn parse_tokens(amount: &str) -> AnyhowResult<Tokens> {
    let parse = |s: &str| {
        s.parse::<u64>()
            .context("Failed to parse tokens as unsigned integer")
    };
    match &amount.split('.').collect::<Vec<_>>().as_slice() {
        [tokens] => new_tokens(parse(tokens)?, 0),
        [tokens, e8s] => {
            let mut e8s = e8s.to_string();
            while e8s.len() < 8 {
                e8s.push('0');
            }
            let e8s = &e8s[..8];
            new_tokens(parse(tokens)?, parse(e8s)?)
        }
        _ => bail!("Cannot parse amount {}", amount),
    }
}

fn token_amount_validator(tokens: &str) -> AnyhowResult<()> {
    parse_tokens(tokens).map(|_| ())
}

fn memo_validator(memo: &str) -> Result<(), String> {
    if memo.parse::<u64>().is_ok() {
        return Ok(());
    }
    Err("Memo must be an unsigned integer".to_string())
}
