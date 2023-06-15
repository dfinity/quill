use crate::{
    commands::{send::Memo, transfer},
    lib::{
        governance_canister_id, neuron_name_to_nonce, parse_tokens,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, AuthInfo, ParsedNnsAccount, ParsedSubaccount, ROLE_NNS_GOVERNANCE,
    },
};
use anyhow::ensure;
use candid::{CandidType, Encode, Principal};
use clap::Parser;
use ic_nns_constants::GOVERNANCE_CANISTER_ID;
use icp_ledger::Tokens;
use icrc_ledger_types::icrc1::account::Account;

#[derive(CandidType)]
pub struct ClaimOrRefreshNeuronFromAccount {
    pub memo: Memo,
    pub controller: Option<Principal>,
}

/// Signs topping up of a neuron (new or existing).
#[derive(Parser)]
pub struct StakeOpts {
    /// ICPs to be staked on the newly created neuron.
    #[clap(long, value_parser = parse_tokens, conflicts_with = "already-transferred", required_unless_present = "already-transferred")]
    amount: Option<Tokens>,

    /// Skips signing the transfer of ICP, signing only the staking request.
    #[clap(long)]
    already_transferred: bool,

    /// The name of the neuron (up to 8 ASCII characters).
    #[clap(
        long,
        value_parser = neuron_name_to_nonce,
        conflicts_with = "nonce",
        required_unless_present = "nonce"
    )]
    name: Option<u64>,

    /// The nonce of the neuron.
    #[clap(long)]
    nonce: Option<u64>,

    /// Transaction fee, default is 0.0001 ICP.
    #[clap(long, value_parser = parse_tokens)]
    fee: Option<Tokens>,

    /// The subaccount to transfer from.
    #[clap(long)]
    from_subaccount: Option<ParsedSubaccount>,

    #[clap(from_global)]
    ledger: bool,
}

pub fn exec(auth: &AuthInfo, opts: StakeOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    ensure!(
        !opts.ledger,
        "Cannot use `--ledger` with this command. This version of Quill does not support staking new neurons with a Ledger device"
    );
    let (controller, _) = crate::commands::public::get_ids(auth)?;
    let nonce = opts.name.unwrap_or_else(|| opts.nonce.unwrap());
    let gov_subaccount = get_neuron_subaccount(&controller, nonce);
    let account = Account {
        owner: GOVERNANCE_CANISTER_ID.into(),
        subaccount: Some(gov_subaccount),
    };
    let mut messages = if !opts.already_transferred {
        transfer::exec(
            auth,
            transfer::TransferOpts {
                to: ParsedNnsAccount::Icrc1(account),
                amount: opts.amount.unwrap(),
                fee: opts.fee,
                memo: Some(nonce),
                from_subaccount: opts.from_subaccount,
                to_subaccount: None,
            },
        )?
    } else {
        Vec::new()
    };
    let args = Encode!(&ClaimOrRefreshNeuronFromAccount {
        memo: Memo(nonce),
        controller: Some(controller),
    })?;

    messages.push(sign_ingress_with_request_status_query(
        auth,
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "claim_or_refresh_neuron_from_account",
        args,
    )?);

    Ok(messages)
}

// This function _must_ correspond to how the governance canister computes the
// subaccount.
fn get_neuron_subaccount(controller: &Principal, nonce: u64) -> [u8; 32] {
    use openssl::sha::Sha256;
    let mut data = Sha256::new();
    data.update(&[0x0c]);
    data.update(b"neuron-stake");
    data.update(controller.as_slice());
    data.update(&nonce.to_be_bytes());
    data.finish()
}
