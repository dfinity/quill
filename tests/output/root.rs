use std::io::Write;

use tempfile::NamedTempFile;

use crate::{
    escape_p, ledger_compatible, quill, quill_authed, quill_query, quill_query_authed, quill_send,
    OutputExt,
};

// Uncomment tests on next ledger app update
ledger_compatible![
    account_balance,
    // claim_neurons,
    list_neurons,
    // neuron_stake,
    public_ids,
    transfer_icrc1,
];

#[test]
fn account_balance() {
    quill_query("account-balance ec0e2456fb9ff6c80f1d475b301d9b2ab873612f96e7fd74e7c0c0b2d58e6693")
        .diff("account_balance/simple.txt");

    quill_query_authed("account-balance").diff("account_balance/authed.txt");

    quill_query(
        "account-balance bz3ru-7uwvd-5yubs-mc75n-pbtpy-rz4bh-detlt-qmrls-sprg2-g7vmz-mqe-ce6fvoi.1",
    )
    .diff("account_balance/icrc1.txt")
}

#[test]
fn claim_neurons() {
    quill_send("claim-neurons").diff("claim_neurons/simple.txt");
}

#[test]
fn get_neuron_info() {
    quill_query("get-neuron-info 22174").diff("get_neuron_info/simple.txt");
}

#[test]
fn get_proposal_info() {
    quill_query("get-proposal-info 22174").diff("get_proposal_info/simple.txt");
}

#[test]
fn list_neurons() {
    quill_send("list-neurons").diff("list_neurons/simple.txt");
    quill_send("list-neurons 123 456 789").diff("list_neurons/many.txt");
}

#[test]
fn list_proposals() {
    quill_query("list-proposals").diff("list_proposals/simple.txt");
}

#[test]
fn neuron_stake() {
    quill_send("neuron-stake --amount 12 --from-subaccount 01 --nonce 777")
        .diff("neuron_stake/with_nonce.txt");
    quill_send("neuron-stake --amount 12 --name myNeuron").diff("neuron_stake/with_name.txt");
    quill_send("neuron-stake --name myNeuron --already-transferred")
        .diff("neuron_stake/stake_only.txt");
}

#[test]
fn generate() {
    let pem = NamedTempFile::new().unwrap();
    let seed = NamedTempFile::new().unwrap();
    quill(&format!(
        r#"generate --phrase "tornado allow zero warm have deer wool finish tiger ski dynamic strong"
             --seed-file {seed} --overwrite-seed-file --pem-file {pem} --overwrite-pem-file --storage-mode plaintext"#,
        seed = escape_p(&seed),
        pem = escape_p(&pem),
    )).assert_success();
    quill(&format!("public-ids --pem-file {}", escape_p(&pem))).diff_s(
        b"\
Principal id: beckf-r6bg7-t6ju6-s7k45-b5jtj-mcm57-zjaie-svgrr-7ekzs-55v75-sae
Legacy account id: ffc463646a2c92dce58d1179d26c64d4ccbaf1079a6edc5628cedc0d4b3b1866",
    );
    quill(&format!(
        "generate --pem-file {pem} --overwrite-pem-file",
        pem = escape_p(&pem)
    ))
    .diff_err("generate/no_password.txt");
    let mut good_password = NamedTempFile::new().unwrap();
    good_password
        .write_all(b"correct horse battery staple")
        .unwrap();
    let mut bad_password = NamedTempFile::new().unwrap();
    bad_password.write_all(b"hunter2").unwrap();
    quill(&format!(
        "generate --pem-file {pem} --overwrite-pem-file --password-file {good_password}",
        pem = escape_p(&pem),
        good_password = escape_p(&good_password)
    ))
    .assert_success();
    quill(&format!(
        "generate --pem-file {pem} --overwrite-pem-file --password-file {bad_password}",
        pem = escape_p(&pem),
        bad_password = escape_p(&bad_password)
    ))
    .diff_err("base_command/bad_password.txt");
    quill(&format!(
        "public-ids --pem-file {pem}",
        pem = escape_p(&pem)
    ))
    .diff_err("base_command/need_password.txt");
    quill(&format!(
        "public-ids --pem-file {pem} --password-file {pem}",
        pem = escape_p(&pem),
    ))
    .assert_err();
    quill(&format!(
        "public-ids --pem-file {pem} --password-file {pass}",
        pem = escape_p(&pem),
        pass = escape_p(&good_password)
    ))
    .assert_success();
}

