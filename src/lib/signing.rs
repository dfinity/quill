use crate::lib::get_idl_string;
use crate::lib::{AnyhowResult, AuthInfo};
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use ic_agent::agent::UpdateBuilder;
use ic_agent::RequestId;
use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use std::convert::TryFrom;
use std::time::Duration;

#[cfg(feature = "ledger")]
use super::ledger::LedgerIdentity;
use super::{get_agent, resolve_role};

/// Represents a signed message with the corresponding request id.
#[derive(Clone)]
pub struct SignedMessageWithRequestId {
    pub message: Ingress,
    pub request_id: Option<RequestId>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct RequestStatus {
    pub canister_id: String,
    pub request_id: String,
    pub content: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Ingress {
    pub call_type: String,
    pub request_id: Option<String>,
    pub content: String,
    pub role: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct IngressWithRequestId {
    pub ingress: Ingress,
    pub request_status: RequestStatus,
}

impl Ingress {
    pub fn parse(&self) -> AnyhowResult<(Principal, Principal, String, String, String)> {
        let cbor: Value = serde_cbor::from_slice(&hex::decode(&self.content)?)
            .context("Invalid cbor data in the content of the message.")?;
        if let Value::Map(m) = cbor {
            let cbor_content = m
                .get(&Value::Text("content".to_string()))
                .ok_or_else(|| anyhow!("Invalid cbor content"))?;
            if let Value::Map(m) = cbor_content {
                if let (
                    Some(Value::Bytes(sender)),
                    Some(Value::Bytes(canister_id)),
                    Some(Value::Text(method_name)),
                    Some(Value::Bytes(arg)),
                ) = (
                    m.get(&Value::Text("sender".to_string())),
                    m.get(&Value::Text("canister_id".to_string())),
                    m.get(&Value::Text("method_name".to_string())),
                    m.get(&Value::Text("arg".to_string())),
                ) {
                    let sender = Principal::try_from(sender)?;
                    let canister_id = Principal::try_from(canister_id)?;
                    // `call_type` is not covered by the signature; it decides whether
                    // `send` asks for confirmation, so it must match the signed envelope.
                    let request_type = match m.get(&Value::Text("request_type".to_string())) {
                        Some(Value::Text(t)) => t.as_str(),
                        _ => bail!("Invalid cbor content: missing request_type"),
                    };
                    let expected_request_type = match self.call_type.as_str() {
                        "update" => "call",
                        "query" => "query",
                        other => bail!("Unknown call type '{other}' in message"),
                    };
                    if request_type != expected_request_type {
                        bail!(
                            "Message is labeled as call type '{}' but the signed content is a \
                             '{request_type}' request. The message file may have been tampered with.",
                            self.call_type
                        );
                    }
                    let role = resolve_role(canister_id, self.role.as_deref())?;
                    return Ok((
                        sender,
                        canister_id,
                        method_name.to_string(),
                        get_idl_string(arg, canister_id, role, method_name, "args")?,
                        role.to_owned(),
                    ));
                }
            }
        }
        Err(anyhow!("Invalid cbor content"))
    }
}

pub fn request_status_sign(
    auth: &AuthInfo,
    request_id: RequestId,
    canister_id: Principal,
) -> AnyhowResult<RequestStatus> {
    let agent = get_agent(auth)?;
    let val = agent.sign_request_status(canister_id, request_id)?;
    Ok(RequestStatus {
        canister_id: canister_id.to_string(),
        request_id: request_id.into(),
        content: hex::encode(val.signed_request_status),
    })
}

pub fn sign(
    auth: &AuthInfo,
    canister_id: Principal,
    method_name: &str,
    args: Vec<u8>,
    role: &str,
    #[allow(unused)] is_staking: bool,
) -> AnyhowResult<SignedMessageWithRequestId> {
    let ingress_expiry = Duration::from_secs(5 * 60);
    let agent = get_agent(auth)?;
    let signed_update = sign_with(auth, is_staking, || {
        UpdateBuilder::new(&agent, canister_id, method_name.to_string())
            .with_arg(args)
            .expire_after(ingress_expiry)
            .sign()
    })?;
    let content = hex::encode(signed_update.signed_update);
    let request_id = signed_update.request_id;

    Ok(SignedMessageWithRequestId {
        message: Ingress {
            call_type: "update".to_string(),
            request_id: Some(request_id.into()),
            content,
            role: Some(role.to_owned()),
        },
        request_id: Some(request_id),
    })
}

fn sign_with<T>(
    #[allow(unused)] auth: &AuthInfo,
    #[allow(unused)] is_staking: bool,
    f: impl FnOnce() -> T,
) -> T {
    #[cfg(feature = "ledger")]
    if is_staking && matches!(auth, AuthInfo::Ledger) {
        return LedgerIdentity::with_staking(f);
    }
    f()
}

/// Generates a bundle of signed messages (ingress + request status query).
pub fn sign_ingress_with_request_status_query(
    auth: &AuthInfo,
    canister_id: Principal,
    role: &str,
    method_name: &str,
    args: Vec<u8>,
) -> AnyhowResult<IngressWithRequestId> {
    sign_ingress_with_request_status_query_internal(
        auth,
        canister_id,
        role,
        method_name,
        args,
        false,
    )
}

/// Same as [`sign_ingress_with_request_status_query`], but signals that the request is staking.
pub fn sign_staking_ingress_with_request_status_query(
    auth: &AuthInfo,
    canister_id: Principal,
    role: &str,
    method_name: &str,
    args: Vec<u8>,
) -> AnyhowResult<IngressWithRequestId> {
    sign_ingress_with_request_status_query_internal(
        auth,
        canister_id,
        role,
        method_name,
        args,
        true,
    )
}

fn sign_ingress_with_request_status_query_internal(
    auth: &AuthInfo,
    canister_id: Principal,
    role: &str,
    method_name: &str,
    args: Vec<u8>,
    is_staking: bool,
) -> AnyhowResult<IngressWithRequestId> {
    #[cfg(feature = "ledger")]
    if matches!(auth, AuthInfo::Ledger)
        && !super::ledger::supported_transaction(&canister_id, method_name)
    {
        anyhow::bail!(
            "Cannot use --ledger with this command. This version of Quill only supports transfers \
            and certain neuron management operations with a Ledger device"
        );
    }
    let msg_with_req_id = sign(auth, canister_id, method_name, args, role, is_staking)?;
    let request_id = msg_with_req_id
        .request_id
        .context("No request id for transfer call found")?;
    let request_status = request_status_sign(auth, request_id, canister_id)?;
    let message = IngressWithRequestId {
        ingress: msg_with_req_id.message,
        request_status,
    };
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::{ROLE_NNS_GOVERNANCE, ROLE_SNS_GOVERNANCE};
    use candid::Encode;
    use ic_sns_governance::pb::v1::{
        manage_neuron::{disburse::Amount, Command, Disburse},
        Account, ManageNeuron,
    };
    use std::collections::BTreeMap;

    const SNS_GOVERNANCE: &str = "zqfso-syaaa-aaaaq-aaafq-cai";
    const PAYEE: &str = "jndu2-vwnnt-bpu6t-2jrke-fg3kj-vbrgf-ajecf-gv6ju-onyol-wc3e5-kqe";

    fn ingress(
        canister_id: Principal,
        request_type: &str,
        arg: Vec<u8>,
        call_type: &str,
        role: Option<&str>,
    ) -> Ingress {
        let text = |s: &str| Value::Text(s.to_string());
        let content = BTreeMap::from([
            (text("request_type"), text(request_type)),
            (
                text("sender"),
                Value::Bytes(Principal::anonymous().as_slice().to_vec()),
            ),
            (
                text("canister_id"),
                Value::Bytes(canister_id.as_slice().to_vec()),
            ),
            (text("method_name"), text("manage_neuron")),
            (text("arg"), Value::Bytes(arg)),
        ]);
        let envelope = Value::Map(BTreeMap::from([(text("content"), Value::Map(content))]));
        Ingress {
            call_type: call_type.to_string(),
            request_id: None,
            content: hex::encode(serde_cbor::to_vec(&envelope).unwrap()),
            role: role.map(str::to_string),
        }
    }

    fn sns_disburse() -> Vec<u8> {
        let payee = Principal::from_text(PAYEE).unwrap();
        Encode!(&ManageNeuron {
            subaccount: vec![1; 32],
            command: Some(Command::Disburse(Disburse {
                amount: Some(Amount { e8s: 9_999_000_000 }),
                to_account: Some(Account {
                    owner: Some(payee.into()),
                    subaccount: None,
                }),
            })),
        })
        .unwrap()
    }

    #[test]
    fn honest_role_shows_disburse_destination() {
        let canister = Principal::from_text(SNS_GOVERNANCE).unwrap();
        let msg = ingress(
            canister,
            "call",
            sns_disburse(),
            "update",
            Some(ROLE_SNS_GOVERNANCE),
        );
        let (_, _, _, args, _) = msg.parse().unwrap();
        assert!(args.contains(PAYEE), "{args}");
    }

    /// `role` is not covered by the signature, so it must not be able to pick a
    /// candid interface that silently drops fields of the signed argument.
    #[test]
    fn nns_role_on_sns_canister_is_rejected() {
        let canister = Principal::from_text(SNS_GOVERNANCE).unwrap();
        let msg = ingress(
            canister,
            "call",
            sns_disburse(),
            "update",
            Some(ROLE_NNS_GOVERNANCE),
        );
        match msg.parse() {
            Ok((_, _, _, args, _)) => panic!("tampered role accepted, displayed: {args}"),
            Err(e) => assert!(e.to_string().contains(ROLE_NNS_GOVERNANCE), "{e}"),
        }
    }

    /// NNS manage_neuron decodes with the NNS interface whether or not the role is given.
    #[test]
    fn nns_governance_role_defaults_from_canister() {
        let arg = Encode!(&ic_nns_governance_api::ManageNeuronRequest::default()).unwrap();
        for role in [None, Some(ROLE_NNS_GOVERNANCE)] {
            let msg = ingress(
                crate::lib::governance_canister_id(),
                "call",
                arg.clone(),
                "update",
                role,
            );
            let (_, _, _, _, resolved) = msg.parse().unwrap();
            assert_eq!(resolved, ROLE_NNS_GOVERNANCE);
        }
        let msg = ingress(
            crate::lib::governance_canister_id(),
            "call",
            arg,
            "update",
            Some(ROLE_SNS_GOVERNANCE),
        );
        assert!(msg.parse().is_err());
    }

    /// The ICP ledger is legitimately addressed with both its own and the ICRC-1 interface.
    #[test]
    fn icp_ledger_accepts_icrc1_role() {
        use crate::lib::{ledger_canister_id, resolve_role, ROLE_ICRC1_LEDGER, ROLE_NNS_LEDGER};
        for role in [ROLE_NNS_LEDGER, ROLE_ICRC1_LEDGER] {
            assert_eq!(
                resolve_role(ledger_canister_id(), Some(role)).unwrap(),
                role
            );
        }
        let sns = Principal::from_text(SNS_GOVERNANCE).unwrap();
        assert!(resolve_role(sns, Some(ROLE_NNS_LEDGER)).is_err());
        assert_eq!(
            resolve_role(sns, Some(ROLE_ICRC1_LEDGER)).unwrap(),
            ROLE_ICRC1_LEDGER
        );
    }

    /// `call_type` decides whether `send` asks for confirmation, so it must match the envelope.
    #[test]
    fn call_type_must_match_signed_request_type() {
        let canister = Principal::from_text(SNS_GOVERNANCE).unwrap();
        let msg = ingress(
            canister,
            "call",
            sns_disburse(),
            "query",
            Some(ROLE_SNS_GOVERNANCE),
        );
        assert!(msg.parse().is_err());
        let msg = ingress(
            canister,
            "query",
            sns_disburse(),
            "update",
            Some(ROLE_SNS_GOVERNANCE),
        );
        assert!(msg.parse().is_err());
        let msg = ingress(
            canister,
            "query",
            sns_disburse(),
            "query",
            Some(ROLE_SNS_GOVERNANCE),
        );
        assert!(msg.parse().is_ok());
    }
}
