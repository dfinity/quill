use candid::Decode;

use crate::lib::AnyhowResult;

pub fn display_update_node_operator_config_directly(blob: &[u8]) -> AnyhowResult<String> {
    Decode!(blob, ())?;
    Ok("Successfully updated node operator config".to_string())
}
