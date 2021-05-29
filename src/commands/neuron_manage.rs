use crate::{
    commands::{request_status_sign, sign},
    lib::{environment::Environment, get_idl_string, DfxResult, GOVERNANCE_CANISTER_ID},
};
use candid::{CandidType, Encode};
use clap::Clap;
use ic_types::Principal;

#[derive(CandidType)]
pub struct IncreaseDissolveDelay {
    pub additional_dissolve_delay_seconds: u32,
}

#[derive(CandidType)]
pub struct NeuronId {
    pub id: u64,
}

#[derive(CandidType)]
pub struct AddHotKey {
    pub new_hot_key: Option<Principal>,
}

#[derive(CandidType)]
pub enum Operation {
    AddHotKey(AddHotKey),
    IncreaseDissolveDelay(IncreaseDissolveDelay),
}

#[derive(CandidType)]
pub struct Configure {
    pub operation: Option<Operation>,
}

#[derive(CandidType)]
pub enum Command {
    Configure(Configure),
}

#[derive(CandidType)]
struct ManageNeuron {
    id: Option<NeuronId>,
    command: Option<Command>,
}

/// Signs a neuron configuration
#[derive(Clap)]
pub struct ManageOpts {
    /// Neuron Id
    #[clap(long)]
    neuron_id: u64,

    /// Amount of dissolve seconds to add.
    #[clap(short, long)]
    additional_dissolve_delay_seconds: u32,
}

pub async fn exec(env: &dyn Environment, opts: ManageOpts) -> DfxResult<String> {
    let args = Encode!(&ManageNeuron {
        id: Some(NeuronId { id: opts.neuron_id }),
        command: Some(Command::Configure(Configure {
            operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                additional_dissolve_delay_seconds: opts.additional_dissolve_delay_seconds
            }))
        }))
    })?;

    let msg = generate(env, args).await?;
    let mut out = String::new();
    out.push_str("[");
    out.push_str(&msg);
    out.push_str("]");

    Ok(out)
}

pub async fn generate(env: &dyn Environment, args: Vec<u8>) -> DfxResult<String> {
    let method_name = "manage_neuron".to_string();
    let argument = Some(get_idl_string(
        &args,
        GOVERNANCE_CANISTER_ID,
        &method_name,
        "args",
        "raw",
    )?);
    let canister_id = GOVERNANCE_CANISTER_ID.to_string();
    let opts = sign::SignOpts {
        canister_id: canister_id.clone(),
        method_name,
        query: false,
        update: true,
        argument,
        r#type: Some("raw".to_string()),
    };
    let msg_with_req_id = sign::exec(env, opts).await?;
    let request_id: String = msg_with_req_id
        .request_id
        .expect("No request id for transfer call found")
        .into();
    let req_status_signed_msg = request_status_sign::exec(
        env,
        request_status_sign::RequestStatusSignOpts {
            request_id: format!("0x{}", request_id),
            canister_id,
        },
    )
    .await?;

    let mut out = String::new();
    out.push_str("{ \"ingress\": ");
    out.push_str(&msg_with_req_id.buffer);
    out.push_str(", \"request_status\": ");
    out.push_str(&req_status_signed_msg);
    out.push_str("}");

    Ok(out)
}
