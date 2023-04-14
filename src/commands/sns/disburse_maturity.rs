use candid::Encode;
use clap::Parser;
use ic_sns_governance::pb::v1::{
    manage_neuron::{Command, DisburseMaturity},
    ManageNeuron,
};

use crate::{
    commands::get_account,
    lib::{
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, AuthInfo, ParsedAccount, ParsedSubaccount, ROLE_SNS_GOVERNANCE,
    },
};

use super::{governance_account, ParsedSnsNeuron, SnsCanisterIds};

/// Converts the maturity from a neuron into SNS utility tokens.
#[derive(Parser)]
pub struct DisburseMaturityOpts {
    /// The neuron ID to disburse maturity from.
    neuron_id: ParsedSnsNeuron,
    /// The account to transfer the SNS utility tokens to. If not provided, defaults to the caller.
    #[clap(long, required_unless_present = "auth")]
    to: Option<ParsedAccount>,
    /// The subaccount to transfer the SNS utility tokens to.
    #[clap(long)]
    subaccount: Option<ParsedSubaccount>,
    /// The percentage, as a number from 1 to 100, of the maturity to disburse.
    #[clap(long, value_parser = 1..=100, default_value_t = 100)]
    percentage: i64,
}

pub fn exec(
    auth: &AuthInfo,
    canister_ids: &SnsCanisterIds,
    opts: DisburseMaturityOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let account = get_account(Some(auth), opts.to, opts.subaccount)?;
    let args = ManageNeuron {
        subaccount: opts.neuron_id.0.id,
        command: Some(Command::DisburseMaturity(DisburseMaturity {
            percentage_to_disburse: opts.percentage as u32,
            to_account: Some(governance_account(account)),
        })),
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
