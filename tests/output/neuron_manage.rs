use crate::{ledger_compatible, quill_send, OutputExt, ALICE, PRINCIPAL};

const NEURON_ID: &str = "2313380519530470538";

// uncomment tests on next ledger app update
ledger_compatible![
    // hot_key,
    dissolve_delay,
    // disburse,
    dissolve,
    // follow,
    // community_fund,
    maturity,
    merge,
    split,
    // vote
];

#[test]
fn hot_key() {
    quill_send(&format!("hotkey {NEURON_ID} --add {PRINCIPAL}"))
        .diff("neuron_manage/add_hot_key.txt");
    quill_send(&format!("hotkey {NEURON_ID} --remove {PRINCIPAL}"))
        .diff("neuron_manage/remove_hot_key.txt");
}

#[test]
fn dissolve_delay() {
    quill_send(&format!("dissolve-delay {NEURON_ID} --increase-by 1h"))
        .diff("neuron_manage/additional_dissolve_delay_seconds.txt");
    quill_send(&format!(
        "dissolve-delay {NEURON_ID} --increase-to '10 days'"
    ))
    .diff("neuron_manage/dissolve_fixed_timestamp.txt");
}

#[test]
fn disburse() {
    quill_send(&format!("disburse {NEURON_ID} --to {ALICE} --amount 57.31"))
        .diff("neuron_manage/disburse_somewhat_to_someone.txt");
    quill_send(&format!("disburse {NEURON_ID}")).diff("neuron_manage/disburse.txt");
}

#[test]
fn dissolve() {
    quill_send(&format!("dissolve {NEURON_ID} --start")).diff("neuron_manage/start_dissolving.txt");
    quill_send(&format!("dissolve {NEURON_ID} --stop")).diff("neuron_manage/stop_dissolving.txt");
}

#[test]
fn follow() {
    quill_send(&format!(
        "follow {NEURON_ID} --topic-id 0 --followees 380519530470538,380519530470539"
    ))
    .diff("neuron_manage/follow.txt");
    quill_send(&format!(
        "follow {NEURON_ID} --unfollow --type neuron-management"
    ))
    .diff("neuron_manage/clear.txt");
}

#[test]
fn community_fund() {
    quill_send(&format!("community-fund {NEURON_ID} --join"))
        .diff("neuron_manage/join_community_fund.txt");
    quill_send(&format!("community-fund {NEURON_ID} --leave"))
        .diff("neuron_manage/leave_community_fund.txt");
}

#[test]
fn maturity() {
    quill_send(&format!("stake-maturity {NEURON_ID} --percentage 100"))
        .diff("neuron_manage/stake_maturity.txt");

    quill_send(&format!("stake-maturity {NEURON_ID} --disable-automatic"))
        .diff("neuron_manage/auto_stake_disable.txt");
    quill_send(&format!("stake-maturity {NEURON_ID} --automatic"))
        .diff("neuron_manage/auto_stake_enable.txt");

    quill_send("neuron-manage 65 --merge-maturity 100")
        .diff_err("neuron_manage/merge_maturity.txt");

    quill_send(&format!("spawn {NEURON_ID}")).diff("neuron_manage/spawn.txt");
    quill_send(&format!("spawn {NEURON_ID} --to {ALICE} --percentage 20"))
        .diff("neuron_manage/spawn_to.txt");
}

#[test]
fn merge() {
    quill_send(&format!("merge {NEURON_ID} --from 380519530470538")).diff("neuron_manage/merge.txt")
}

#[test]
fn split() {
    quill_send(&format!("split {NEURON_ID} --amount 100")).diff("neuron_manage/split.txt")
}

#[test]
fn vote() {
    quill_send(&format!("vote {NEURON_ID} --approve --proposal-id 123"))
        .diff("neuron_manage/vote.txt")
}
