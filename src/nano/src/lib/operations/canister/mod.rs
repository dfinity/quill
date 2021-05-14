use ic_types::principal::Principal as CanisterId;
use std::path::PathBuf;

pub fn get_local_candid_path(canister_id: CanisterId) -> Option<PathBuf> {
    match canister_id.to_string().as_ref() {
        crate::lib::nns_types::LEDGER_CANISTER_ID => Some(PathBuf::from("ledger.did")),
        _ => None,
    }
}
