use crate::lib::{
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, TargetCanister,
};
use crate::CanisterIds;
use anyhow::{anyhow, bail, Context, Error};
use candid::{CandidType, Encode};
use clap::Parser;
use ic_base_types::PrincipalId;
use ledger_canister::{AccountIdentifier, Memo, TimeStamp, Tokens, DEFAULT_TRANSFER_FEE};

/// Arguments for the `transfer` call.
#[derive(CandidType)]
pub struct TransferArgs {
    /// Transaction memo.
    pub memo: Memo,

    /// The amount that the caller wants to transfer to the destination address.
    pub amount: Tokens,

    /// The amount that the caller pays for the transaction.
    /// Must be 10000 e8s.
    pub fee: Tokens,

    /// The subaccount from which the caller wants to transfer funds.
    pub from_subaccount: Option<Vec<u8>>,

    /// The destination account.
    pub to: Vec<u8>,

    /// The point in time when the caller created this request.
    pub created_at_time: Option<TimeStamp>,
}

/// Signs a ledger transfer update call.
#[derive(Default, Parser)]
pub struct TransferOpts {
    /// Destination account.
    pub to: String,

    /// Amount of Governance Tokens to transfer (with up to 8 decimal digits after comma).
    #[clap(long, validator(tokens_amount_validator))]
    pub amount: String,

    /// Reference number, default is 0.
    #[clap(long, validator(memo_validator))]
    pub memo: Option<String>,

    /// Transaction fee, default is 10_000 e8s.
    #[clap(long, validator(tokens_amount_validator))]
    pub fee: Option<String>,
}

pub fn exec(
    pem: &str,
    canister_ids: &CanisterIds,
    opts: TransferOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
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
    let ledger_canister_id = PrincipalId::from(canister_ids.ledger_canister_id).0;
    let to_account_identifier = AccountIdentifier::from_hex(&opts.to).map_err(Error::msg)?;

    let args = Encode!(&TransferArgs {
        memo,
        amount,
        fee,
        from_subaccount: None,
        to: to_account_identifier.to_vec(),
        created_at_time: None,
    })?;

    let msg = sign_ingress_with_request_status_query(
        pem,
        ledger_canister_id,
        "transfer",
        args,
        TargetCanister::Ledger,
    )?;

    Ok(vec![msg])
}

fn new_tokens(tokens: u64, e8s: u64) -> AnyhowResult<Tokens> {
    Tokens::new(tokens, e8s)
        .map_err(|err| anyhow!(err))
        .context("Cannot create new Tokens")
}

fn parse_tokens(amount: &str) -> AnyhowResult<Tokens> {
    let parse = |s: &str| {
        s.parse::<u64>()
            .context("Failed to parse Tokens as unsigned integer")
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

fn tokens_amount_validator(tokens: &str) -> AnyhowResult<()> {
    parse_tokens(tokens).map(|_| ())
}

fn memo_validator(memo: &str) -> Result<(), String> {
    if memo.parse::<u64>().is_ok() {
        return Ok(());
    }
    Err("Memo must be an unsigned integer".to_string())
}
