use std::{
    fs,
    process::{Command, Stdio},
};

#[test]
fn documented_local_workflow_runs_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("state.json");
    let bin = env!("CARGO_BIN_EXE_srr");
    let call = |args: &[&str]| {
        Command::new(bin)
            .arg("--data")
            .arg(&data)
            .args(args)
            .output()
            .unwrap()
    };

    assert!(call(&["init"]).status.success());
    assert!(
        call(&[
            "job",
            "add",
            "every-minute",
            "--schedule",
            "* * * * *",
            "--grace",
            "15m"
        ])
        .status
        .success()
    );
    assert!(
        call(&["run", "start", "every-minute", "--run-id", "example-1"])
            .status
            .success()
    );
    assert!(
        call(&[
            "run",
            "finish",
            "every-minute",
            "--run-id",
            "example-1",
            "--status",
            "success"
        ])
        .status
        .success()
    );
    let list = call(&["job", "list", "--json"]);
    assert!(list.status.success());
    assert!(
        String::from_utf8(list.stdout)
            .unwrap()
            .contains("every-minute")
    );
    let report = dir.path().join("report.html");
    assert!(
        Command::new(bin)
            .arg("--data")
            .arg(&data)
            .args(["export", "--output"])
            .arg(&report)
            .status()
            .unwrap()
            .success()
    );
    assert!(fs::read_to_string(report).unwrap().contains("Run ledger"));
}

#[test]
fn help_is_actionable() {
    let output = Command::new(env!("CARGO_BIN_EXE_srr"))
        .arg("--help")
        .output()
        .unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("notice when they did not"));
    assert!(text.contains("receipt"));
    assert!(text.contains("export"));
}

#[test]
fn demo_writes_disposable_sample_evidence() {
    let output = Command::new(env!("CARGO_BIN_EXE_srr"))
        .arg("demo")
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Demo ledger:"));
    assert!(text.contains("Demo weekly evidence:"));
    assert!(text.contains("Sample findings:"));
}

#[test]
fn concurrent_receipt_writers_preserve_every_accepted_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("state.json");
    let bin = env!("CARGO_BIN_EXE_srr");
    let call = |args: &[&str]| {
        Command::new(bin)
            .arg("--data")
            .arg(&data)
            .args(args)
            .output()
            .unwrap()
    };

    assert!(call(&["init"]).status.success());
    assert!(
        call(&[
            "job",
            "add",
            "concurrent",
            "--schedule",
            "* * * * *",
            "--grace",
            "1m",
        ])
        .status
        .success()
    );
    let scheduled_at = chrono::Utc::now().to_rfc3339();

    let children: Vec<_> = (1..=20)
        .map(|index| {
            let mut command = Command::new(bin);
            command
                .arg("--data")
                .arg(&data)
                .args(["run", "start", "concurrent", "--run-id"])
                .arg(format!("run-{index}"))
                .args(["--scheduled-at", &scheduled_at])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            command.spawn().unwrap()
        })
        .collect();

    for child in children {
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "writer failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("Accepted Start receipt"));
    }

    let state: serde_json::Value = serde_json::from_slice(&fs::read(data).unwrap()).unwrap();
    assert_eq!(state["receipts"].as_array().unwrap().len(), 20);
    assert_eq!(state["seen_nonces"].as_array().unwrap().len(), 20);
}

#[test]
fn extreme_durations_return_input_errors_without_panicking() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("state.json");
    let bin = env!("CARGO_BIN_EXE_srr");
    assert!(
        Command::new(bin)
            .arg("--data")
            .arg(&data)
            .arg("init")
            .status()
            .unwrap()
            .success()
    );

    for duration in ["9223372036854775807s", "9223372036854775807d"] {
        let output = Command::new(bin)
            .arg("--data")
            .arg(&data)
            .args([
                "job",
                "add",
                "absurd",
                "--schedule",
                "* * * * *",
                "--grace",
                duration,
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("duration is too large"));
    }
    let state: serde_json::Value = serde_json::from_slice(&fs::read(data).unwrap()).unwrap();
    assert!(state["jobs"].as_object().unwrap().is_empty());
}
