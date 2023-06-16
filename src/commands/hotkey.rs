use anyhow::ensure;
use candid::{Encode, Principal};
use clap::{ArgGroup, Parser};
use ic_nns_governance::pb::v1::{
    manage_neuron::{configure::Operation, AddHotKey, Command, Configure, RemoveHotKey},
    ManageNeuron,
};

use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ROLE_NNS_GOVERNANCE,
};

/// Signs a neuron configuration message to add or remove a hotkey.
#[derive(Parser)]
#[clap(group(ArgGroup::new("operation").required(true)), alias = "hot-key")]
pub struct HotkeyOpts {
    /// The ID of the neuron to configure.
    neuron_id: ParsedNeuron,

    /// Add the specified principal as a hotkey.
    #[clap(long, group = "operation", value_name = "PRINCIPAL")]
    add: Option<Principal>,

    /// Remove the specified principal as a hotkey.
    #[clap(long, group = "operation", value_name = "PRINCIPAL")]
    remove: Option<Principal>,

    #[clap(from_global)]
    ledger: bool,
}

pub fn exec(auth: &AuthInfo, opts: HotkeyOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    ensure!(!opts.ledger, "Cannot use `--ledger` with this command. This version of Quill does not support modifying hotkeys with a Ledger device.");
    let command = if let Some(add) = opts.add {
        Command::Configure(Configure {
            operation: Some(Operation::AddHotKey(AddHotKey {
                new_hot_key: Some(add.into()),
            })),
        })
    } else if let Some(remove) = opts.remove {
        Command::Configure(Configure {
            operation: Some(Operation::RemoveHotKey(RemoveHotKey {
                hot_key_to_remove: Some(remove.into()),
            })),
        })
    } else {
        unreachable!()
    };
    let arg = Encode!(&ManageNeuron {
        command: Some(command),
        id: Some(opts.neuron_id.0),
        neuron_id_or_subaccount: None,
    })?;
    let msg = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "manage_neuron",
        arg,
    )?;
    Ok(vec![msg])
}
