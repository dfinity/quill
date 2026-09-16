use anyhow::Context;
use askama::Template;
use candid::Decode;
use ic_sns_governance::pb::v1::{manage_neuron_response::Command, ManageNeuronResponse};

use crate::lib::{format::filters, AnyhowResult};

pub fn display_manage_neuron(blob: &[u8]) -> AnyhowResult<String> {
    use Command::*;
    let response = Decode!(blob, ManageNeuronResponse)?;
    let command = response.command.context("command was null")?;
    #[derive(Template)]
    #[template(path = "sns/manage_neuron.txt")]
    struct ManageNeuron {
        command: Command,
    }
    let fmt = ManageNeuron { command }.render()?;
    Ok(fmt)
}
