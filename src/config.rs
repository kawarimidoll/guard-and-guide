use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub version: u32,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub matcher: String,
    pub regex: String,
    pub message: String,
}

pub fn load(path: &Path) -> Result<Config, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let config: Config =
        toml::from_str(&content).map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_config() {
        let toml = r#"
version = 1

[[rules]]
matcher = "File"
regex = '\.env'
message = "Access to .env files is prohibited."

[[rules]]
matcher = "Bash"
regex = '\bgit\s+push\b'
message = "Do not execute 'git push'. Ask the user."
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].matcher, "File");
        assert_eq!(config.rules[0].regex, r"\.env");
        assert_eq!(config.rules[1].matcher, "Bash");
        assert_eq!(config.rules[1].regex, r"\bgit\s+push\b");
    }

    #[test]
    fn parse_empty_rules() {
        let toml = "version = 1\nrules = []\n";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.rules.len(), 0);
    }

    #[test]
    fn parse_missing_field() {
        let toml = r#"
version = 1

[[rules]]
matcher = "Bash"
regex = 'test'
"#;
        let err = toml::from_str::<Config>(toml).unwrap_err();
        assert!(err.to_string().contains("message"));
    }

    #[test]
    fn load_nonexistent_file() {
        let err = load(Path::new("/nonexistent/rules.toml")).unwrap_err();
        assert!(err.contains("failed to read"));
    }
}
