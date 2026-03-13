use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Agent {
    ClaudeCode,
    GeminiCli,
    CopilotCli,
}

/// Canonical tool names used in rules.toml
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalTool {
    Bash,
    Read,
    Write,
    Edit,
}

impl CanonicalTool {
    /// Resolve an agent-specific tool name to a canonical name.
    /// Returns None if the tool name is not recognized.
    pub fn from_agent_name(agent: Agent, name: &str) -> Option<Self> {
        match (agent, name) {
            (Agent::ClaudeCode, "Bash")
            | (Agent::GeminiCli, "run_shell_command")
            | (Agent::CopilotCli, "bash") => Some(Self::Bash),

            (Agent::ClaudeCode, "Read")
            | (Agent::GeminiCli, "read_file")
            | (Agent::CopilotCli, "view") => Some(Self::Read),

            (Agent::ClaudeCode, "Write")
            | (Agent::GeminiCli, "write_file")
            | (Agent::CopilotCli, "create") => Some(Self::Write),

            (Agent::ClaudeCode, "Edit")
            | (Agent::GeminiCli, "replace")
            | (Agent::CopilotCli, "edit") => Some(Self::Edit),

            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bash => "Bash",
            Self::Read => "Read",
            Self::Write => "Write",
            Self::Edit => "Edit",
        }
    }

    /// Infer the canonical field name for this tool.
    pub fn default_field(&self) -> &'static str {
        match self {
            Self::Bash => "command",
            Self::Read | Self::Write | Self::Edit => "file_path",
        }
    }
}

/// Expand matcher aliases. "File" expands to ["Read", "Write", "Edit"].
/// Non-alias names are returned as-is.
pub fn expand_matcher(matcher: &str) -> Vec<&str> {
    matcher
        .split('|')
        .flat_map(|name| match name {
            "File" => vec!["Read", "Write", "Edit"],
            other => vec![other],
        })
        .collect()
}

/// Resolve a canonical field name to the agent-specific field name in tool_input.
pub fn resolve_field(agent: Agent, canonical_field: &str) -> &str {
    match (agent, canonical_field) {
        (Agent::CopilotCli, "file_path") => "path",
        _ => canonical_field,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_tool_from_claude_code() {
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::ClaudeCode, "Bash"),
            Some(CanonicalTool::Bash)
        );
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::ClaudeCode, "Read"),
            Some(CanonicalTool::Read)
        );
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::ClaudeCode, "Write"),
            Some(CanonicalTool::Write)
        );
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::ClaudeCode, "Edit"),
            Some(CanonicalTool::Edit)
        );
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::ClaudeCode, "Unknown"),
            None
        );
    }

    #[test]
    fn canonical_tool_from_gemini_cli() {
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::GeminiCli, "run_shell_command"),
            Some(CanonicalTool::Bash)
        );
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::GeminiCli, "read_file"),
            Some(CanonicalTool::Read)
        );
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::GeminiCli, "write_file"),
            Some(CanonicalTool::Write)
        );
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::GeminiCli, "replace"),
            Some(CanonicalTool::Edit)
        );
    }

    #[test]
    fn canonical_tool_from_copilot_cli() {
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::CopilotCli, "bash"),
            Some(CanonicalTool::Bash)
        );
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::CopilotCli, "view"),
            Some(CanonicalTool::Read)
        );
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::CopilotCli, "create"),
            Some(CanonicalTool::Write)
        );
        assert_eq!(
            CanonicalTool::from_agent_name(Agent::CopilotCli, "edit"),
            Some(CanonicalTool::Edit)
        );
    }

    #[test]
    fn default_field_inference() {
        assert_eq!(CanonicalTool::Bash.default_field(), "command");
        assert_eq!(CanonicalTool::Read.default_field(), "file_path");
        assert_eq!(CanonicalTool::Write.default_field(), "file_path");
        assert_eq!(CanonicalTool::Edit.default_field(), "file_path");
    }

    #[test]
    fn resolve_field_copilot_file_path() {
        assert_eq!(resolve_field(Agent::CopilotCli, "file_path"), "path");
    }

    #[test]
    fn resolve_field_passthrough() {
        assert_eq!(resolve_field(Agent::ClaudeCode, "file_path"), "file_path");
        assert_eq!(resolve_field(Agent::GeminiCli, "file_path"), "file_path");
        assert_eq!(resolve_field(Agent::ClaudeCode, "command"), "command");
        assert_eq!(resolve_field(Agent::CopilotCli, "command"), "command");
    }

    #[test]
    fn expand_matcher_file_alias() {
        assert_eq!(expand_matcher("File"), vec!["Read", "Write", "Edit"]);
    }

    #[test]
    fn expand_matcher_mixed() {
        assert_eq!(
            expand_matcher("Bash|File"),
            vec!["Bash", "Read", "Write", "Edit"]
        );
    }

    #[test]
    fn expand_matcher_no_alias() {
        assert_eq!(expand_matcher("Bash"), vec!["Bash"]);
        assert_eq!(expand_matcher("Read|Write"), vec!["Read", "Write"]);
    }

    #[test]
    fn canonical_tool_as_str() {
        assert_eq!(CanonicalTool::Bash.as_str(), "Bash");
        assert_eq!(CanonicalTool::Read.as_str(), "Read");
        assert_eq!(CanonicalTool::Write.as_str(), "Write");
        assert_eq!(CanonicalTool::Edit.as_str(), "Edit");
    }
}