#[test]
fn public_ids() {
    quill_authed("public-ids").diff("public_ids/basic.txt");
    quill_authed("public-ids --subaccount 010203").diff("public_ids/with_subaccount.txt");
    quill(
        "public-ids --principal-id 44mwt-bq3um-tqicz-bwhad-iipx4-6wzex-olvaj-z63bj-wkelv-xoua3-rqe",
    )
    .diff_s(
        b"\
Principal id: 44mwt-bq3um-tqicz-bwhad-iipx4-6wzex-olvaj-z63bj-wkelv-xoua3-rqe
Legacy account id: fe09de27b0fc2f9541f6e24ae41d0652aab116212dec7f75f0d502417539e6d4",
    );
    let mut seed = NamedTempFile::new().unwrap();
    seed.write_all(
        b"\
-----BEGIN EC PRIVATE KEY-----
MHQCAQEEIIb7zeOCb1+GCE3/zikHvfpNsLEyD/lD7dEDEdkV6Rs9oAcGBSuBBAAK
oUQDQgAEKmH3zLC+gmKTJr9MRsGpNWzXIAyHf/YUPoFymekUh/3KEEmj3Ut+RbrO
RJt7Z+jd+BKc8GEzTfxhNW2KtBkLdA==
-----END EC PRIVATE KEY-----",
    )
    .unwrap();
    quill(&format!(
        "public-ids --genesis-dfn --pem-file {}",
        escape_p(&seed)
    ))
    .diff_s(
        b"\
Principal id: 5lfph-rgykk-rizf2-kw7bi-52qsm-j44vi-2c7fu-kmpym-ohxe2-upuuw-oae
Legacy account id: e4f68df16f65a47e15acdd6089ca9f2a357b3c0f3b0a53f7bc060af350cb560f
DFN address: f452d283ff8381b9ca3cfa3cbc32e45177ea3ee6",
    );
}

#[test]
fn node_provider() {
    quill_send("replace-node-provider-id --node-operator-id fdsgv-62ihb-nbiqv-xgic5-iefsv-3cscz-tmbzv-63qd5-vh43v-dqfrt-pae --node-provider-id pnf55-r7gzn-s3oqn-ah2v7-r6b63-a2ma2-wyzhb-dzbwb-sghid-lzcxh-4ae")
        .diff("node_provider/replace.txt");
    quill_send("update-node-provider --reward-account ec0e2456fb9ff6c80f1d475b301d9b2ab873612f96e7fd74e7c0c0b2d58e6693")
        .diff("node_provider/update.txt");
}

#[test]
fn transfer() {
    quill_send("transfer 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 --amount 0.000123")
        .diff("transfer/simple.txt");
    quill_send("transfer 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 --amount 0.123456")
        .diff("transfer/e8s.txt");
    quill_send("transfer 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 --from-subaccount 01 --amount 0.0000000999999")
        .diff("transfer/e8s-2.txt");
    quill_send("transfer 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 --amount 1.23456")
        .diff("transfer/icp-and-e8s.txt");
    quill_send("transfer 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 --amount 123.0456 --fee 0.0023")
        .diff("transfer/with-fees.txt");
    quill_send("transfer 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 --amount 123.0456 --fee 0.0023 --memo 777")
        .diff("transfer/with-fees-and-memo.txt");
}

#[test]
fn transfer_icrc1() {
    quill_send("transfer bz3ru-7uwvd-5yubs-mc75n-pbtpy-rz4bh-detlt-qmrls-sprg2-g7vmz-mqe-ce6fvoi.1 --amount 12")
        .diff("transfer/icrc1.txt");
}

#[test]
fn ledger_fail_early() {
    quill("replace-node-provider-id --ledger --node-operator-id fdsgv-62ihb-nbiqv-xgic5-iefsv-3cscz-tmbzv-63qd5-vh43v-dqfrt-pae \
            --node-provider-id pnf55-r7gzn-s3oqn-ah2v7-r6b63-a2ma2-wyzhb-dzbwb-sghid-lzcxh-4ae")
        .diff_err("ledger_incompatible/by_function.txt");
    quill("neuron-stake --ledger --amount 12 --name myNeuron")
        .diff_err("ledger_incompatible/by_command.txt");
    quill("neuron-manage 1 --ledger --join-community-fund")
        .diff_err("ledger_incompatible/by_flag.txt");
}
