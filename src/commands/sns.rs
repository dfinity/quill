use std::{
    fmt::{self, Display, Formatter},
    fs,
    path::PathBuf,
    str::FromStr,
};

use anyhow::{ensure, Context};
use candid::{Deserialize, Principal};
use clap::{Parser, Subcommand};
use ic_sns_governance::pb::v1::Account as GovAccount;
use ic_sns_governance::pb::v1::{NeuronId, Subaccount};
use icrc_ledger_types::icrc1::account::Account;
use serde::Serialize;

use crate::lib::{AnyhowResult, AuthInfo};

use super::print_vec;

mod balance;
mod configure_dissolve_delay;
mod disburse;
mod disburse_maturity;
mod follow_neuron;
mod get_sale_participation;
mod get_swap_refund;
mod list_deployed_snses;
mod make_proposal;
mod make_upgrade_canister_proposal;
mod make_commit_proposed_batch_proposal;
mod neuron_id;
mod neuron_permission;
mod new_sale_ticket;
mod pay;
mod register_vote;
mod split_neuron;
mod stake_maturity;
mod stake_neuron;
mod status;
mod transfer;

/// Commands for interacting with a Service Nervous System's Ledger & Governance canisters.
///
/// Most commands require a JSON file containing a JSON map of canister names to canister IDs.
///
/// For example,
/// {
///   "governance_canister_id": "rrkah-fqaaa-aaaaa-aaaaq-cai",
///   "ledger_canister_id": "ryjl3-tyaaa-aaaaa-aaaba-cai",
///   "root_canister_id": "r7inp-6aaaa-aaaaa-aaabq-cai",
///   "swap_canister_id": "rkp4c-7iaaa-aaaaa-aaaca-cai"
/// }
#[derive(Parser)]
pub struct SnsOpts {
    /// Path to a SNS canister JSON file (see `quill sns help`)
    #[arg(long, global = true, help_heading = "COMMON")]
    canister_ids_file: Option<PathBuf>,
    #[command(subcommand)]
    subcommand: SnsCommand,
    #[arg(from_global)]
    ledger: bool,
}

#[derive(Subcommand)]
pub enum SnsCommand {
    Balance(balance::BalanceOpts),
    ConfigureDissolveDelay(configure_dissolve_delay::ConfigureDissolveDelayOpts),
    Disburse(disburse::DisburseOpts),
    DisburseMaturity(disburse_maturity::DisburseMaturityOpts),
    FollowNeuron(follow_neuron::FollowNeuronOpts),
    GetSwapRefund(get_swap_refund::GetSwapRefundOpts),
    ListDeployedSnses(list_deployed_snses::ListDeployedSnsesOpts),
    MakeProposal(make_proposal::MakeProposalOpts),
    MakeUpgradeCanisterProposal(make_upgrade_canister_proposal::MakeUpgradeCanisterProposalOpts),
    MakeCommitProposedBatchProposal(make_commit_proposed_batch_proposal::MakeCommitProposedBatchProposalOpts),
    NeuronId(neuron_id::NeuronIdOpts),
    NeuronPermission(neuron_permission::NeuronPermissionOpts),
    NewSaleTicket(new_sale_ticket::NewSaleTicketOpts),
    RegisterVote(register_vote::RegisterVoteOpts),
    GetSaleParticipation(get_sale_participation::GetSaleParticipationOpts),
    SplitNeuron(split_neuron::SplitNeuronOpts),
    StakeMaturity(stake_maturity::StakeMaturityOpts),
    StakeNeuron(stake_neuron::StakeNeuronOpts),
    Status(status::StatusOpts),
    Pay(pay::PayOpts),
    Transfer(transfer::TransferOpts),
}

pub fn dispatch(auth: &AuthInfo, opts: SnsOpts, qr: bool, fetch_root_key: bool) -> AnyhowResult {
    if opts.ledger {
        ensure!(matches!(
            opts.subcommand,
            SnsCommand::Balance(_) | SnsCommand::Transfer(_) | SnsCommand::NeuronPermission(_) | SnsCommand::Disburse(_)
                | SnsCommand::ConfigureDissolveDelay(_) | SnsCommand::StakeMaturity(_) | SnsCommand::NeuronId(_)
        ), "Cannot use --ledger with this command. This version of Quill only supports transfers and certain neuron management operations with a Ledger device");
    }
    let canister_ids = opts.canister_ids_file
        .context("Cannot sign message without knowing the SNS canister ids, did you forget `--canister-ids-file <json-file>`?")
        .and_then(|file| Ok(serde_json::from_slice::<SnsCanisterIds>(&fs::read(file)?)?));
    match opts.subcommand {
        SnsCommand::Balance(opts) => {
            balance::exec(auth, &canister_ids?, opts, fetch_root_key)?;
        }
        SnsCommand::ConfigureDissolveDelay(opts) => {
            let out = configure_dissolve_delay::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::Disburse(opts) => {
            let out = disburse::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::DisburseMaturity(opts) => {
            let out = disburse_maturity::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::FollowNeuron(opts) => {
            let out = follow_neuron::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::GetSwapRefund(opts) => {
            let out = get_swap_refund::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::ListDeployedSnses(opts) => list_deployed_snses::exec(opts, fetch_root_key)?,
        SnsCommand::MakeProposal(opts) => {
            let out = make_proposal::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::MakeCommitProposedBatchProposal(opts) => {
            let out = make_commit_proposed_batch_proposal::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::MakeUpgradeCanisterProposal(opts) => {
            let out = make_upgrade_canister_proposal::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::NeuronId(opts) => {
            neuron_id::exec(auth, opts)?;
        }
        SnsCommand::NeuronPermission(opts) => {
            let out = neuron_permission::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::NewSaleTicket(opts) => {
            let out = new_sale_ticket::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::RegisterVote(opts) => {
            let out = register_vote::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::GetSaleParticipation(opts) => {
            get_sale_participation::exec(auth, &canister_ids?, opts, fetch_root_key)?;
        }
        SnsCommand::SplitNeuron(opts) => {
            let out = split_neuron::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::StakeMaturity(opts) => {
            let out = stake_maturity::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::StakeNeuron(opts) => {
            let out = stake_neuron::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::Status(opts) => status::exec(&canister_ids?, opts, fetch_root_key)?,
        SnsCommand::Pay(opts) => {
            let out = pay::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
        SnsCommand::Transfer(opts) => {
            let out = transfer::exec(auth, &canister_ids?, opts)?;
            print_vec(qr, &out)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnsCanisterIds {
    pub governance_canister_id: Principal,
    pub ledger_canister_id: Principal,
    pub root_canister_id: Principal,
    pub swap_canister_id: Principal,
}

#[derive(Clone)]
pub struct ParsedSnsNeuron(pub NeuronId);

impl Display for ParsedSnsNeuron {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(&self.0.id))
    }
}

impl FromStr for ParsedSnsNeuron {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(NeuronId {
            id: hex::decode(s)?,
        }))
    }
}

fn governance_account(account: Account) -> GovAccount {
    GovAccount {
        owner: Some(account.owner.into()),
        subaccount: account.subaccount.map(|sub| Subaccount {
            subaccount: sub.to_vec(),
        }),
    }
}
