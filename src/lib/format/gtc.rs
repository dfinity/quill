use askama::Template;
use candid::Decode;
use ic_nns_common::pb::v1::NeuronId;

use crate::lib::AnyhowResult;

pub fn format_claim_neurons(blob: &[u8]) -> AnyhowResult<String> {
    #[derive(Template)]
    #[template(path = "claim_neurons.txt")]
    struct ClaimNeurons {
        ids: Vec<u64>,
    }
    let result = Decode!(blob, Result<Vec<NeuronId>, String>)?;
    let fmt = match result {
        Ok(ids) => ClaimNeurons {
            ids: ids.iter().map(|id| id.id).collect(),
        }
        .render()?,
        Err(e) => format!("NNS error: {e}"),
    };
    Ok(fmt)
}
