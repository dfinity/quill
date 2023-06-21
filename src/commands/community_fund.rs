use anyhow::ensure;
use candid::Encode;
use clap::{ArgGroup, Parser};
use ic_nns_governance::pb::v1::{
    manage_neuron::{
        configure::Operation, Command, Configure, JoinCommunityFund, LeaveCommunityFund,
    },
    ManageNeuron,
};

use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult, AuthInfo, ParsedNeuron, ROLE_NNS_GOVERNANCE,
};

/// Signs a message to join or leave the Internet Computer's community fund with this neuron's maturity.
#[derive(Parser)]
#[clap(group(ArgGroup::new("state").required(true)))]
pub struct CommunityFundOpts {
    /// The ID of the neuron to configure.
    neuron_id: ParsedNeuron,

    /// Join the community fund.
    #[clap(long, group = "state")]
    join: bool,

    /// Leave the community fund.
    #[clap(long, group = "state")]
    leave: bool,

    #[clap(from_global)]
    ledger: bool,
}

pub fn exec(auth: &AuthInfo, opts: CommunityFundOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    ensure!(
        !opts.ledger,
        "Cannot use `--ledger` with this command. This version of Quill does not support joining or leaving the community fund with a Ledger device.",
    );
    let command = if opts.join {
        Command::Configure(Configure {
            operation: Some(Operation::JoinCommunityFund(JoinCommunityFund {})),
        })
    } else if opts.leave {
        Command::Configure(Configure {
            operation: Some(Operation::LeaveCommunityFund(LeaveCommunityFund {})),
        })
    } else {
        unreachable!()
    };
    let arg = ManageNeuron {
        id: Some(opts.neuron_id.0),
        command: Some(command),
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
