use crate::lib::{format::filters, AnyhowResult, ParsedAccount};

use anyhow::{anyhow, Context};
use askama::Template;
use candid::Decode;
use ic_sns_swap::pb::v1::{
    error_refund_icp_response::Result as RefundResult,
    new_sale_ticket_response::{err::Type, Result as TicketResult},
    BuyerState, ErrorRefundIcpResponse, GetBuyerStateResponse, Icrc1Account, NewSaleTicketResponse,
    RefreshBuyerTokensResponse,
};

use super::icrc1_account;

pub fn display_get_buyer_state(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, GetBuyerStateResponse)?;
    let state = response.buyer_state.context("buyer state was null")?;
    #[derive(Template)]
    #[template(path = "sns/get_buyer_state.txt")]
    struct GetBuyerState {
        state: BuyerState,
    }
    let fmt = GetBuyerState { state }.render()?;
    Ok(fmt)
}

pub fn display_refund(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, ErrorRefundIcpResponse)?;
    let result = response.result.context("result was null")?;
    #[derive(Template)]
    #[template(path = "sns/refund.txt")]
    struct Refund {
        result: RefundResult,
    }
    let fmt = Refund { result }.render()?;
    Ok(fmt)
}

pub fn display_new_sale_ticket(blob: &[u8]) -> AnyhowResult<String> {
    use Type::*;
    #[derive(Template)]
    #[template(path = "sns/new_sale_ticket.txt")]
    struct NewSaleTicket {
        result: TicketResult,
    }
    let response = Decode!(blob, NewSaleTicketResponse)?;
    let result = response.result.context("result was null")?;
    let fmt = NewSaleTicket { result }.render()?;
    Ok(fmt)
}

pub fn display_refresh_buyer_tokens(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, RefreshBuyerTokensResponse)?;
    #[derive(Template)]
    #[template(path = "sns/refresh_buyer_tokens.txt")]
    struct RefreshBuyerTokens {
        response: RefreshBuyerTokensResponse,
    }
    let fmt = RefreshBuyerTokens { response }.render()?;
    Ok(fmt)
}

fn icrc1_helper(account: &Icrc1Account) -> AnyhowResult<ParsedAccount> {
    Ok(icrc1_account(
        account.owner.unwrap().0,
        Some(
            account
                .subaccount()
                .try_into()
                .map_err(|_| anyhow!("subaccount had wrong length"))?,
        ),
    ))
}
