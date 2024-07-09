use anyhow::Context;
use candid::Decode;
use ic_sns_governance::pb::v1::{
    manage_neuron_response::Command, GovernanceError, ManageNeuronResponse,
};

use crate::lib::{e8s_to_tokens, AnyhowResult};

pub fn display_manage_neuron(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, ManageNeuronResponse)?;
    let command = response.command.context("command was null")?;
    let fmt = match command {
        Command::Error(error) => display_governance_error(error),
        Command::Configure(_) => "Neuron successfully configured".to_string(),
        Command::RegisterVote(_) => "Successfully voted".to_string(),
        Command::Follow(_) => "Successfully set following relationship".to_string(),
        Command::AddNeuronPermission(_) => "Successfully added neuron permissions".to_string(),
        Command::RemoveNeuronPermission(_) => "Successfully removed neuron permissions".to_string(),
        Command::Disburse(c) => format!(
            "Successfully disbursed ICP at block index {}",
            c.transfer_block_height
        ),
        Command::ClaimOrRefresh(c) => {
            if let Some(id) = c.refreshed_neuron_id {
                format!("Successfully updated the stake of neuron {id}")
            } else {
                "Successfully updated the stake of unknown neuron".to_string()
            }
        }
        Command::DisburseMaturity(c) => format!(
            "Successfully disbursed {} maturity",
            c.amount_deducted_e8s()
        ),
        Command::MakeProposal(c) => {
            if let Some(id) = c.proposal_id {
                format!("Successfully created new proposal with ID {id}", id = id.id)
            } else {
                "Successfully created new proposal with unknown ID".to_string()
            }
        }
        Command::MergeMaturity(c) => format!(
            "Successfully merged {merged} maturity (total stake now {total})",
            merged = e8s_to_tokens(c.merged_maturity_e8s.into()),
            total = e8s_to_tokens(c.new_stake_e8s.into())
        ),
        Command::Split(c) => {
            if let Some(id) = c.created_neuron_id {
                format!("Neuron successfully split off to new neuron {id}")
            } else {
                "Neuron successfully split off to unknown new neuron".to_string()
            }
        }
        Command::StakeMaturity(c) => format!(
            "Successfully staked maturity ({staked} staked maturity total, {remaining} unstaked)",
            staked = e8s_to_tokens(c.staked_maturity_e8s.into()),
            remaining = e8s_to_tokens(c.maturity_e8s.into())
        ),
    };
    Ok(fmt)
}

pub fn display_governance_error(err: GovernanceError) -> String {
    format!("SNS governance error: {}", err.error_message)
}
