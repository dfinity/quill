use crate::{
    commands::{send::Memo, sign::sign_ingress_with_request_status_query, transfer},
    lib::{governance_canister_id, sign::signed_message::NeuronStakeMessage, AnyhowResult},
};
use candid::{CandidType, Encode};
use clap::Clap;
use ic_nns_constants::GOVERNANCE_CANISTER_ID;
use ic_types::Principal;
use ledger_canister::{AccountIdentifier, Subaccount};

#[derive(CandidType)]
pub struct ClaimOrRefreshNeuronFromAccount {
    pub memo: Memo,
    pub controller: Option<Principal>,
}

/// Signs topping up of a neuron (new or existing).
#[derive(Clap)]
pub struct StakeOpts {
    /// ICPs to be staked on the newly created neuron.
    #[clap(long)]
    amount: String,

    /// The name of the neuron (up to 8 ASCII characters).
    #[clap(long, validator(neuron_name_validator))]
    name: String,

    /// Transaction fee, default is 10000 e8s.
    #[clap(long)]
    fee: Option<String>,
}

pub async fn exec(pem: &Option<String>, opts: StakeOpts) -> AnyhowResult<NeuronStakeMessage> {
    let (controller, _) = crate::commands::public::get_ids(pem)?;
    let nonce = convert_name_to_nonce(&opts.name);
    let gov_subaccount = get_neuron_subaccount(&controller, nonce);
    let account = AccountIdentifier::new(GOVERNANCE_CANISTER_ID.get(), Some(gov_subaccount));
    let transfer_message = transfer::exec(
        pem,
        transfer::TransferOpts {
            to: account.to_hex(),
            amount: opts.amount,
            fee: opts.fee,
            memo: Some(nonce.to_string()),
        },
    )
    .await?;
    let args = Encode!(&ClaimOrRefreshNeuronFromAccount {
        memo: Memo(nonce),
        controller: Some(controller),
    })?;

    let claim_message = sign_ingress_with_request_status_query(
        pem,
        governance_canister_id(),
        "claim_or_refresh_neuron_from_account",
        args,
    )
    .await?;

    // Generate a JSON list of signed messages.
    let message = NeuronStakeMessage {
        transfer: transfer_message,
        claim: claim_message,
    };
    Ok(message)
}

// This function _must_ correspond to how the governance canister computes the subaccount.
fn get_neuron_subaccount(controller: &Principal, nonce: u64) -> Subaccount {
    use openssl::sha::Sha256;
    let mut data = Sha256::new();
    data.update(&[0x0c]);
    data.update(b"neuron-stake");
    data.update(&controller.as_slice());
    data.update(&nonce.to_be_bytes());
    Subaccount(data.finish())
}

fn convert_name_to_nonce(name: &str) -> u64 {
    let mut bytes = std::collections::VecDeque::from(name.as_bytes().to_vec());
    while bytes.len() < 8 {
        bytes.push_front(0)
    }
    let mut arr: [u8; 8] = [0; 8];
    arr.copy_from_slice(&bytes.into_iter().collect::<Vec<_>>());
    u64::from_be_bytes(arr)
}

fn neuron_name_validator(name: &str) -> Result<(), String> {
    // Convert to bytes before checking the length to restrict it to ASCII only
    if name.as_bytes().len() > 8 {
        return Err("The neuron name must be 8 character or less".to_string());
    }
    Ok(())
}
