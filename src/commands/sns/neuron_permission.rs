use anyhow::anyhow;
use candid::{Encode, Principal};
use clap::{Parser, ValueEnum};
use ic_sns_governance::pb::v1::{
    manage_neuron::{AddNeuronPermissions, Command, RemoveNeuronPermissions},
    ManageNeuron, NeuronPermissionList, NeuronPermissionType,
};

use crate::lib::{
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_SNS_GOVERNANCE,
};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Signs a ManageNeuron message to add or remove permissions for a principal to/from a neuron.
///
/// This will selectively enable/disable that principal to do a variety of management tasks for the neuron, including voting and disbursing.
#[derive(Parser)]
pub struct NeuronPermissionOpts {
    /// Whether to add or remove permissions.
    #[arg(value_enum)]
    subcommand: Subcmd,

    /// The id of the neuron to configure as a hex encoded string.
    neuron_id: ParsedSnsNeuron,

    /// The principal to change the permissions of.
    #[arg(long)]
    principal: Principal,

    /// The permissions to add to/remove from the principal. You can specify multiple in one command.
    #[arg(
        long,
        num_args = 1..,
        use_value_delimiter = true,
        value_enum,
        required = true,
    )]
    permissions: Vec<NeuronPermissionArg>,
}

#[derive(ValueEnum, Clone)]
enum Subcmd {
    Add,
    Remove,
}

pub fn exec(
    auth: &AuthInfo,
    canister_ids: &SnsCanisterIds,
    opts: NeuronPermissionOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let neuron_subaccount = opts.neuron_id.0.subaccount().map_err(|e| anyhow!(e))?;
    let permission_list = NeuronPermissionList {
        permissions: opts
            .permissions
            .into_iter()
            .map(|x| NeuronPermissionType::from(x) as i32)
            .collect(),
    };
    let req = ManageNeuron {
        subaccount: neuron_subaccount.to_vec(),
        command: Some(if let Subcmd::Add = opts.subcommand {
            Command::AddNeuronPermissions(AddNeuronPermissions {
                principal_id: Some(opts.principal.into()),
                permissions_to_add: Some(permission_list),
            })
        } else {
            Command::RemoveNeuronPermissions(RemoveNeuronPermissions {
                principal_id: Some(opts.principal.into()),
                permissions_to_remove: Some(permission_list),
            })
        }),
    };
    let msg = sign_ingress_with_request_status_query(
        auth,
        canister_ids.governance_canister_id,
        ROLE_SNS_GOVERNANCE,
        "manage_neuron",
        Encode!(&req)?,
    )?;
    Ok(vec![msg])
}

#[derive(ValueEnum, Debug, Copy, Clone)]
pub enum NeuronPermissionArg {
    Unspecified,
    ConfigureDissolveState,
    ManagePrincipals,
    SubmitProposal,
    Vote,
    Disburse,
    Split,
    MergeMaturity,
    DisburseMaturity,
    StakeMaturity,
    ManageVotingPermission,
}

impl From<NeuronPermissionArg> for NeuronPermissionType {
    fn from(t: NeuronPermissionArg) -> Self {
        use NeuronPermissionArg as A;
        match t {
            A::Unspecified => Self::Unspecified,
            A::ConfigureDissolveState => Self::ConfigureDissolveState,
            A::ManagePrincipals => Self::ManagePrincipals,
            A::SubmitProposal => Self::SubmitProposal,
            A::Vote => Self::Vote,
            A::Disburse => Self::Disburse,
            A::Split => Self::Split,
            A::MergeMaturity => Self::MergeMaturity,
            A::DisburseMaturity => Self::DisburseMaturity,
            A::StakeMaturity => Self::StakeMaturity,
            A::ManageVotingPermission => Self::ManageVotingPermission,
        }
    }
}
