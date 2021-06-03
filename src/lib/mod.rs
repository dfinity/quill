use anyhow::anyhow;
use candid::parser::typing::{check_prog, TypeEnv};
use candid::types::{Function, Type};
use candid::IDLProg;

pub const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
pub const GOVERNANCE_CANISTER_ID: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";

/// The type to represent DFX results.
pub type AnyhowResult<T = ()> = anyhow::Result<T>;

pub mod environment;
pub mod nns_types;
pub mod sign;

#[derive(Clone, Debug)]
pub struct NetworkDescriptor {
    pub name: String,
    pub providers: Vec<String>,
    pub is_ic: bool,
}

pub fn get_local_candid(canister_id: &str) -> Option<String> {
    match canister_id {
        GOVERNANCE_CANISTER_ID => {
            Some(String::from_utf8(include_bytes!("../../candid/governance.did").to_vec()).ok()?)
        }
        LEDGER_CANISTER_ID => {
            Some(String::from_utf8(include_bytes!("../../candid/ledger.did").to_vec()).ok()?)
        }
        _ => None,
    }
}

pub fn get_idl_string(
    blob: &[u8],
    canister_id: &str,
    method_name: &str,
    part: &str,
    output_type: &str,
) -> AnyhowResult<String> {
    let spec = get_local_candid(canister_id);
    let method_type = spec.and_then(|spec| get_candid_type(spec, method_name));
    match output_type {
        "raw" => {
            let hex_string = hex::encode(blob);
            return Ok(format!("{}", hex_string));
        }
        "idl" | "pp" => {
            let result = match method_type {
                None => candid::IDLArgs::from_bytes(blob),
                Some((env, func)) => candid::IDLArgs::from_bytes_with_types(
                    blob,
                    &env,
                    if part == "args" {
                        &func.args
                    } else {
                        &func.rets
                    },
                ),
            };
            return Ok(if output_type == "idl" {
                format!("{:?}", result?)
            } else {
                format!("{}", result?)
            });
        }
        v => return Err(anyhow!("Invalid output type: {}", v)),
    }
}

/// Parse IDL file into TypeEnv. This is a best effort function: it will succeed if
/// the IDL file can be parsed and type checked in Rust parser, and has an
/// actor in the IDL file. If anything fails, it returns None.
pub fn get_candid_type(idl: String, method_name: &str) -> Option<(TypeEnv, Function)> {
    let (env, ty) = check_candid(idl).ok()?;
    let actor = ty?;
    let method = env.get_method(&actor, method_name).ok()?.clone();
    Some((env, method))
}

pub fn check_candid(idl: String) -> AnyhowResult<(TypeEnv, Option<Type>)> {
    let ast = candid::pretty_parse::<IDLProg>("/dev/null", &idl)?;
    let mut env = TypeEnv::new();
    let actor = check_prog(&mut env, &ast)?;
    Ok((env, actor))
}

pub fn read_json(path: &str) -> AnyhowResult<String> {
    use std::io::Read;
    let mut json = String::new();
    if path == "-" {
        std::io::stdin().read_to_string(&mut json)?;
    } else {
        let path = std::path::Path::new(&path);
        let mut file =
            std::fs::File::open(&path).map_err(|_| anyhow!("Message file doesn't exist"))?;
        file.read_to_string(&mut json)
            .map_err(|_| anyhow!("Cannot read the message file."))?;
    }
    Ok(json)
}
