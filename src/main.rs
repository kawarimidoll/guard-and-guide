pub mod agent;
pub mod config;
pub mod evaluator;
pub mod input;
pub mod output;

use std::io::Read;
use std::path::PathBuf;
use std::process;

use clap::Parser;

use agent::Agent;

fn version() -> &'static str {
    let v = env!("CARGO_PKG_VERSION");
    let hash = env!("GIT_HASH");
    if v == "0.0.0" && !hash.is_empty() {
        hash
    } else {
        v
    }
}

#[derive(Parser)]
#[command(version = version(), about)]
struct Cli {
    /// Agent type for input/output format
    #[arg(
        long,
        value_enum,
        default_value = "claude-code",
        long_help = "Agent type: claude-code, gemini-cli, copilot-cli"
    )]
    agent: Agent,

    /// Path to rules config file
    #[arg(long, default_value = "~/.config/guard-and-guide/rules.toml")]
    config: String,
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let config_path = expand_tilde(&cli.config);
    let config = config::load(&config_path)?;

    let mut stdin_buf = String::new();
    std::io::stdin()
        .read_to_string(&mut stdin_buf)
        .map_err(|e| format!("failed to read stdin: {e}"))?;

    let hook_input = input::parse(cli.agent, &stdin_buf)?;

    if let Some(result) = evaluator::evaluate(cli.agent, &config.rules, &hook_input) {
        print!("{}", output::format_deny(cli.agent, &result.message));
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("guard-and-guide: {e}");
        process::exit(1);
    }
}
