use crate::{quill_send, OutputExt, ALICE, PRINCIPAL};

const NEURON_ID: &str = "2313380519530470538";

#[test]
fn hot_key() {
    quill_send(&format!("hotkey {NEURON_ID} --add {PRINCIPAL}"))
        .diff("neuron_manage/add_hot_key.txt");
    quill_send(&format!("hotkey {NEURON_ID} --remove {PRINCIPAL}"))
        .diff("neuron_manage/remove_hot_key.txt");
}

#[test]
fn additional_dissolve_delay_seconds() {
    quill_send(&format!("dissolve {NEURON_ID} --increase-delay-by 1h"))
        .diff("neuron_manage/additional_dissolve_delay_seconds.txt");
}

#[test]
fn disburse() {
    quill_send(&format!(
        "neuron-manage {NEURON_ID} --disburse-to {ALICE} --disburse-amount 57.31"
    ))
    .diff("neuron_manage/disburse_somewhat_to_someone.txt");
    quill_send(&format!(
        "neuron-manage {NEURON_ID} --disburse --stop-dissolving"
    ))
    .diff("neuron_manage/disburse_stop_dissolving.txt");
}

#[test]
fn follow() {
    quill_send(&format!("neuron-manage {NEURON_ID} --follow-topic 0 --follow-neurons 380519530470538 380519530470539"))
        .diff("neuron_manage/follow.txt");
    quill_send(&format!(
        "neuron-manage {NEURON_ID} --clear-manage-neuron-followees"
    ))
    .diff("neuron_manage/clear.txt");
}

#[test]
fn community_fund() {
    quill_send(&format!("neuron-manage {NEURON_ID} --join-community-fund"))
        .diff("neuron_manage/join_community_fund.txt");
    quill_send(&format!("neuron-manage {NEURON_ID} --leave-community-fund"))
        .diff("neuron_manage/leave_community_fund.txt");
}

#[test]
fn maturity() {
    quill_send(&format!("neuron-manage {NEURON_ID} --stake-maturity 100"))
        .diff("neuron_manage/stake_maturity.txt");

    quill_send(&format!(
        "neuron-manage {NEURON_ID} --auto-stake-maturity disabled"
    ))
    .diff("neuron_manage/auto_stake_disable.txt");
    quill_send(&format!(
        "neuron-manage {NEURON_ID} --auto-stake-maturity enabled"
    ))
    .diff("neuron_manage/auto_stake_enable.txt");

    quill_send("neuron-manage 65 --merge-maturity 100")
        .diff_err("neuron_manage/merge_maturity.txt");

    quill_send(&format!("neuron-manage {NEURON_ID} --spawn")).diff("neuron_manage/spawn.txt")
}

#[test]
fn merge() {
    quill_send(&format!(
        "neuron-manage {NEURON_ID} --merge-from-neuron 380519530470538"
    ))
    .diff("neuron_manage/merge.txt")
}

#[test]
fn split() {
    quill_send(&format!("neuron-manage {NEURON_ID} --split 100")).diff("neuron_manage/split.txt")
}

#[test]
fn vote() {
    quill_send(&format!(
        "neuron-manage {NEURON_ID} --register-vote 123 456"
    ))
    .diff("neuron_manage/vote.txt")
}