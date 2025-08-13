use anyhow::Context;
use askama::Template;
use candid::Decode;
use ic_sns_wasm::pb::v1::{DeployedSns, ListDeployedSnsesResponse};

use crate::lib::AnyhowResult;

pub fn display_list_snses(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, ListDeployedSnsesResponse)?;
    #[derive(Template)]
    #[template(path = "sns/list_snses.txt")]
    struct ListSnses {
        instances: Vec<DeployedSns>,
    }
    let fmt = ListSnses {
        instances: response.instances,
    }
    .render()?;
    Ok(fmt)
}
