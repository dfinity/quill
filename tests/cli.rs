use std::{
    io::Read,
    process::{Command, Stdio},
};

#[test]
fn broken_pipe() {
    let mut command = Command::new(env!("CARGO_BIN_EXE_quill"));
    command
        .args([
            "transfer",
            "345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752",
            "--amount=123.0456",
        ])
        .stdout(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let mut pipe = child.stdout.take().unwrap();
    let _ = pipe.read(&mut [0; 16]).unwrap();
    drop(pipe);
    let status = child.wait().unwrap();
    assert!(status.success());
}
