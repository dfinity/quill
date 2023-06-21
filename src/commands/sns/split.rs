use candid::Encode;
use clap::Parser;
use ic_sns_governance::pb::v1::{
    manage_neuron::{Command, Split},
    ManageNeuron,
};
use icp_ledger::Tokens;

use crate::lib::{
    neuron_name_to_nonce, parse_tokens,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_SNS_GOVERNANCE,
};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Splits a neuron into two neurons.
#[derive(Parser)]
#[clap(alias = "split-neuron")]
pub struct SplitOpts {
    /// The neuron to split.
    neuron_id: ParsedSnsNeuron,
    /// A number to identify this neuron. Must be unique among your neurons for this SNS.
    #[clap(long, alias = "memo")]
    nonce: Option<u64>,
    /// A name to identify this neuron. Must be unique among your neurons for this SNS.
    #[clap(
        long,
        conflicts_with = "nonce",
        required_unless_present = "nonce",
        value_parser = neuron_name_to_nonce,
    )]
    name: Option<u64>,
    /// The number of tokens, in decimal form, to split off.
    #[clap(long, value_parser = parse_tokens)]
    amount: Tokens,
}

pub fn exec(
    auth: &AuthInfo,
    canister_ids: &SnsCanisterIds,
    opts: SplitOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let args = ManageNeuron {
        subaccount: opts.neuron_id.0.id,
        command: Some(Command::Split(Split {
            amount_e8s: opts.amount.get_e8s(),
            memo: opts.name.unwrap_or_else(|| opts.nonce.unwrap()),
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
