use crate::lib::environment::Environment;
use crate::lib::sign::sign_transport::SignReplicaV2Transport;
use crate::lib::sign::sign_transport::SignedMessageWithRequestId;
use crate::lib::DfxResult;
use anyhow::anyhow;
use clap::Clap;
use ic_agent::{AgentError, RequestId};
use ic_types::Principal;
use tokio::runtime::Runtime;

mod neuron_manage;
mod neuron_stake;
mod public;
mod request_status_submit;
mod send;
mod sign;
mod transfer;

pub use public::get_ids;

#[derive(Clap)]
pub enum Command {
    /// Prints the principal id and the accound id.
    PublicIds,
    Send(send::SendOpts),
    Sign(sign::SignOpts),
    Transfer(transfer::TransferOpts),
    NeuronStake(neuron_stake::StakeOpts),
    NeuronManage(neuron_manage::ManageOpts),
}

pub fn exec(env: &dyn Environment, cmd: Command) -> DfxResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::PublicIds => public::exec(env),
        Command::Transfer(v) => runtime.block_on(async {
            transfer::exec(env, v).await.and_then(|out| {
                println!("{}", out);
                Ok(())
            })
        }),
        Command::Sign(v) => runtime.block_on(async {
            sign::exec(env, v).await.and_then(|out| {
                println!("{}", out.buffer);
                Ok(())
            })
        }),
        Command::NeuronStake(v) => runtime.block_on(async {
            neuron_stake::exec(env, v).await.and_then(|out| {
                println!("{}", out);
                Ok(())
            })
        }),
        Command::NeuronManage(v) => runtime.block_on(async {
            neuron_manage::exec(env, v).await.and_then(|out| {
                println!("{}", out);
                Ok(())
            })
        }),
        Command::Send(v) => runtime.block_on(async { send::exec(env, v).await }),
    }
}

pub async fn request_status_sign(
    env: &dyn Environment,
    request_id: RequestId,
    canister_id: Principal,
) -> DfxResult<String> {
    let mut agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let data = SignedMessageWithRequestId::new();
    data.write().unwrap().request_id = Some(request_id);
    let transport = SignReplicaV2Transport { data: data.clone() };
    agent.set_transport(transport);
    match agent.request_status_raw(&request_id, canister_id).await {
        Err(AgentError::MissingReplicaTransport()) => {
            return Ok(data.read().unwrap().buffer.clone());
        }
        val => panic!("Unexpected output from the signing agent: {:?}", val),
    }
}
