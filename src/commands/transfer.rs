use crate::lib::{
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, TargetCanister,
};
use crate::SnsCanisterIds;
use anyhow::{anyhow, bail, Context, Error};
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ledger_canister::{AccountIdentifier, Memo, Tokens, TransferArgs, DEFAULT_TRANSFER_FEE};

/// Signs a ledger transfer update call.
#[derive(Default, Parser)]
pub struct TransferOpts {
    /// The AccountIdentifier of the destination account. For example: d5662fbce449fbd4adb4b9aff6c59035bd93e7c2eff5010a446ebc3dd81007f8
    pub to: String,

    /// Amount of governance tokens to transfer (with up to 8 decimal digits after decimal point)
    #[clap(long, validator(tokens_amount_validator))]
    pub amount: String,

    /// An arbitrary number associated with a transaction. The default is 0
    #[clap(long, validator(memo_validator))]
    pub memo: Option<String>,

    /// The amount that the caller pays for the transaction, default is 10_000 e8s. Specify this amount
    /// when using an SNS that sets its own transaction fee
    #[clap(long, validator(tokens_amount_validator))]
    pub fee: Option<String>,
}

pub fn exec(
    private_key_pem: &str,
    sns_canister_ids: &SnsCanisterIds,
    opts: TransferOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let amount = parse_tokens(&opts.amount).context("Cannot parse amount")?;
    let fee = opts.fee.map_or(Ok(DEFAULT_TRANSFER_FEE), |fee| {
        parse_tokens(&fee).context("Cannot parse fee")
    })?;
    let memo = Memo(
        opts.memo
            .unwrap_or_else(|| "0".to_string())
            .parse::<u64>()
            .context("Failed to parse memo as unsigned integer")?,
    );
    let ledger_canister_id = PrincipalId::from(sns_canister_ids.ledger_canister_id).0;
    let to_account_identifier = AccountIdentifier::from_hex(&opts.to).map_err(Error::msg)?;

    let args = Encode!(&TransferArgs {
        memo,
        amount,
        fee,
        from_subaccount: None,
        to: to_account_identifier.to_address(),
        created_at_time: None,
    })?;

    let msg = sign_ingress_with_request_status_query(
        private_key_pem,
        "transfer",
        args,
        TargetCanister::Ledger(ledger_canister_id),
    )?;

    Ok(vec![msg])
}

fn new_tokens(tokens: u64, e8s: u64) -> AnyhowResult<Tokens> {
    Tokens::new(tokens, e8s)
        .map_err(|err| anyhow!(err))
        .context("Cannot create new Tokens")
}

fn parse_tokens(amount: &str) -> AnyhowResult<Tokens> {
    let parse_u64 = |s: &str| {
        s.parse::<u64>()
            .context("Failed to parse Tokens as unsigned integer")
    };
    match &amount.split('.').collect::<Vec<_>>().as_slice() {
        [tokens] => new_tokens(parse_u64(tokens)?, 0),
        [tokens, e8s] => {
            let mut e8s = e8s.to_string();
            // Pad e8s with zeros on the right so that its length is 8.
            while e8s.len() < 8 {
                e8s.push('0');
            }
            let e8s = &e8s[..8];
            new_tokens(parse_u64(tokens)?, parse_u64(e8s)?)
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
