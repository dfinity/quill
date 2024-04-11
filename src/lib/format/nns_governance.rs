use std::fmt::Write;

use candid::Decode;
use ic_nns_governance::pb::v1::{GovernanceError, NeuronInfo};

use crate::lib::{
    e8s_to_tokens,
    format::{format_duration_seconds, format_timestamp_seconds},
    AnyhowResult,
};

pub fn display_get_neuron_info(blob: &[u8]) -> AnyhowResult<String> {
    let info = Decode!(blob, Result<NeuronInfo, GovernanceError>)?;
    match info {
        Ok(info) => {
            let mut fmt = format!(
                "\
Age: {age}
Total stake: {icp} ICP
Voting power: {power}

State: {state:?}
Dissolve delay: {delay}
Created {creation}
",
                age = format_duration_seconds(info.age_seconds),
                icp = e8s_to_tokens(info.stake_e8s.into()),
                power = e8s_to_tokens(info.voting_power.into()),
                state = info.state(),
                delay = format_duration_seconds(info.dissolve_delay_seconds),
                creation = format_timestamp_seconds(info.created_timestamp_seconds)
            );
            if let Some(cf) = info.joined_community_fund_timestamp_seconds {
                writeln!(
                    fmt,
                    "Member of the community fund since {}",
                    format_timestamp_seconds(cf)
                )?;
            }
            if let Some(known) = info.known_neuron_data {
                writeln!(fmt, "\nKnown neuron: {}", known.name)?;
                if let Some(desc) = known.description {
                    writeln!(fmt, "Description: {desc}")?;
                }
            }
            write!(
                fmt,
                "\nAccurate as of {}",
                format_timestamp_seconds(info.retrieved_at_timestamp_seconds)
            )?;
            Ok(fmt)
        }
        Err(e) => display_governance_error(e),
    }
}

pub fn display_governance_error(err: GovernanceError) -> AnyhowResult<String> {
    Ok(format!("NNS error: {}", err.error_message))
}
