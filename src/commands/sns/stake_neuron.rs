use crate::commands::transfer::parse_tokens;
use crate::lib::now_nanos;
use crate::lib::signing::{
    sign_ingress_with_request_status_query, sign_staking_ingress_with_request_status_query,
};
use crate::{
    lib::{
        signing::IngressWithRequestId, AuthInfo, ParsedSubaccount, ROLE_ICRC1_LEDGER,
        ROLE_SNS_GOVERNANCE,
    },
    AnyhowResult,
};
use candid::Encode;
use clap::Parser;
use ic_nervous_system_common::ledger;
use ic_sns_governance::pb::v1::{
    manage_neuron,
    manage_neuron::{
        claim_or_refresh::{By, MemoAndController},
        ClaimOrRefresh,
    },
    ManageNeuron,
};
use icp_ledger::Tokens;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{Memo, TransferArg};

use super::SnsCanisterIds;

/// Signs messages needed to stake governance tokens for a neuron. First, stake-neuron will sign
/// a ledger transfer to a subaccount of the Governance canister calculated from the
/// provided private key and memo. Second, stake-neuron will sign a ManageNeuron message for
/// Governance to claim the neuron for the principal derived from the provided private key.
#[derive(Parser)]
pub struct StakeNeuronOpts {
    /// The amount of tokens to be transferred to the Governance canister's ledger subaccount
    /// (the neuron's AccountId) from the AccountId derived from the provided private key. This is
    /// known as a staking transfer. These funds will be returned when disbursing the neuron.
    #[clap(long, value_parser = parse_tokens, required_unless_present = "claim-only")]
    amount: Option<Tokens>,

    /// The subaccount to make the transfer from. Only necessary if `--amount` is specified.
    #[clap(long, requires = "amount")]
    from_subaccount: Option<ParsedSubaccount>,

    /// An arbitrary number used in calculating the neuron's subaccount. The memo must be unique among
    /// the neurons claimed for a single PrincipalId. More information on ledger accounts and
    /// subaccounts can be found here: https://smartcontracts.org/docs/integration/ledger-quick-start.html#_ledger_canister_overview
    #[clap(long)]
    memo: u64,

    /// The amount that the caller pays for the transaction, default is 0.0001 tokens. Specify this amount
    /// when using an SNS that sets its own transaction fee
    #[clap(long, value_parser = parse_tokens)]
    fee: Option<Tokens>,

    /// If this flag is set, then no transfer will be made, and only the neuron claim message will be generated.
    /// This is useful if there was an error previously submitting the notification which you have since rectified,
    /// or if you have made the transfer with another tool.
    #[clap(long, conflicts_with = "amount", conflicts_with = "from-subaccount")]
    claim_only: bool,
}

pub fn exec(
    auth: &AuthInfo,
    sns_canister_ids: &SnsCanisterIds,
    opts: StakeNeuronOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let controller = crate::lib::get_principal(auth)?;
    let neuron_subaccount = ledger::compute_neuron_staking_subaccount(controller.into(), opts.memo);

    let governance_canister_id = sns_canister_ids.governance_canister_id;

    let mut messages = Vec::new();

    // If amount is provided, sign a transfer message that will transfer tokens from the principal's
    // account on the ledger to a subaccount of the governance canister.
    if let Some(amount) = opts.amount {
        let args = TransferArg {
            memo: Some(Memo::from(opts.memo)),
            amount: amount.get_e8s().into(),
            fee: opts.fee.map(|fee| fee.get_e8s().into()),
            from_subaccount: opts.from_subaccount.map(|x| x.0 .0),
            created_at_time: Some(now_nanos()),
            to: Account {
                owner: governance_canister_id,
                subaccount: Some(neuron_subaccount.0),
            },
        };

        let msg = sign_ingress_with_request_status_query(
            auth,
            sns_canister_ids.ledger_canister_id,
            ROLE_ICRC1_LEDGER,
            "icrc1_transfer",
            Encode!(&args)?,
        )?;
        messages.push(msg);
    }

    // Sign a message claiming the neuron with funds staked to the previously calculated subaccount.
    let args = Encode!(&ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(manage_neuron::Command::ClaimOrRefresh(ClaimOrRefresh {
            by: Some(By::MemoAndController(MemoAndController {
                memo: opts.memo,
                controller: Some(controller.into()),
            }))
        }))
    })?;

    messages.push(sign_staking_ingress_with_request_status_query(
        auth,
        governance_canister_id,
        ROLE_SNS_GOVERNANCE,
        "manage_neuron",
        args,
    )?);

    Ok(messages)
}
