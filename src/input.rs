use serde::Deserialize;
use serde_json::Value;

use crate::agent::Agent;

/// Agent-independent representation of a hook input.
#[derive(Debug)]
pub struct HookInput {
    pub tool_name: String,
    pub tool_input: Value,
}

/// Claude Code / Gemini CLI share the same stdin structure.
#[derive(Deserialize)]
struct CommonInput {
    tool_name: String,
    tool_input: Value,
}

/// Copilot CLI uses camelCase and double-encoded toolArgs.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CopilotInput {
    tool_name: String,
    tool_args: String,
}

pub fn parse(agent: Agent, stdin: &str) -> Result<HookInput, String> {
    match agent {
        Agent::ClaudeCode | Agent::GeminiCli => {
            let input: CommonInput =
                serde_json::from_str(stdin).map_err(|e| format!("failed to parse stdin: {e}"))?;
            Ok(HookInput {
                tool_name: input.tool_name,
                tool_input: input.tool_input,
            })
        }
        Agent::CopilotCli => {
            let input: CopilotInput =
                serde_json::from_str(stdin).map_err(|e| format!("failed to parse stdin: {e}"))?;
            let tool_input: Value = serde_json::from_str(&input.tool_args)
                .map_err(|e| format!("failed to parse toolArgs: {e}"))?;
            Ok(HookInput {
                tool_name: input.tool_name,
                tool_input,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_claude_code() {
        let json = r#"{
            "session_id": "abc",
            "cwd": "/tmp",
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": { "command": "npm test" },
            "tool_use_id": "toolu_001"
        }"#;
        let input = parse(Agent::ClaudeCode, json).unwrap();
        assert_eq!(input.tool_name, "Bash");
        assert_eq!(input.tool_input["command"], "npm test");
    }

    #[test]
    fn parse_gemini_cli() {
        let json = r#"{
            "session_id": "test-123",
            "cwd": "/tmp",
            "hook_event_name": "BeforeTool",
            "tool_name": "write_file",
            "tool_input": { "file_path": "test.txt", "content": "hello" }
        }"#;
        let input = parse(Agent::GeminiCli, json).unwrap();
        assert_eq!(input.tool_name, "write_file");
        assert_eq!(input.tool_input["file_path"], "test.txt");
    }

    #[test]
    fn parse_copilot_cli() {
        let json = r#"{
            "timestamp": 1704614600000,
            "cwd": "/tmp",
            "toolName": "bash",
            "toolArgs": "{\"command\":\"rm -rf dist\"}"
        }"#;
        let input = parse(Agent::CopilotCli, json).unwrap();
        assert_eq!(input.tool_name, "bash");
        assert_eq!(input.tool_input["command"], "rm -rf dist");
    }

    #[test]
    fn parse_copilot_cli_invalid_tool_args() {
        let json = r#"{
            "timestamp": 1704614600000,
            "cwd": "/tmp",
            "toolName": "bash",
            "toolArgs": "not json"
        }"#;
        let err = parse(Agent::CopilotCli, json).unwrap_err();
        assert!(err.contains("failed to parse toolArgs"));
    }

    #[test]
    fn parse_invalid_json() {
        let err = parse(Agent::ClaudeCode, "not json").unwrap_err();
        assert!(err.contains("failed to parse stdin"));
    }
}
