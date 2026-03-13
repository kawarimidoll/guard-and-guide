use serde_json::json;

use crate::agent::Agent;

/// Format the deny response JSON for the given agent.
pub fn format_deny(agent: Agent, message: &str) -> String {
    let json = match agent {
        Agent::ClaudeCode => json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": message,
            }
        }),
        Agent::GeminiCli => json!({
            "decision": "deny",
            "reason": message,
        }),
        Agent::CopilotCli => json!({
            "permissionDecision": "deny",
            "permissionDecisionReason": message,
        }),
    };
    serde_json::to_string(&json).expect("failed to serialize JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_code_output() {
        let output = format_deny(Agent::ClaudeCode, "blocked");
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["hookSpecificOutput"]["permissionDecision"], "deny");
        assert_eq!(
            parsed["hookSpecificOutput"]["permissionDecisionReason"],
            "blocked"
        );
        assert_eq!(parsed["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    }

    #[test]
    fn gemini_cli_output() {
        let output = format_deny(Agent::GeminiCli, "blocked");
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["decision"], "deny");
        assert_eq!(parsed["reason"], "blocked");
    }

    #[test]
    fn copilot_cli_output() {
        let output = format_deny(Agent::CopilotCli, "blocked");
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["permissionDecision"], "deny");
        assert_eq!(parsed["permissionDecisionReason"], "blocked");
    }
}
