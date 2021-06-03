use crate::{
    commands::{request_status, sign::sign, transfer},
    lib::{
        environment::Environment,
        nns_types::account_identifier::{AccountIdentifier, Subaccount},
        nns_types::Memo,
        AnyhowResult, GOVERNANCE_CANISTER_ID,
    },
};
use candid::{CandidType, Encode};
use clap::Clap;
use ic_types::Principal;

#[derive(CandidType)]
pub struct ClaimOrRefreshNeuronFromAccount {
    pub memo: Memo,
    pub controller: Option<Principal>,
}

/// Creates a neuron with the specified amount of ICPs
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

pub async fn exec(env: &dyn Environment, opts: StakeOpts) -> AnyhowResult<String> {
    let (controller, _) = crate::commands::public::get_ids(env)?;
    let nonce = convert_name_to_nonce(&opts.name);
    let gov_subaccount = get_neuron_subaccount(&controller, nonce);
    let account = AccountIdentifier::new(
        Principal::from_text(GOVERNANCE_CANISTER_ID)?,
        Some(gov_subaccount),
    );
    let transfer_message = transfer::exec(
        env,
        transfer::TransferOpts {
            to: account.to_hex(),
            amount: Some(opts.amount),
            fee: opts.fee,
            memo: Some(nonce.to_string()),
            ..Default::default()
        },
    )
    .await?;
    let args = Encode!(&ClaimOrRefreshNeuronFromAccount {
        memo: Memo(nonce),
        controller: Some(controller),
    })?;

    let method_name = "claim_or_refresh_neuron_from_account".to_string();
    let canister_id = Principal::from_text(GOVERNANCE_CANISTER_ID)?;
    let msg_with_req_id = sign(env, canister_id.clone(), &&method_name, args).await?;
    let request_id = msg_with_req_id
        .request_id
        .expect("No request id for transfer call found");
    let req_status_signed_msg = request_status::sign(env, request_id, canister_id).await?;

    // Generate a JSON list of signed messages.
    let mut out = String::new();
    out.push_str("{ \"transfer\": ");
    out.push_str(&transfer_message);
    out.push_str(", \"claim\": ");
    out.push_str("{ \"ingress\": ");
    out.push_str(&msg_with_req_id.buffer);
    out.push_str(", \"request_status\": ");
    out.push_str(&req_status_signed_msg);
    out.push_str("}");
    out.push_str("}");

    Ok(out)
}

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
    // Convert to bytes before checking the length to restrict it ot ASCII only
    if name.as_bytes().len() > 8 {
        return Err("The neuron name must be 8 character or less".to_string());
    }
    Ok(())
}
