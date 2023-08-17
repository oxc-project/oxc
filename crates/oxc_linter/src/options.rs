use std::path::PathBuf;

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct LintOptions {
    pub paths: Vec<PathBuf>,
    /// Allow / Deny rules in order. [("allow" / "deny", rule name)]
    /// Defaults to [("deny", "correctness")]
    pub rules: Vec<(AllowWarnDeny, String)>,
    pub list_rules: bool,
    pub fix: bool,
    pub quiet: bool,
    pub ignore_path: PathBuf,
    pub no_ignore: bool,
    pub ignore_pattern: Vec<String>,
    pub max_warnings: Option<usize>,
    pub print_execution_times: bool,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            paths: vec![],
            rules: vec![(AllowWarnDeny::Deny, String::from("correctness"))],
            list_rules: false,
            fix: false,
            quiet: false,
            ignore_path: PathBuf::default(),
            no_ignore: false,
            ignore_pattern: vec![],
            max_warnings: None,
            print_execution_times: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AllowWarnDeny {
    Allow,
    // Warn,
    Deny,
}

impl From<&'static str> for AllowWarnDeny {
    fn from(s: &'static str) -> Self {
        match s {
            "allow" => Self::Allow,
            "deny" => Self::Deny,
            _ => unreachable!(),
        }
    }
}
