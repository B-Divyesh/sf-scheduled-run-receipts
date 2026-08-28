use std::{fs, process::Command};

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
