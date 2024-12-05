use anyhow::Context;
use candid::Decode;
use ic_sns_swap::pb::v1::{
    error_refund_icp_response::Result as RefundResult,
    new_sale_ticket_response::{err::Type, Result as TicketResult},
    ErrorRefundIcpResponse, GetBuyerStateResponse, NewSaleTicketResponse,
    RefreshBuyerTokensResponse,
};
use std::fmt::Write;

use crate::lib::{e8s_to_tokens, AnyhowResult};

use super::{format_timestamp_nanoseconds, icrc1_account};

pub fn display_get_buyer_state(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, GetBuyerStateResponse)?;
    let state = response.buyer_state.context("buyer state was null")?;
    let fmt = format!(
        "Total participation: {icp} ICP",
        icp = e8s_to_tokens(state.amount_icp_e8s().into())
    );
    Ok(fmt)
}

pub fn display_refund(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, ErrorRefundIcpResponse)?;
    let result = response.result.context("result was null")?;
    let fmt = match result {
        RefundResult::Ok(transfer) => {
            if let Some(index) = transfer.block_height {
                format!("Refunded ICP at block index {index}")
            } else {
                "Refunded ICP (unknown block index)".to_string()
            }
        }
        RefundResult::Err(err) => format!("Refund error: {}", err.description()),
    };
    Ok(fmt)
}

pub fn display_new_sale_ticket(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, NewSaleTicketResponse)?;
    let result = response.result.context("result was null")?;
    let fmt = match result {
        TicketResult::Ok(ticket) => {
            if let Some(ticket) = ticket.ticket {
                let mut fmt = format!(
                    "\
Successfully created ticket with ID {id}
Creation time: {time}
Ticket amount: {amount} ICP",
                    id = ticket.ticket_id,
                    time = format_timestamp_nanoseconds(ticket.creation_time),
                    amount = e8s_to_tokens(ticket.amount_icp_e8s.into()),
                );
                if let Some(account) = ticket.account {
                    if let Some(owner) = account.owner {
                        write!(
                            fmt,
                            "\nTicket owner: {}",
                            icrc1_account(
                                owner.into(),
                                Some(
                                    account
                                        .subaccount()
                                        .try_into()
                                        .context("subaccount had wrong length")?
                                )
                            )
                        )?;
                    }
                }
                fmt
            } else {
                "Ticket successfully created (no data available)".to_string()
            }
        }
        TicketResult::Err(err) => match err.error_type() {
            Type::InvalidPrincipal => {
                "Ticket creation error: This principal is forbidden from creating tickets"
                    .to_string()
            }
            Type::InvalidSubaccount => {
                "Ticket creation error: Invalid subaccount, not 32 bytes (64 hex digits)"
                    .to_string()
            }
            Type::InvalidUserAmount => {
                if let Some(user_amount) = err.invalid_user_amount {
                    format!("Ticket creation error: Invalid amount, must be between {min} and {max} ICP", min = e8s_to_tokens(user_amount.min_amount_icp_e8s_included.into()), max = e8s_to_tokens(user_amount.max_amount_icp_e8s_included.into()))
                } else {
                    "Ticket creation error: Invalid amount, not within the required range"
                        .to_string()
                }
            }
            Type::SaleClosed => {
                "Ticket creation error: Token sale has already been closed".to_string()
            }
            Type::SaleNotOpen => "Ticket creation error: Token sale has not yet opened".to_string(),
            Type::TicketExists => {
                if let Some(ticket) = err.existing_ticket {
                    let mut fmt = format!(
                        "\
Ticket creation error: An open ticket from this account already exists.
Ticket ID: {id}
Creation time: {time}
Ticket amount: {amount} ICP",
                        id = ticket.ticket_id,
                        time = format_timestamp_nanoseconds(ticket.creation_time),
                        amount = e8s_to_tokens(ticket.amount_icp_e8s.into())
                    );
                    if let Some(account) = ticket.account {
                        if let Some(owner) = account.owner {
                            write!(
                                fmt,
                                "\nTicket owner: {}",
                                icrc1_account(
                                    owner.into(),
                                    Some(
                                        account
                                            .subaccount()
                                            .try_into()
                                            .context("subaccount had wrong length")?
                                    )
                                )
                            )?;
                        }
                    }
                    fmt
                } else {
                    "Ticket creation error: An open ticket from this account already exists."
                        .to_string()
                }
            }
            Type::Unspecified => "Ticket creation error: unknown".to_string(),
        },
    };
    Ok(fmt)
}

pub fn display_refresh_buyer_tokens(blob: &[u8]) -> AnyhowResult<String> {
    let response = Decode!(blob, RefreshBuyerTokensResponse)?;
    let fmt = format!(
        "\
Ticket balance: {ticket} ICP
Total transferred ICP: {total} ICP",
        ticket = e8s_to_tokens(response.icp_accepted_participation_e8s.into()),
        total = e8s_to_tokens(response.icp_ledger_account_balance_e8s.into()),
    );
    Ok(fmt)
}
