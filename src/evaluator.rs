use regex::Regex;

use crate::agent::{Agent, CanonicalTool, expand_matcher, resolve_field};
use crate::config::Rule;
use crate::input::HookInput;

/// Result of evaluating rules against a hook input.
pub struct EvalResult {
    pub message: String,
}

/// Evaluate rules against a hook input. Returns the first matching rule's message.
pub fn evaluate(agent: Agent, rules: &[Rule], input: &HookInput) -> Option<EvalResult> {
    let canonical_name = CanonicalTool::from_agent_name(agent, &input.tool_name)?;

    for rule in rules {
        let matcher_tools = expand_matcher(&rule.matcher);
        if !matcher_tools.contains(&canonical_name.as_str()) {
            continue;
        }

        let canonical_field = canonical_name.default_field();
        let agent_field = resolve_field(agent, canonical_field);

        let value = input.tool_input.get(agent_field).and_then(|v| v.as_str())?;

        let re = Regex::new(&rule.regex).ok()?;
        if re.is_match(value) {
            return Some(EvalResult {
                message: rule.message.clone(),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_input(tool_name: &str, tool_input: serde_json::Value) -> HookInput {
        HookInput {
            tool_name: tool_name.to_string(),
            tool_input,
        }
    }

    fn make_rule(matcher: &str, regex: &str, message: &str) -> Rule {
        Rule {
            matcher: matcher.to_string(),
            regex: regex.to_string(),
            message: message.to_string(),
        }
    }

    #[test]
    fn match_file_alias() {
        let rules = vec![make_rule("File", r"\.env", "blocked .env")];
        let input = make_input("Read", json!({"file_path": "/home/.env"}));
        let result = evaluate(Agent::ClaudeCode, &rules, &input).unwrap();
        assert_eq!(result.message, "blocked .env");
    }

    #[test]
    fn match_bash_command() {
        let rules = vec![make_rule("Bash", r"\bgit\s+push\b", "no push")];
        let input = make_input("Bash", json!({"command": "git push origin main"}));
        let result = evaluate(Agent::ClaudeCode, &rules, &input).unwrap();
        assert_eq!(result.message, "no push");
    }

    #[test]
    fn no_match() {
        let rules = vec![make_rule("File", r"\.env", "blocked")];
        let input = make_input("Read", json!({"file_path": "/home/readme.md"}));
        assert!(evaluate(Agent::ClaudeCode, &rules, &input).is_none());
    }

    #[test]
    fn unknown_tool_skipped() {
        let rules = vec![make_rule("File", r"\.env", "blocked")];
        let input = make_input("Agent", json!({"prompt": "hello"}));
        assert!(evaluate(Agent::ClaudeCode, &rules, &input).is_none());
    }

    #[test]
    fn first_match_wins() {
        let rules = vec![
            make_rule("File", r"\.env", "first"),
            make_rule("File", r"\.env", "second"),
        ];
        let input = make_input("Write", json!({"file_path": ".env.local"}));
        let result = evaluate(Agent::ClaudeCode, &rules, &input).unwrap();
        assert_eq!(result.message, "first");
    }

    #[test]
    fn gemini_cli_tool_resolution() {
        let rules = vec![make_rule("File", r"\.ssh/", "no ssh")];
        let input = make_input("write_file", json!({"file_path": "/home/.ssh/id_rsa"}));
        let result = evaluate(Agent::GeminiCli, &rules, &input).unwrap();
        assert_eq!(result.message, "no ssh");
    }

    #[test]
    fn copilot_cli_field_resolution() {
        let rules = vec![make_rule("File", r"\.env", "blocked")];
        let input = make_input("create", json!({"path": "/app/.env"}));
        let result = evaluate(Agent::CopilotCli, &rules, &input).unwrap();
        assert_eq!(result.message, "blocked");
    }

    #[test]
    fn matcher_pipe_separation() {
        let rules = vec![make_rule("Read|Write", r"secret", "blocked")];
        let input = make_input("Write", json!({"file_path": "secret.txt"}));
        let result = evaluate(Agent::ClaudeCode, &rules, &input).unwrap();
        assert_eq!(result.message, "blocked");
    }

    #[test]
    fn matcher_no_match_different_tool() {
        let rules = vec![make_rule("Bash", r"\.env", "blocked")];
        let input = make_input("Read", json!({"file_path": ".env"}));
        assert!(evaluate(Agent::ClaudeCode, &rules, &input).is_none());
    }
}
