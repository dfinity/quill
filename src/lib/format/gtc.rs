use candid::Decode;
use ic_nns_common::pb::v1::NeuronId;
use itertools::Itertools;

use crate::lib::AnyhowResult;

pub fn format_claim_neurons(blob: &[u8]) -> AnyhowResult<String> {
    let result = Decode!(blob, Result<Vec<NeuronId>, String>)?;
    let fmt = match result {
        Ok(ids) => format!(
            "Claimed neurons {}",
            ids.iter()
                .format_with(", ", |i, f| f(&format_args!("{}", i.id)))
        ),
        Err(e) => format!("NNS error: {e}"),
    };
    Ok(fmt)
}
