use crate::{quill_authed, quill_query, quill_query_authed, quill_send, OutputExt, PRINCIPAL};

#[test]
fn balance() {
    quill_query_authed("ckbtc balance").diff("ckbtc/balance/authed.txt");
    quill_query_authed("ckbtc balance --testnet").diff("ckbtc/balance/testnet.txt");
}

#[test]
fn retrieve_btc() {
    quill_send("ckbtc retrieve-btc 3L2Uyh1eHpfPyPayqrh5WjfnTzWiG4xPLu --amount 3.14 --memo 9")
        .diff("ckbtc/retrieve_btc/simple.txt");
    quill_send(
        "ckbtc retrieve-btc 3L2Uyh1eHpfPyPayqrh5WjfnTzWiG4xPLu --already-transferred --amount 3.14",
    )
    .diff("ckbtc/retrieve_btc/already_transferred.txt");
    quill_query("ckbtc retrieve-btc-status 77").diff("ckbtc/retrieve_btc/status.txt");
}

#[test]
fn withdrawal_address() {
    quill_authed("ckbtc withdrawal-address")
        .diff_s(b"mqygn-kiaaa-aaaar-qaadq-cai-xvrx2hq.2b1ce8130386fc895735f19538973c109bf1de45749657b1039fefd4fd6bd50a");
}

#[test]
fn update_balance() {
    quill_send("ckbtc update-balance --subaccount bde9c3b148b84b82fdd86ec6f20d0c7b8809e54499f893cbca379dc535ea194b")
        .diff("ckbtc/update_balance/simple.txt");
    quill_send("ckbtc update-balance --subaccount bde9c3b148b84b82fdd86ec6f20d0c7b8809e54499f893cbca379dc535ea194b --testnet")
        .diff("ckbtc/update_balance/testnet.txt");
}

#[test]
fn transfer() {
    quill_send(&format!(
        "ckbtc transfer {PRINCIPAL} --memo 3 --amount 3.14"
    ))
    .diff("ckbtc/transfer/simple.txt");
}
