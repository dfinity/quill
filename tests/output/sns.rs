use crate::{
    asset, ledger_compatible, quill, quill_authed, quill_query, quill_sns_query,
    quill_sns_query_authed, quill_sns_send, OutputExt, PRINCIPAL,
};

const NEURON_ID: &str = "83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069";
const FOLLOWEE: &str = "75c606cac8d0dab0a6f5db99d64c9b5312ed8cca2f971ea0ea960926db530d7f";

// uncomment tests on next ledger app update
ledger_compatible![
    // follow,
    transfer,
    neuron_id,
    neuron_permission,
    dissolve,
    disburse,
    // make_proposal,
    // stake_neuron,
    stake_maturity,
    // vote,
];

#[test]
fn balance() {
    quill_sns_query(&format!("sns balance --of {PRINCIPAL}")).diff("sns/balance/simple.txt");
}

#[test]
fn dissolve_delay() {
    quill_sns_send(&format!(
        "sns configure-dissolve-delay {NEURON_ID} --additional-dissolve-delay-seconds 1000"
    ))
    .diff("sns/dissolve_delay/add_seconds.txt");
}

#[test]
fn dissolve() {
    quill_sns_send(&format!(
        "sns configure-dissolve-delay {NEURON_ID} --start-dissolving"
    ))
    .diff("sns/dissolve_delay/start_dissolving.txt");
    quill_sns_send(&format!(
        "sns configure-dissolve-delay {NEURON_ID} --stop-dissolving"
    ))
    .diff("sns/dissolve_delay/stop_dissolving.txt");
}

#[test]
fn disburse() {
    quill_sns_send(&format!(
        "sns disburse {NEURON_ID} --to {PRINCIPAL} --amount 50"
    ))
    .diff("sns/disburse/simple.txt");
    quill_sns_send(&format!("sns disburse {NEURON_ID} --subaccount 02"))
        .diff("sns/disburse/subaccount.txt");
}

#[test]
fn stake_maturity() {
    quill_sns_send(&format!("sns stake-maturity {NEURON_ID} --percentage 70"))
        .diff("sns/manage_neuron/stake_maturity.txt");
}

#[test]
fn manage_neuron() {
    quill_sns_send(&format!(
        "sns disburse-maturity {NEURON_ID} --percentage 25 --to {PRINCIPAL}"
    ))
    .diff("sns/manage_neuron/disburse.txt");
    quill_sns_send(&format!(
        "sns disburse-maturity {NEURON_ID} --subaccount 03"
    ))
    .diff("sns/manage_neuron/disburse_subaccount.txt");
    quill_sns_send(&format!(
        "sns split-neuron {NEURON_ID} --memo 47 --amount 230.5"
    ))
    .diff("sns/manage_neuron/split.txt")
}

#[test]
fn follow() {
    quill_sns_send(&format!(
        "sns follow-neuron {NEURON_ID} --type transfer-sns-treasury-funds --followees {FOLLOWEE}"
    ))
    .diff("sns/follow/follow.txt");
    quill_sns_send(&format!(
        "sns follow-neuron {NEURON_ID} --type upgrade-sns-to-next-version --unfollow"
    ))
    .diff("sns/follow/unfollow.txt");
}

#[test]
fn sns() {
    quill_query("sns list-deployed-snses").diff("sns/deployed.txt");
    quill_sns_query("sns status").diff("sns/status.txt");
}

