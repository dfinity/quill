use candid::Encode;
use clap::{ArgAction, ArgGroup, Parser, ValueEnum};
use ic_sns_governance::pb::v1::{
    manage_neuron::{Command, Follow},
    ManageNeuron, NeuronId,
};

use crate::lib::{
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ROLE_SNS_GOVERNANCE,
};

use super::{ParsedSnsNeuron, SnsCanisterIds};

/// Configures a neuron to follow another neuron or group of neurons. Following neurons
/// causes your neuron to automatically vote when the majority (by voting power) of the
/// followed neurons do.
///
/// Follow relationships are granular by the purpose of a proposal, e.g. you can follow a neuron
/// only for votes on transferring SNS treasury funds. There are several built-in proposal
/// functions which this command accepts by name, but new ones can be added at any time via
/// AddGenericNervousSystemFunction proposals and those must be addressed by integer ID.
#[derive(Parser)]
#[clap(
    group(ArgGroup::new("function").required(true)),
    group(ArgGroup::new("following").required(true)),
)]
pub struct FollowNeuronOpts {
    /// The neuron to configure.
    neuron_id: ParsedSnsNeuron,
    /// The name of the built-in proposal function to restrict following to.
    #[clap(long, group = "function", value_enum)]
    r#type: Option<Type>,
    /// The numeric ID of the proposal function to restrict following to.
    #[clap(long, group = "function")]
    function_id: Option<u64>,
    /// A list of neurons to follow for this proposal function, separated by commas.
    #[clap(long, action = ArgAction::Append, value_delimiter = ',', group = "following")]
    followees: Vec<ParsedSnsNeuron>,
    /// Remove any followees for this proposal function.
    #[clap(long, group = "following")]
    unfollow: bool,
}

#[derive(ValueEnum, Clone)]
enum Type {
    All = 0,
    Motion = 1,
    ManageNervousSystemParameters = 2,
    UpgradeSnsControlledCanister = 3,
    AddGenericNervousSystemFunction = 4,
    RemoveGenericNervousSystemFunction = 5,
    UpgradeSnsToNextVersion = 7,
    ManageSnsMetadata = 8,
    TransferSnsTreasuryFunds = 9,
    RegisterDappCanisters = 10,
    DeregisterDappCanisters = 11,
    MintSnsTokens = 12,
}

pub fn exec(
    auth: &AuthInfo,
    canister_ids: &SnsCanisterIds,
    opts: FollowNeuronOpts,
) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let function_id = if let Some(id) = opts.function_id {
        id
    } else {
        opts.r#type.unwrap() as u64
    };
    let followees = if opts.unfollow {
        vec![]
    } else {
        opts.followees
            .into_iter()
            .map(|followee| NeuronId { id: followee.0.id })
            .collect()
    };
    let args = ManageNeuron {
        subaccount: opts.neuron_id.0.id,
        command: Some(Command::Follow(Follow {
            function_id,
            followees,
        })),
    };
    let message = sign_ingress_with_request_status_query(
        auth,
        canister_ids.governance_canister_id,
        ROLE_SNS_GOVERNANCE,
        "manage_neuron",
        Encode!(&args)?,
    )?;
    Ok(vec![message])
}
