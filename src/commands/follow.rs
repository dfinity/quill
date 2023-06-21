use anyhow::ensure;
use candid::Encode;
use clap::{ArgAction, ArgEnum, ArgGroup, Parser};
use ic_nns_governance::pb::v1::{
    manage_neuron::{Command, Follow},
    ManageNeuron,
};

use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ROLE_NNS_GOVERNANCE,
};

/// Signs a neuron configuration message to change a neuron's follow relationships.
#[derive(Parser)]
#[clap(
    group(ArgGroup::new("topic").required(true)),
    group(ArgGroup::new("following").required(true))
)]
pub struct FollowOpts {
    /// The ID of the neuron to configure.
    neuron_id: ParsedNeuron,

    /// The name of the proposal topic to restrict following to.
    #[clap(long, arg_enum, group = "topic")]
    r#type: Option<Type>,

    /// The numeric ID of the proposal topic to restrict following to.
    #[clap(long, group = "topic", alias = "function-id")]
    topic_id: Option<i32>,

    /// A comma-separated list of neuron IDs to follow.
    #[clap(long, action = ArgAction::Append, value_delimiter = ',', group = "following")]
    followees: Vec<ParsedNeuron>,

    /// Unfollow all neurons for this topic.
    #[clap(long, group = "following")]
    unfollow: bool,

    #[clap(from_global)]
    ledger: bool,
}

pub fn exec(auth: &AuthInfo, opts: FollowOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    ensure!(!opts.ledger, "Cannot use `--ledger` with this command. This version of Quill does not support changing follow relationships with a Ledger device.");
    let topic = opts.topic_id.unwrap_or_else(|| opts.r#type.unwrap() as i32);
    let arg = ManageNeuron {
        id: Some(opts.neuron_id.0),
        command: Some(Command::Follow(Follow {
            followees: opts.followees.into_iter().map(|x| x.0).collect(), // empty vec if --unfollow was specified
            topic,
        })),
        neuron_id_or_subaccount: None,
    };
    let msg = sign_ingress_with_request_status_query(
        auth,
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "manage_neuron",
        Encode!(&arg)?,
    )?;
    Ok(vec![msg])
}

#[derive(ArgEnum, Clone)]
enum Type {
    All = 0,
    #[clap(alias = "manage-neuron")]
    NeuronManagement = 1,
    ExchangeRate = 2,
    NetworkEconomics = 3,
    Governance = 4,
    NodeAdmin = 5,
    ParticipantManagement = 6,
    SubnetManagement = 7,
    NetworkCanisterManagement = 8,
    Kyc = 9,
    NodeProviderRewards = 10,
    SnsDecentralizationSale = 11,
    SubnetReplicaVersionManagement = 12,
    ReplicaVersionManagement = 13,
    SnsAndCommunityFund = 14,
}