#[test]
fn make_proposal() {
    let proposal = r#"
    ( record { 
        title = "SNS Launch";
        url = "https://dfinity.org";
        summary = "A motion to start the SNS";
        action = opt variant { Motion = record { 
            motion_text = "I hereby raise the motion that the use of the SNS shall commence"; 
        } };
    } )"#;
    quill_sns_send(&format!(
        "sns make-proposal {NEURON_ID} --proposal '{proposal}'"
    ))
    .diff("sns/make_proposal/simple.txt");
    let proposal_with_blob = r#"
    ( record {
        title = "Transfer ICP from SNS treasury.";
        url = "example.com";
        summary = "";
        action = opt variant { TransferSnsTreasuryFunds = record {
            from_treasury = 1 : int32;
            to_principal = opt principal "rrkah-fqaaa-aaaaa-aaaaq-cai";
            to_subaccount = opt record { subaccount = vec {0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;1} };
            memo = null;
            amount_e8s = 1_000_000_000: nat64
        } };
    } )"#;
    quill_sns_send(&format!(
        "sns make-proposal {NEURON_ID} --proposal '{proposal_with_blob}'"
    ))
    .diff("sns/make_proposal/with_blob.txt");
    let proposal_bin = asset("sns_proposal.bin");
    quill_sns_send(&format!(
        "sns make-proposal {NEURON_ID} --proposal-path '{proposal_bin}'",
    ))
    .diff("sns/make_proposal/from_file.txt");

    let canister_wasm = asset("sns_canister.wasm");
    quill_sns_send(&format!("sns make-upgrade-canister-proposal {NEURON_ID} --wasm-path '{canister_wasm}' --target-canister-id pycv5-3jbbb-ccccc-ddddd-cai"))
        .diff("sns/make_proposal/upgrade.txt");
    quill_sns_send(&format!("sns make-upgrade-canister-proposal {NEURON_ID} --wasm-path '{canister_wasm}'
            --canister-upgrade-arg '(record {{major=2:nat32; minor=3:nat32;}})' --target-canister-id pycv5-3jbbb-ccccc-ddddd-cai"))
        .diff("sns/make_proposal/upgrade_arg.txt");

    let upgrade_summary = asset("upgrade_summary.txt");
    quill_sns_send(&format!(
        "sns make-upgrade-canister-proposal {NEURON_ID} --wasm-path '{canister_wasm}' 
        --target-canister-id pycv5-3jbbb-ccccc-ddddd-cai --summary-path '{upgrade_summary}'"
    ))
    .diff("sns/make_proposal/upgrade_summary_path.txt");
}

#[test]
fn neuron_id() {
    quill_authed("sns neuron-id --memo 0").diff("sns/neuron_id/memo0.txt");
    quill("sns neuron-id --memo 0 --principal-id 44mwt-bq3um-tqicz-bwhad-iipx4-6wzex-olvaj-z63bj-wkelv-xoua3-rqe")
        .diff_s(b"SNS Neuron Id: 785d80b7abaf0d01fdadcef37ecd93cef68db3be7b2d66687bd1d64954c56c55");
}

#[test]
fn neuron_permission() {
    quill_sns_send(&format!("sns neuron-permission add {NEURON_ID} --principal {PRINCIPAL} --permissions submit-proposal,vote"))
        .diff("sns/neuron_permission/add.txt");
    quill_sns_send(&format!("sns neuron-permission remove {NEURON_ID} --principal {PRINCIPAL} --permissions merge-maturity disburse-maturity"))
        .diff("sns/neuron_permission/remove.txt");
}

#[test]
fn stake_neuron() {
    quill_sns_send("sns stake-neuron --amount 12 --memo 777").diff("sns/stake_neuron/memo.txt");
    quill_sns_send("sns stake-neuron --memo 777 --claim-only")
        .diff("sns/stake_neuron/no_amount.txt");
}

#[test]
fn swap() {
    quill_sns_send("sns new-sale-ticket --amount-icp-e8s 100000 --subaccount e000d80101")
        .diff("sns/swap/new_ticket.txt");
    quill_sns_send("sns pay --amount-icp-e8s 100000 --subaccount e000d80101 --ticket-creation-time 1676321340000 --ticket-id 100")
        .diff("sns/swap/pay.txt");
    quill_sns_send("sns pay --amount-icp-e8s 100000 --ticket-creation-time 1676321340000 --ticket-id 100 --confirmation-text 'testing!'")
        .diff("sns/swap/pay_with_confirmation_text.txt");
    quill_sns_send("sns get-swap-refund").diff("sns/swap/refund.txt");
    quill_sns_query_authed("sns get-sale-participation").diff("sns/swap/participation.txt");
}

#[test]
fn vote() {
    quill_sns_send(&format!(
        "sns register-vote {NEURON_ID} --proposal-id 1 --vote n"
    ))
    .diff("sns/vote/no.txt");
    quill_sns_send(&format!(
        "sns register-vote {NEURON_ID} --proposal-id 1 --vote y"
    ))
    .diff("sns/vote/yes.txt");
}

#[test]
fn transfer() {
    quill_sns_send(&format!("sns transfer {PRINCIPAL} --amount 0.000123"))
        .diff("sns/transfer/simple.txt");
    quill_sns_send(&format!(
        "sns transfer {PRINCIPAL} --amount 123.0456 --fee 0.0023 --memo 777"
    ))
    .diff("sns/transfer/fees_and_memo.txt");
}
