use candid::Encode;
use clap::Parser;
use ic_sns_governance::pb::v1::{
    manage_neuron::{disburse::Amount, Command, Disburse},
    ManageNeuron,
};
use icp_ledger::Tokens;

use crate::{
    commands::{get_account, transfer::parse_tokens},
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, AuthInfo, ParsedAccount, ParsedSubaccount, ROLE_SNS_GOVERNANCE,
    },
};

use super::{governance_account, ParsedSnsNeuron, SnsCanisterIds};

/// Converts a fully-dissolved neuron into SNS utility tokens.
#[derive(Parser)]
pub struct DisburseOpts {
    /// The neuron to disburse.
    neuron_id: ParsedSnsNeuron,
    /// The account to transfer the SNS utility tokens to. If unset, defaults to the caller.
    #[clap(long, required_unless_present = "auth")]
    to: Option<ParsedAccount>,
    /// The subaccount to transfer the SNS utility tokens to.
    #[clap(long)]
    subaccount: Option<ParsedSubaccount>,
    /// The number of tokens, in decimal form, to disburse. If unset, fully consumes the neuron.
    #[clap(long, value_parser = parse_tokens)]
    amount: Option<Tokens>,
}

pub fn exec(
    auth: &AuthInfo,
    canister_ids: &SnsCanisterIds,
    opts: DisburseOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let account = get_account(Some(auth), opts.to, opts.subaccount)?;
    let args = ManageNeuron {
        command: Some(Command::Disburse(Disburse {
            amount: opts.amount.map(|amount| Amount {
                e8s: amount.get_e8s(),
            }),
            to_account: Some(governance_account(account)),
        })),
        subaccount: opts.neuron_id.0.id,
    };
    let message = sign_ingress_with_request_status_query(
        auth,
        canister_ids.governance_canister_id,
        ROLE_SNS_GOVERNANCE,
        "manage_neuron",
        Encode!(&args)?,
    )?;
    Ok(vec![message])
}
