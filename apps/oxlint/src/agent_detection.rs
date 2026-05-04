use std::ffi::OsString;

use cow_utils::CowUtils;

/// Information about the AI coding agent detected from the environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentInfo {
    /// The detected AI coding agent name.
    pub name: Option<String>,
}

impl AgentInfo {
    fn none() -> Self {
        Self { name: None }
    }

    fn new(name: impl Into<String>) -> Self {
        Self { name: Some(name.into()) }
    }

    /// Return `true` when an AI coding agent was detected.
    pub fn is_agent(&self) -> bool {
        self.name.is_some()
    }
}

/// Detect the current AI coding agent from environment variables.
///
/// This mirrors the environment-based detection used by `std-env` while keeping
/// the logic reusable for Rust CLI entry points.
pub fn detect_agent() -> AgentInfo {
    detect_agent_from_env(|key| std::env::var_os(key))
}

/// Return `true` when the current process appears to be running inside an AI
/// coding agent.
pub fn is_agent() -> bool {
    detect_agent().is_agent()
}

fn detect_agent_from_env(mut env: impl FnMut(&str) -> Option<OsString>) -> AgentInfo {
    if let Some(ai_agent) = env_string(&mut env, "AI_AGENT")
        && !ai_agent.is_empty()
    {
        return AgentInfo::new(ai_agent.cow_to_ascii_lowercase().into_owned());
    }

    if has_any_env(&mut env, &["CLAUDECODE", "CLAUDE_CODE"]) {
        AgentInfo::new("claude")
    } else if has_any_env(&mut env, &["REPL_ID"]) {
        AgentInfo::new("replit")
    } else if has_any_env(&mut env, &["GEMINI_CLI"]) {
        AgentInfo::new("gemini")
    } else if has_any_env(&mut env, &["CODEX_SANDBOX", "CODEX_THREAD_ID"]) {
        AgentInfo::new("codex")
    } else if has_any_env(&mut env, &["OPENCODE"]) {
        AgentInfo::new("opencode")
    } else if env_contains_path_segment(&mut env, "PATH", ".pi/agent") {
        AgentInfo::new("pi")
    } else if has_any_env(&mut env, &["AUGMENT_AGENT"]) {
        AgentInfo::new("auggie")
    } else if has_any_env(&mut env, &["GOOSE_PROVIDER"]) {
        AgentInfo::new("goose")
    } else if env_contains(&mut env, "EDITOR", "devin") {
        AgentInfo::new("devin")
    } else if has_any_env(&mut env, &["CURSOR_AGENT"]) {
        AgentInfo::new("cursor")
    } else if env_contains(&mut env, "TERM_PROGRAM", "kiro") {
        AgentInfo::new("kiro")
    } else {
        AgentInfo::none()
    }
}

fn has_any_env(env: &mut impl FnMut(&str) -> Option<OsString>, keys: &[&str]) -> bool {
    keys.iter().any(|key| env(key).is_some_and(|value| !value.is_empty()))
}

fn env_string(env: &mut impl FnMut(&str) -> Option<OsString>, key: &str) -> Option<String> {
    env(key).filter(|value| !value.is_empty()).map(|value| value.to_string_lossy().into_owned())
}

fn env_contains(env: &mut impl FnMut(&str) -> Option<OsString>, key: &str, needle: &str) -> bool {
    env_string(env, key).is_some_and(|value| value.contains(needle))
}

fn env_contains_path_segment(
    env: &mut impl FnMut(&str) -> Option<OsString>,
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

    fn detect(vars: &[(&str, &str)]) -> AgentInfo {
        let env = vars
            .iter()
            .map(|(key, value)| ((*key).to_string(), OsString::from(value)))
            .collect::<FxHashMap<_, _>>();

        detect_agent_from_env(|key| env.get(key).cloned())
    }

    #[test]
    fn detects_explicit_ai_agent() {
        assert_eq!(detect(&[("AI_AGENT", "CustomAgent")]).name.as_deref(), Some("customagent"));
    }

    #[test]
    fn explicit_ai_agent_takes_precedence() {
        assert_eq!(
            detect(&[("AI_AGENT", "custom"), ("CLAUDECODE", "1")]).name.as_deref(),
            Some("custom")
        );
    }

    #[test]
    fn ignores_empty_env_values() {
        assert_eq!(detect(&[("AI_AGENT", ""), ("CLAUDECODE", "")]).name, None);
    }

    #[test]
    fn detects_agent_env_vars() {
        let cases = [
            ("claude", "CLAUDECODE"),
            ("claude", "CLAUDE_CODE"),
            ("replit", "REPL_ID"),
            ("gemini", "GEMINI_CLI"),
            ("codex", "CODEX_SANDBOX"),
            ("codex", "CODEX_THREAD_ID"),
            ("opencode", "OPENCODE"),
            ("auggie", "AUGMENT_AGENT"),
            ("goose", "GOOSE_PROVIDER"),
            ("cursor", "CURSOR_AGENT"),
        ];

        for (agent, env_key) in cases {
            assert_eq!(detect(&[(env_key, "1")]).name.as_deref(), Some(agent));
        }
    }

    #[test]
    fn detects_agent_env_matchers() {
        assert_eq!(
            detect(&[("PATH", "/usr/bin:/home/me/.pi/agent/bin")]).name.as_deref(),
            Some("pi")
        );
        assert_eq!(detect(&[("PATH", r"C:\Users\me\.pi\agent\bin")]).name.as_deref(), Some("pi"));
        assert_eq!(detect(&[("EDITOR", "devin")]).name.as_deref(), Some("devin"));
        assert_eq!(detect(&[("TERM_PROGRAM", "kiro")]).name.as_deref(), Some("kiro"));
    }

    #[test]
    fn returns_none_without_agent_env() {
        assert_eq!(detect(&[]).name, None);
    }
}
