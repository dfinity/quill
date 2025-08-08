use crate::{
    commands::transfer::{self, parse_tokens},
    lib::{
        get_principal, governance_canister_id,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult, AuthInfo, ParsedNnsAccount, ParsedSubaccount, ROLE_NNS_GOVERNANCE,
    },
};
use anyhow::anyhow;
use candid::{Encode, Principal};
use clap::Parser;
use ic_nns_constants::GOVERNANCE_CANISTER_ID;
use ic_nns_governance::pb::v1::{
    manage_neuron::{
        claim_or_refresh::{By, MemoAndController},
        ClaimOrRefresh, Command,
    },
    ManageNeuron,
};
use icp_ledger::{AccountIdentifier, Subaccount, Tokens};
use sha2::{Digest, Sha256};

/// Signs topping up of a neuron (new or existing).
#[derive(Parser)]
pub struct StakeOpts {
    /// ICPs to be staked on the newly created neuron.
    #[arg(long, value_parser = parse_tokens, conflicts_with = "already_transferred", required_unless_present = "already_transferred")]
    amount: Option<Tokens>,

    /// Skips signing the transfer of ICP, signing only the staking request.
    #[arg(long)]
    already_transferred: bool,

    /// The name of the neuron (up to 8 ASCII characters).
    #[arg(
        long,
        value_parser = neuron_name_parser,
        conflicts_with = "nonce",
        required_unless_present = "nonce"
    )]
    name: Option<u64>,

    /// The nonce of the neuron.
    #[arg(long)]
    nonce: Option<u64>,

    /// Transaction fee, default is 0.0001 ICP.
    #[arg(long, value_parser = parse_tokens)]
    fee: Option<Tokens>,

    /// The subaccount to transfer from.
    #[arg(long)]
    from_subaccount: Option<ParsedSubaccount>,

    #[arg(from_global)]
    ledger: bool,
}

pub fn exec(auth: &AuthInfo, opts: StakeOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let controller = crate::lib::get_principal(auth)?;
    let nonce = match (&opts.nonce, &opts.name) {
        (Some(nonce), _) => *nonce,
        (_, Some(name)) => *name,
        _ => return Err(anyhow!("Either a nonce or a name should be specified")),
    };
    let gov_subaccount = get_neuron_subaccount(&controller, nonce);
    let account = AccountIdentifier::new(GOVERNANCE_CANISTER_ID.get(), Some(gov_subaccount));
    let mut messages = if !opts.already_transferred {
        transfer::exec(
            auth,
            transfer::TransferOpts {
                to: ParsedNnsAccount::Original(account),
                amount: opts.amount.unwrap(),
                fee: opts.fee,
                memo: Some(nonce),
                from_subaccount: opts.from_subaccount,
            },
        )?
    } else {
        Vec::new()
    };
    let args = ManageNeuron {
        neuron_id_or_subaccount: None,
        id: None,
        command: Some(Command::ClaimOrRefresh(ClaimOrRefresh {
            by: Some(By::MemoAndController(MemoAndController {
                controller: Some(get_principal(auth)?.into()),
                memo: nonce,
            })),
        })),
    };
    messages.push(sign_ingress_with_request_status_query(
        &AuthInfo::NoAuth,
        governance_canister_id(),
        ROLE_NNS_GOVERNANCE,
        "manage_neuron",
        Encode!(&args)?,
    )?);

    Ok(messages)
}

// This function _must_ correspond to how the governance canister computes the
// subaccount.
fn get_neuron_subaccount(controller: &Principal, nonce: u64) -> Subaccount {
    let mut data = Sha256::new();
    data.update([0x0c]);
    data.update(b"neuron-stake");
    data.update(controller.as_slice());
    data.update(nonce.to_be_bytes());
    Subaccount(data.finalize().into())
}

fn convert_name_to_nonce(name: &str) -> u64 {
    let mut bytes = std::collections::VecDeque::from(name.as_bytes().to_vec());
    while bytes.len() < 8 {
        bytes.push_front(0);
    }
    let mut arr: [u8; 8] = [0; 8];
    arr.copy_from_slice(&bytes.into_iter().collect::<Vec<_>>());
    u64::from_be_bytes(arr)
}

fn neuron_name_parser(name: &str) -> Result<u64, String> {
    if name.len() > 8 || !name.is_ascii() {
        return Err("The neuron name must be 8 character or less".to_string());
    }
    Ok(convert_name_to_nonce(name))
}
