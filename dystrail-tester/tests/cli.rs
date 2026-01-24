use std::process::Command;

fn temp_path(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "dystrail-cli-{label}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

#[test]
fn cli_list_scenarios_writes_output() {
    let exe = env!("CARGO_BIN_EXE_dystrail-tester");
    let output_path = temp_path("list");
    let status = Command::new(exe)
        .args(["--list-scenarios", "--output"])
        .arg(&output_path)
        .status()
        .expect("run cli");
    assert!(status.success());
    let content = std::fs::read_to_string(output_path).expect("read output");
    assert!(content.contains("Available scenarios"));
}

#[test]
fn cli_runs_with_unknown_browser_and_json_report() {
    let exe = env!("CARGO_BIN_EXE_dystrail-tester");
    let output_path = temp_path("run");
    let output = Command::new(exe)
        .args([
            "--mode",
            "browser",
            "--browsers",
            "unknown",
            "--report",
            "json",
            "--scenarios",
            "smoke",
            "--iterations",
            "1",
            "--seeds",
            "1",
            "--output",
        ])
        .arg(&output_path)
        .output()
        .expect("run cli");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("Dystrail Automated Tester") || stderr.contains("Unknown browser"));
}
