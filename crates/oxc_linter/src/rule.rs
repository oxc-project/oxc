use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};

use oxc_semantic::SymbolId;

use crate::{context::LintContext, AllowWarnDeny, AstNode, RuleEnum};

pub trait Rule: Sized + Default + fmt::Debug {
    /// Initialize from eslint json configuration
    fn from_configuration(_value: serde_json::Value) -> Self {
        Self::default()
    }

    /// Visit each AST Node
    fn run<'a>(&self, _node: &AstNode<'a>, _ctx: &LintContext<'a>) {}

    /// Visit each symbol
    fn run_on_symbol(&self, _symbol_id: SymbolId, _ctx: &LintContext<'_>) {}

    /// Run only once. Useful for inspecting scopes and trivias etc.
    fn run_once(&self, _ctx: &LintContext) {}
}

pub trait RuleMeta {
    const NAME: &'static str;

    const CATEGORY: RuleCategory;

    const USE_CFG: bool;

    fn documentation() -> Option<&'static str> {
        None
    }
}

/// Rule categories defined by rust-clippy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    /// Code that is outright wrong or useless
    Correctness,
    /// Code that is most likely wrong or useless
    Suspicious,
    /// Lints which are rather strict or have occasional false positives
    Pedantic,
    /// Code that can be written to run faster
    Perf,
    /// Code that should be written in a more idiomatic way
    Style,
    /// Lints which prevent the use of language and library features
    /// The restriction category should, emphatically, not be enabled as a whole.
    /// The contained lints may lint against perfectly reasonable code, may not have an alternative suggestion,
    /// and may contradict any other lints (including other categories).
    /// Lints should be considered on a case-by-case basis before enabling.
    Restriction,
    /// New lints that are still under development
    Nursery,
}

impl RuleCategory {
    pub fn from(input: &str) -> Option<Self> {
        match input {
            "correctness" => Some(Self::Correctness),
            "suspicious" => Some(Self::Suspicious),
            "pedantic" => Some(Self::Pedantic),
            "perf" => Some(Self::Perf),
            "style" => Some(Self::Style),
            "restriction" => Some(Self::Restriction),
            "nursery" => Some(Self::Nursery),
            _ => None,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Correctness => "Code that is outright wrong or useless.",
            Self::Suspicious => "code that is most likely wrong or useless.",
            Self::Pedantic => "Lints which are rather strict or have occasional false positives.",
            Self::Perf => "Code that can be written to run faster.",
            Self::Style => "Code that should be written in a more idiomatic way.",
            Self::Restriction => {
                "Lints which prevent the use of language and library features. Must not be enabled as a whole, should be considered on a case-by-case basis before enabling."
            }
            Self::Nursery => "New lints that are still under development.",
        }
    }
}

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Correctness => write!(f, "Correctness"),
            Self::Suspicious => write!(f, "Suspicious"),
            Self::Pedantic => write!(f, "Pedantic"),
            Self::Perf => write!(f, "Perf"),
            Self::Style => write!(f, "Style"),
            Self::Restriction => write!(f, "Restriction"),
            Self::Nursery => write!(f, "Nursery"),
        }
    }
}

#[derive(Clone)]
pub struct RuleWithSeverity {
    pub rule: RuleEnum,
    pub severity: AllowWarnDeny,
}

impl Hash for RuleWithSeverity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
    }
}

impl PartialEq for RuleWithSeverity {
    fn eq(&self, other: &Self) -> bool {
        self.rule == other.rule
    }
}

impl Eq for RuleWithSeverity {}

impl Deref for RuleWithSeverity {
    type Target = RuleEnum;

    fn deref(&self) -> &Self::Target {
        &self.rule
    }
}

impl RuleWithSeverity {
    pub fn new(rule: RuleEnum, severity: AllowWarnDeny) -> Self {
        Self { rule, severity }
    }
}

#[cfg(test)]
mod test {
    use crate::rules::RULES;

    #[test]
    fn ensure_documentation() {
        assert!(!RULES.is_empty());
        for rule in RULES.iter() {
            assert!(rule.documentation().is_some_and(|s| !s.is_empty()), "{}", rule.name());
        }
    }
}
