use std::ffi::OsString;

use cow_utils::CowUtils;

/// Information about the AI coding agent detected from the environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentInfo {
    Claude,
    Replit,
    Gemini,
    Codex,
    Copilot,
    OpenCode,
    Pi,
    Devin,
    Cursor,
    Kiro,
    Other(String),
}

/// Detect the current AI coding agent from environment variables.
///
/// This mirrors the environment-based detection used by `std-env` while keeping
/// the logic reusable for Rust CLI entry points.
pub fn detect_agent() -> Option<AgentInfo> {
    detect_agent_from_env(|key| std::env::var_os(key))
}

/// Return `true` when the current process appears to be running inside an AI
/// coding agent.
pub fn is_agent() -> bool {
    detect_agent().is_some()
}

fn detect_agent_from_env(env: impl Fn(&str) -> Option<OsString>) -> Option<AgentInfo> {
    if let Some(ai_agent) = env_string(&env, "AI_AGENT") {
        return Some(AgentInfo::Other(ai_agent.cow_to_ascii_lowercase().into_owned()));
    }

    if has_any_env(&env, &["CLAUDECODE", "CLAUDE_CODE"]) {
        Some(AgentInfo::Claude)
    } else if has_any_env(&env, &["REPL_ID"]) {
        Some(AgentInfo::Replit)
    } else if has_any_env(&env, &["GEMINI_CLI"]) {
        Some(AgentInfo::Gemini)
    } else if has_any_env(&env, &["CODEX_SANDBOX", "CODEX_THREAD_ID"]) {
        Some(AgentInfo::Codex)
    } else if has_any_env(&env, &["COPILOT_CLI"]) {
        Some(AgentInfo::Copilot)
    } else if has_any_env(&env, &["OPENCODE"]) {
        Some(AgentInfo::OpenCode)
    } else if env_contains_path_segment(&env, "PATH", ".pi/agent") {
        Some(AgentInfo::Pi)
    } else if env_contains(&env, "EDITOR", "devin") {
        Some(AgentInfo::Devin)
    } else if has_any_env(&env, &["CURSOR_AGENT"]) {
        Some(AgentInfo::Cursor)
    } else if env_contains(&env, "TERM_PROGRAM", "kiro") {
        Some(AgentInfo::Kiro)
    } else {
        None
    }
}

fn has_any_env(env: &impl Fn(&str) -> Option<OsString>, keys: &[&str]) -> bool {
    keys.iter().any(|key| env(key).is_some_and(|value| !value.is_empty()))
}

fn env_string(env: &impl Fn(&str) -> Option<OsString>, key: &str) -> Option<String> {
    env(key).filter(|value| !value.is_empty()).map(|value| value.to_string_lossy().into_owned())
}

fn env_contains(env: &impl Fn(&str) -> Option<OsString>, key: &str, needle: &str) -> bool {
    env_string(env, key).is_some_and(|value| value.contains(needle))
}

fn env_contains_path_segment(
    env: &impl Fn(&str) -> Option<OsString>,
    key: &str,
    needle: &str,
) -> bool {
    env_string(env, key).is_some_and(|value| {
        let windows_needle = needle.cow_replace('/', "\\");
        value.contains(needle) || value.contains(windows_needle.as_ref())
    })
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use rustc_hash::FxHashMap;

    use super::{AgentInfo, detect_agent_from_env};

    fn detect(vars: &[(&str, &str)]) -> Option<AgentInfo> {
        let env = vars
            .iter()
            .map(|(key, value)| ((*key).to_string(), OsString::from(value)))
            .collect::<FxHashMap<_, _>>();

        detect_agent_from_env(|key| env.get(key).cloned())
    }

    #[test]
    fn detects_explicit_ai_agent() {
        assert_eq!(
            detect(&[("AI_AGENT", "CustomAgent")]),
            Some(AgentInfo::Other("customagent".into()))
        );
    }

    #[test]
    fn explicit_ai_agent_takes_precedence() {
        assert_eq!(
            detect(&[("AI_AGENT", "custom"), ("CLAUDECODE", "1")]),
            Some(AgentInfo::Other("custom".into()))
        );
    }

    #[test]
    fn ignores_empty_env_values() {
        assert_eq!(detect(&[("AI_AGENT", ""), ("CLAUDECODE", "")]), None);
    }

    #[test]
    fn detects_agent_env_vars() {
        let cases = [
            (AgentInfo::Claude, "CLAUDECODE"),
            (AgentInfo::Claude, "CLAUDE_CODE"),
            (AgentInfo::Replit, "REPL_ID"),
            (AgentInfo::Gemini, "GEMINI_CLI"),
            (AgentInfo::Codex, "CODEX_SANDBOX"),
            (AgentInfo::Codex, "CODEX_THREAD_ID"),
            (AgentInfo::Copilot, "COPILOT_CLI"),
            (AgentInfo::OpenCode, "OPENCODE"),
            (AgentInfo::Cursor, "CURSOR_AGENT"),
        ];

        for (agent, env_key) in cases {
            assert_eq!(detect(&[(env_key, "1")]), Some(agent));
        }
    }

    #[test]
    fn detects_agent_env_matchers() {
        assert_eq!(detect(&[("PATH", "/usr/bin:/home/me/.pi/agent/bin")]), Some(AgentInfo::Pi));
        assert_eq!(detect(&[("PATH", r"C:\Users\me\.pi\agent\bin")]), Some(AgentInfo::Pi));
        assert_eq!(detect(&[("EDITOR", "devin")]), Some(AgentInfo::Devin));
        assert_eq!(detect(&[("TERM_PROGRAM", "kiro")]), Some(AgentInfo::Kiro));
    }

    #[test]
    fn returns_none_without_agent_env() {
        assert_eq!(detect(&[]), None);
    }
}
