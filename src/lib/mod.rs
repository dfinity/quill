use crate::lib::nns_types::{GOVERNANCE_CANISTER_ID, LEDGER_CANISTER_ID};
use crate::{error_invalid_argument, error_invalid_data, error_unknown};
use candid::parser::typing::{check_prog, TypeEnv};
use candid::types::{Function, Type};
use candid::{parser::value::IDLValue, IDLArgs, IDLProg};

/// The type to represent DFX results.
pub type DfxResult<T = ()> = anyhow::Result<T>;

pub mod environment;
pub mod identity;
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
    output_type: &str,
    method_type: &Option<(TypeEnv, Function)>,
) -> DfxResult<String> {
    match output_type {
        "raw" => {
            let hex_string = hex::encode(blob);
            return Ok(format!("{}", hex_string));
        }
        "idl" | "pp" => {
            let result = match method_type {
                None => candid::IDLArgs::from_bytes(blob),
                Some((env, func)) => candid::IDLArgs::from_bytes_with_types(blob, &env, &func.args),
            };
            return Ok(if output_type == "idl" {
                format!("{:?}", result?)
            } else {
                format!("{}", result?)
            });
        }
        v => return Err(error_unknown!("Invalid output type: {}", v)),
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

pub fn check_candid(idl: String) -> DfxResult<(TypeEnv, Option<Type>)> {
    let ast = candid::pretty_parse::<IDLProg>("/dev/null", &idl)?;
    let mut env = TypeEnv::new();
    let actor = check_prog(&mut env, &ast)?;
    Ok((env, actor))
}

pub fn blob_from_arguments(
    arguments: Option<&str>,
    arg_type: Option<&str>,
    method_type: &Option<(TypeEnv, Function)>,
) -> DfxResult<Vec<u8>> {
    let arg_type = arg_type.unwrap_or("idl");
    match arg_type {
        "raw" => {
            let bytes = hex::decode(&arguments.unwrap_or("")).map_err(|e| {
                error_invalid_argument!("Argument is not a valid hex string: {}", e)
            })?;
            Ok(bytes)
        }
        "idl" => {
            let typed_args = match method_type {
                None => {
                    let arguments = arguments.unwrap_or("()");
                    candid::pretty_parse::<IDLArgs>("Candid argument", &arguments)
                        .map_err(|e| error_invalid_argument!("Invalid Candid values: {}", e))?
                        .to_bytes()
                }
                Some((env, func)) => {
                    if let Some(arguments) = arguments {
                        let first_char = arguments.chars().next();
                        let is_candid_format = first_char.map_or(false, |c| c == '(');
                        // If parsing fails and method expects a single value, try parsing as IDLValue.
                        // If it still fails, and method expects a text type, send arguments as text.
                        let args = arguments.parse::<IDLArgs>().or_else(|_| {
                            if func.args.len() == 1 && !is_candid_format {
                                let is_quote = first_char.map_or(false, |c| c == '"');
                                if candid::types::Type::Text == func.args[0] && !is_quote {
                                    Ok(IDLValue::Text(arguments.to_string()))
                                } else {
                                    candid::pretty_parse::<IDLValue>("Candid argument", &arguments)
                                }
                                .map(|v| IDLArgs::new(&[v]))
                            } else {
                                candid::pretty_parse::<IDLArgs>("Candid argument", &arguments)
                            }
                        });
                        args.map_err(|e| error_invalid_argument!("Invalid Candid values: {}", e))?
                            .to_bytes_with_types(&env, &func.args)
                    } else if func.args.is_empty() {
                        use candid::Encode;
                        Encode!()
                    } else {
                        return Err(error_invalid_data!("Expected arguments but found none."));
                    }
                }
            }
            .map_err(|e| error_invalid_data!("Unable to serialize Candid values: {}", e))?;
            Ok(typed_args)
        }
        v => Err(error_unknown!("Invalid type: {}", v)),
    }
}

#[macro_export]
macro_rules! error_invalid_argument {
    ($($args:tt)*) => {
        anyhow::anyhow!("Invalid argument: {}", format_args!($($args)*))
    }
}

#[macro_export]
macro_rules! error_invalid_config {
    ($($args:tt)*) => {
        anyhow::anyhow!("Invalid configuration: {}", format_args!($($args)*))
    }
}

#[macro_export]
macro_rules! error_invalid_data {
    ($($args:tt)*) => {
        anyhow::anyhow!("Invalid data: {}", format_args!($($args)*))
    }
}

#[macro_export]
macro_rules! error_unknown {
    ($($args:tt)*) => {
        anyhow::anyhow!("Unknown error: {}", format_args!($($args)*))
    }
}
