use crate::{
    commands::{
        send::Memo,
        transfer::{self, parse_tokens},
    },
    lib::{
        governance_canister_id,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, AuthInfo, ParsedNnsAccount, ParsedSubaccount, ROLE_NNS_GOVERNANCE,
    },
};
use anyhow::{anyhow, ensure};
use candid::{CandidType, Encode, Principal};
use clap::Parser;
use ic_nns_constants::GOVERNANCE_CANISTER_ID;
use icp_ledger::{AccountIdentifier, Subaccount, Tokens};
use sha2::{Digest, Sha256};

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
        validator(neuron_name_validator),
        conflicts_with = "nonce",
        required_unless_present = "nonce"
    )]
    name: Option<String>,

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
    let controller = crate::lib::get_principal(auth)?;
    let nonce = match (&opts.nonce, &opts.name) {
        (Some(nonce), _) => *nonce,
        (_, Some(name)) => convert_name_to_nonce(name),
        _ => return Err(anyhow!("Either a nonce or a name should be specified")),
    };
    let gov_subaccount = get_neuron_subaccount(&controller, nonce);
    let account = AccountIdentifier::new(GOVERNANCE_CANISTER_ID.get(), Some(gov_subaccount));
    let mut messages = if !opts.already_transferred {
        transfer::exec(
            auth,
            transfer::TransferOpts {
                to: ParsedNnsAccount::Original(account),
                amount: opts.amount.unwrap(),
                fee: opts.fee,
                memo: Some(nonce),
                from_subaccount: opts.from_subaccount,
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
fn get_neuron_subaccount(controller: &Principal, nonce: u64) -> Subaccount {
    let mut data = Sha256::new();
    data.update([0x0c]);
    data.update(b"neuron-stake");
    data.update(controller.as_slice());
    data.update(nonce.to_be_bytes());
    Subaccount(data.finalize().into())
}

fn convert_name_to_nonce(name: &str) -> u64 {
    let mut bytes = std::collections::VecDeque::from(name.as_bytes().to_vec());
    while bytes.len() < 8 {
        bytes.push_front(0);
    }
    let mut arr: [u8; 8] = [0; 8];
    arr.copy_from_slice(&bytes.into_iter().collect::<Vec<_>>());
    u64::from_be_bytes(arr)
}

fn neuron_name_validator(name: &str) -> Result<(), String> {
    if name.len() > 8 || name.chars().any(|c| !c.is_ascii()) {
        return Err("The neuron name must be 8 character or less".to_string());
    }
    Ok(())
}
