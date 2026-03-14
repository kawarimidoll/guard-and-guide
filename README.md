<div align="center">

# guard-and-guide

[![CI](https://github.com/kawarimidoll/guard-and-guide/actions/workflows/ci.yml/badge.svg)](https://github.com/kawarimidoll/guard-and-guide/actions/workflows/ci.yml)
[![Latest version](https://img.shields.io/github/v/release/kawarimidoll/guard-and-guide)](https://github.com/kawarimidoll/guard-and-guide/releases/latest)
[![Claude Code](https://img.shields.io/badge/Claude_Code-black?logo=anthropic)](https://docs.anthropic.com/en/docs/claude-code)
[![Gemini CLI](https://img.shields.io/badge/Gemini_CLI-black?logo=googlegemini)](https://github.com/google-gemini/gemini-cli)

🛡️ Guard coding agents from dangerous operations, and 🧭 Guide them to safer alternatives.

</div>

## Motivation

You tell your AI coding agent: "Don't run `git push`."

The agent gets blocked — and tries harder:

```
🧑 deny(git push)
🤖 Got it! I'll use /usr/bin/git push instead.
🧑 NOOO!!!
```

AI agents don't give up when simply denied. They try absolute paths, alternative commands, and creative workarounds to get the job done.

**guard-and-guide** solves this by not only blocking dangerous operations, but also telling the agent _why_ it was blocked and _what to do instead_. When given clear guidance, agents are much more likely to comply.

## Install

### nix

```sh
nix profile install github:kawarimidoll/guard-and-guide
```

### cargo

```sh
cargo install --git https://github.com/kawarimidoll/guard-and-guide
```

## Setup

### 1. Create rules

Create `~/.config/guard-and-guide/rules.toml` with your rules.
See [`rules.example.toml`](rules.example.toml) for a full example.

Rules use canonical tool names (`Bash`, `File`) and regex patterns:

```toml
version = 1

[[rules]]
matcher = "File"
regex = '\.env$'
message = "Access to .env files is prohibited. Ask the user to check or provide the values you need."

[[rules]]
matcher = "Bash"
regex = '\bgit\s+push\b'
message = "Use of 'git push' is prohibited. Ask the user to execute it."
```

`File` is an alias for `Read|Write|Edit`.

### 2. Register hook

#### Claude Code

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "guard-and-guide" }]
      }
    ]
  }
}
```

#### Gemini CLI

Add to `~/.gemini/settings.json`:

```json
{
  "hooks": {
    "BeforeTool": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "guard-and-guide --agent gemini-cli" }]
      }
    ]
  }
}
```

> Copilot CLI support is planned.

## Usage

```
stdin (hook JSON) | guard-and-guide [OPTIONS]
```

| Option | Description |
|---|---|
| `--agent <AGENT>` | `claude-code` (default), `gemini-cli` |
| `--config <PATH>` | Rules file path (default: `~/.config/guard-and-guide/rules.toml`) |

## How it works

```
Hook stdin → Parse JSON → Resolve tool name → Match rules → Deny or pass
```

1. Agent hook pipes JSON to stdin
2. Parse and extract tool name + input (format depends on `--agent`)
3. Resolve agent-specific tool name to canonical name (`Bash`, `Read`, `Write`, `Edit`)
4. Check each rule: does the canonical name match? Does the regex match the relevant field?
5. First match → output deny JSON to stdout. No match → silent exit 0.

### Tool name mapping

| Canonical | Claude Code | Gemini CLI |
|---|---|---|
| `Bash` | `Bash` | `run_shell_command` |
| `Read` | `Read` | `read_file` |
| `Write` | `Write` | `write_file` |
| `Edit` | `Edit` | `replace` |

## License

MIT
