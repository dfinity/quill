use candid::Encode;
use clap::Parser;
use ic_nns_governance::pb::v1::{
    manage_neuron::{Command, Split},
    ManageNeuron,
};
use icp_ledger::Tokens;

use crate::lib::{
    governance_canister_id, parse_tokens,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ROLE_NNS_GOVERNANCE,
};

/// Signs a neuron management message to split a neuron in two.
#[derive(Parser)]
pub struct SplitOpts {
    /// The ID of the neuron to split.
    neuron_id: ParsedNeuron,
    /// The amount of the stake that should be split, in ICP.
    #[clap(long, value_parser = parse_tokens)]
    amount: Tokens,
}

pub fn exec(auth: &AuthInfo, opts: SplitOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let arg = ManageNeuron {
        id: Some(opts.neuron_id.0),
        command: Some(Command::Split(Split {
            amount_e8s: opts.amount.get_e8s(),
        })),
        neuron_id_or_subaccount: None,
    };
    let msg = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "manage_neuron",
        Encode!(&arg)?,
    )?;
    Ok(vec![msg])
}
