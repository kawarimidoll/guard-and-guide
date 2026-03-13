use std::io::Write;
use std::process::{Command, Stdio};

fn run(agent: &str, config: &str, stdin_data: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_guard-and-guide"))
        .args(["--agent", agent, "--config", config])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin_data.as_bytes())
        .unwrap();

    child.wait_with_output().unwrap()
}

fn rules_path() -> String {
    format!("{}/tests/fixtures/rules.toml", env!("CARGO_MANIFEST_DIR"))
}

// --- Claude Code ---

#[test]
fn claude_code_deny_file() {
    let output = run(
        "claude-code",
        &rules_path(),
        r#"{"tool_name":"Read","tool_input":{"file_path":"/home/.env"}}"#,
    );

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["hookSpecificOutput"]["permissionDecision"], "deny");
    assert!(
        parsed["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .unwrap()
            .contains(".env")
    );
}

#[test]
fn claude_code_deny_bash() {
    let output = run(
        "claude-code",
        &rules_path(),
        r#"{"tool_name":"Bash","tool_input":{"command":"sed -i s/foo/bar/ file.txt"}}"#,
    );

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["hookSpecificOutput"]["permissionDecision"], "deny");
    assert!(
        parsed["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .unwrap()
            .contains("sed")
    );
}

#[test]
fn claude_code_allow() {
    let output = run(
        "claude-code",
        &rules_path(),
        r#"{"tool_name":"Read","tool_input":{"file_path":"/home/readme.md"}}"#,
    );

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn claude_code_unknown_tool_passes() {
    let output = run(
        "claude-code",
        &rules_path(),
        r#"{"tool_name":"Agent","tool_input":{"prompt":"hello"}}"#,
    );

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

// --- Gemini CLI ---

#[test]
fn gemini_cli_deny() {
    let output = run(
        "gemini-cli",
        &rules_path(),
        r#"{"tool_name":"write_file","tool_input":{"file_path":".env.local","content":"x"}}"#,
    );

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["decision"], "deny");
    assert!(parsed["reason"].as_str().unwrap().contains(".env"));
}

#[test]
fn gemini_cli_allow() {
    let output = run(
        "gemini-cli",
        &rules_path(),
        r#"{"tool_name":"read_file","tool_input":{"file_path":"src/main.rs"}}"#,
    );

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

// --- Copilot CLI ---

#[test]
fn copilot_cli_deny() {
    let output = run(
        "copilot-cli",
        &rules_path(),
        r#"{"toolName":"create","toolArgs":"{\"path\":\".env\",\"content\":\"SECRET=x\"}"}"#,
    );

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["permissionDecision"], "deny");
    assert!(
        parsed["permissionDecisionReason"]
            .as_str()
            .unwrap()
            .contains(".env")
    );
}

#[test]
fn copilot_cli_allow() {
    let output = run(
        "copilot-cli",
        &rules_path(),
        r#"{"toolName":"bash","toolArgs":"{\"command\":\"ls -la\"}"}"#,
    );

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

// --- Error cases ---

#[test]
fn invalid_json_exits_with_error() {
    let output = run("claude-code", &rules_path(), "not json");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("guard-and-guide"));
}

#[test]
fn missing_config_exits_with_error() {
    let output = run(
        "claude-code",
        "/nonexistent/rules.toml",
        r#"{"tool_name":"Read","tool_input":{"file_path":"x"}}"#,
    );

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to read"));
}
