use anyhow::{bail, ensure};
use candid::Encode;
use clap::Parser;
use ic_nns_governance::pb::v1::{
    manage_neuron::{disburse::Amount, Command, Disburse},
    ManageNeuron,
};
use icp_ledger::Tokens;
use icrc_ledger_types::icrc1::account::Account;

use crate::lib::parse_tokens;
use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ParsedNnsAccount, ParsedSubaccount, ROLE_NNS_GOVERNANCE,
};

use super::get_ids;

/// Signs a disbursal message to convert a dissolved neuron into ICP.
#[derive(Parser)]
pub struct DisburseOpts {
    /// The ID of the neuron to disburse.
    neuron_id: ParsedNeuron,

    /// The account to transfer the ICP to. If unset, defaults to the caller.
    #[clap(long, required_unless_present = "auth")]
    to: Option<ParsedNnsAccount>,

    /// The subaccount to transfer to.
    #[clap(long)]
    subaccount: Option<ParsedSubaccount>,

    /// The number of tokens, in decimal form, to disburse. If unset, fully consumes the neuron.
    #[clap(long, value_parser = parse_tokens)]
    amount: Option<Tokens>,

    #[clap(from_global)]
    ledger: bool,
}

pub fn exec(auth: &AuthInfo, opts: DisburseOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    ensure!(!opts.ledger, "Cannot use `--ledger` with this command. This version of Quill does not support disbursing neurons with a Ledger device.");
    let to = if let Some(sub) = opts.subaccount {
        match opts.to {
            Some(ParsedNnsAccount::Icrc1(acct)) => Some(ParsedNnsAccount::Icrc1(Account {
                subaccount: Some(sub.0 .0),
                ..acct
            })),
            Some(ParsedNnsAccount::Original(_)) => {
                bail!("Cannot specify both --subaccount and a legacy account ID")
            }
            None => Some(ParsedNnsAccount::Icrc1(Account {
                owner: get_ids(auth)?.0,
                subaccount: Some(sub.0 .0),
            })),
        }
    } else {
        opts.to
    };
    let args = ManageNeuron {
        command: Some(Command::Disburse(Disburse {
            amount: opts.amount.map(|amount| Amount {
                e8s: amount.get_e8s(),
            }),
            to_account: to.map(|to| to.into_identifier().into()),
        })),
        id: Some(opts.neuron_id.0),
        neuron_id_or_subaccount: None,
    };
    let message = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "manage_neuron",
        Encode!(&args)?,
    )?;
    Ok(vec![message])
}
