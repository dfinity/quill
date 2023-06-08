use std::{
    env, fs,
    path::Path,
    process::{Command, Output, Stdio},
};

mod ckbtc;
mod neuron_manage;
mod root;
mod sns;

const PRINCIPAL: &str = "fdsgv-62ihb-nbiqv-xgic5-iefsv-3cscz-tmbzv-63qd5-vh43v-dqfrt-pae";
const ALICE: &str = "pnf55-r7gzn-s3oqn-ah2v7-r6b63-a2ma2-wyzhb-dzbwb-sghid-lzcxh-4ae";
#[allow(unused)]
const BOB: &str = "jndu2-vwnnt-bpu6t-2jrke-fg3kj-vbrgf-ajecf-gv6ju-onyol-wc3e5-kqe";

fn quill_path() -> &'static str {
    env!("CARGO_BIN_EXE_quill")
}

fn quill_command() -> Command {
    let mut cmd = Command::new(quill_path());
    cmd.env("QUILL_TEST_FIXED_TIMESTAMP", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    cmd
}

#[must_use]
fn quill(cmd: &str) -> Output {
    quill_inner(cmd, [])
}

#[must_use]
fn quill_authed(cmd: &str) -> Output {
    quill_inner(cmd, auth_args())
}

fn quill_inner(cmd: &str, args: impl IntoIterator<Item = String>) -> Output {
    quill_command()
        .args(shellwords::split(cmd).unwrap())
        .args(args)
        .output()
        .unwrap()
}

#[must_use]
fn quill_query(cmd: &str) -> Output {
    quill_inner(cmd, ["--dry-run".into()])
}

#[must_use]
fn quill_sns_query(cmd: &str) -> Output {
    quill_inner(cmd, ["--dry-run".into()].into_iter().chain(sns_args()))
}

#[must_use]
fn quill_query_authed(cmd: &str) -> Output {
    quill_inner(cmd, ["--dry-run".into()].into_iter().chain(auth_args()))
}

#[must_use]
fn quill_sns_query_authed(cmd: &str) -> Output {
    quill_inner(
        cmd,
        ["--dry-run".into()]
            .into_iter()
            .chain(auth_args())
            .chain(sns_args()),
    )
}

fn quill_send_inner(cmd: &str, args: impl IntoIterator<Item = String>) -> Output {
    let mut quill = quill_command()
        .args(shellwords::split(cmd).unwrap())
        .args(args)
        .spawn()
        .unwrap();
    let mut out = quill_command()
        .args(["send", "--dry-run", "-y"])
        .stdin(quill.stdout.take().unwrap())
        .output()
        .unwrap();
    let quill = quill.wait_with_output().unwrap();
    out.stderr = quill.stderr;
    out
}

#[must_use]
fn quill_send(cmd: &str) -> Output {
    quill_send_inner(cmd, auth_args())
}

#[must_use]
fn quill_sns_send(cmd: &str) -> Output {
    quill_send_inner(cmd, auth_args().into_iter().chain(sns_args()))
}

fn default_pem() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/e2e/assets/identity.pem")
}

fn asset(asset: &str) -> String {
    format!("{}/e2e/assets/{asset}", env!("CARGO_MANIFEST_DIR"))
}

fn auth_args() -> Vec<String> {
    vec!["--pem-file".into(), default_pem().into()]
}

fn sns_args() -> Vec<String> {
    vec!["--canister-ids-file".into(), asset("sns_canister_ids.json")]
}

fn escape_p(p: &impl AsRef<Path>) -> String {
    shellwords::escape(p.as_ref().to_str().unwrap())
}

trait OutputExt {
    fn assert_success(&self);
    fn assert_err(&self);
    fn diff(&self, output_file: &str);
    fn diff_s(&self, output: &[u8]);
    fn diff_err(&self, output_file: &str);
}

impl OutputExt for Output {
    #[track_caller]
    fn assert_success(&self) {
        if !self.status.success() {
            panic!(
                "
Command exited unsuccesfully!

Stdout:
{}

Stderr:
{}",
                String::from_utf8_lossy(&self.stdout),
                String::from_utf8_lossy(&self.stderr),
            )
        }
    }
    #[track_caller]
    fn assert_err(&self) {
        if self.status.success() {
            panic!(
                "
Command exited successfully (should have been unsuccessful)

Stdout:
{}

Stderr:
{}",
                String::from_utf8_lossy(&self.stdout),
                String::from_utf8_lossy(&self.stderr),
            )
        }
    }
    #[track_caller]
    fn diff(&self, output_file: &str) {
        let output_file = format!(
            "{}/tests/output-tests/outputs/{output_file}",
            env!("CARGO_MANIFEST_DIR")
        );
        if env::var("FIX_OUTPUTS").is_ok() {
            self.assert_success();
            fs::write(output_file, &self.stdout).unwrap();
        } else {
            let output = std::fs::read(output_file).unwrap();
            self.diff_s(&output);
        }
    }
    #[track_caller]
    fn diff_s(&self, output: &[u8]) {
        self.assert_success();
        if !output_eq(&self.stdout, output) {
            panic!(
                "
Expected output:
{}

Generated output:
{}",
                String::from_utf8_lossy(output),
                String::from_utf8_lossy(&self.stdout),
            )
        }
    }
    #[track_caller]
    fn diff_err(&self, output_file: &str) {
        self.assert_err();
        let output_file = format!(
            "{}/tests/output-tests/outputs/{output_file}",
            env!("CARGO_MANIFEST_DIR")
        );
        if env::var("FIX_OUTPUTS").is_ok() {
            std::fs::write(output_file, &self.stderr).unwrap();
        } else {
            let output = std::fs::read(output_file).unwrap();
            if !output_eq(&self.stderr, &output) {
                panic!(
                    "\
Expected ouptut:
{}

Generated output:
{}",
                    String::from_utf8_lossy(&output),
                    String::from_utf8_lossy(&self.stderr),
                )
            }
        }
    }
}

fn output_eq(a: &[u8], b: &[u8]) -> bool {
    let a = trim(a).iter().filter(|&&x| x != b'\r');
    let b = trim(b).iter().filter(|&&x| x != b'\r');
    a.eq(b)
}

fn trim(s: &[u8]) -> &[u8] {
    let Some(start) = s.iter().position(|x| !b" \r\n\t".contains(x)) else { return &[] };
    let end = s.iter().rposition(|x| !b" \r\n\t".contains(x)).unwrap();
    &s[start..=end]
}
