use crate::commands::transfer;
use crate::lib::signing::{sign_ingress_with_request_status_query, IngressWithRequestId};
use crate::lib::TargetCanister;
use crate::{AnyhowResult, CanisterIds};
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_sns_governance::pb::v1::manage_neuron;
use ic_sns_governance::pb::v1::manage_neuron::claim_or_refresh::{By, MemoAndController};
use ic_sns_governance::pb::v1::manage_neuron::ClaimOrRefresh;
use ic_sns_governance::pb::v1::ManageNeuron;
use ic_types::Principal;
use ledger_canister::{AccountIdentifier, Subaccount};

/// Signs staking transfer to the subaccount of a neuron and signs a
/// ManageNeuron::ClaimOrRefresh message to claim the same neuron.
#[derive(Parser)]
pub struct NeuronStakeOpts {
    /// Amount of tokens to be staked on the newly created neuron.
    #[clap(long)]
    amount: Option<String>,

    /// The memo used to calculate the neuron's subaccount.
    #[clap(long)]
    memo: u64,

    /// Transaction fee, default is 10000 e8s.
    #[clap(long)]
    fee: Option<String>,
}

pub fn exec(
    pem: &str,
    canister_ids: &CanisterIds,
    opts: NeuronStakeOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let (controller, _) = crate::commands::public::get_ids(&Some(pem.to_string()))?;
    let neuron_subaccount = get_neuron_subaccount(&controller, opts.memo);

    let governance_canister_id = PrincipalId::from(canister_ids.governance_canister_id);
    let account = AccountIdentifier::new(governance_canister_id, Some(neuron_subaccount));

    // Sign a transfer message that will transfer tokens from the principal's account
    // to a subaccount of the governance canister.
    let mut messages = match &opts.amount {
        Some(amount) => transfer::exec(
            pem,
            canister_ids,
            transfer::TransferOpts {
                to: account.to_hex(),
                amount: amount.clone(),
                fee: opts.fee,
                memo: Some(opts.memo.to_string()),
            },
        )?,
        _ => Vec::new(),
    };

    // Sign a message claiming the neuron of the calculated subaccount.
    let args = Encode!(&ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(manage_neuron::Command::ClaimOrRefresh(ClaimOrRefresh {
            by: Some(By::MemoAndController(MemoAndController {
                memo: opts.memo,
                controller: Some(PrincipalId(controller)),
            }))
        }))
    })?;

    messages.push(sign_ingress_with_request_status_query(
        pem,
        governance_canister_id.0,
        "manage_neuron",
        args,
        TargetCanister::Governance,
    )?);

    Ok(messages)
}

// This function _must_ correspond to how the governance canister computes the
// subaccount.
pub fn get_neuron_subaccount(controller: &Principal, nonce: u64) -> Subaccount {
    use openssl::sha::Sha256;
    let mut data = Sha256::new();
    data.update(&[0x0c]);
    data.update(b"neuron-stake");
    data.update(controller.as_slice());
    data.update(&nonce.to_be_bytes());
    Subaccount(data.finish())
}
