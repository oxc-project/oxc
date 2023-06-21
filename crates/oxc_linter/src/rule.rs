use std::fmt;

use oxc_semantic::SymbolId;

use crate::{context::LintContext, AstNode};

pub trait Rule: Sized + Default + fmt::Debug {
    /// Initialize from eslint json configuration
    fn from_configuration(_value: serde_json::Value) -> Self {
        Self::default()
    }

    fn run_on_symbol(&self, _symbol_id: SymbolId, _ctx: &LintContext<'_>) {}

    fn run<'a>(&self, _node: &AstNode<'a>, _ctx: &LintContext<'a>) {}
}

pub trait RuleMeta {
    const NAME: &'static str;

    const CATEGORY: RuleCategory;

    fn documentation() -> Option<&'static str> {
        None
    }
}

/// Rule categories defined by rust-clippy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    /// Code that is outright wrong or useless
    Correctness,
    ///
    Suspicious,
    /// Pedantic
    Pedantic,
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
            "restriction" => Some(Self::Restriction),
            "nursery" => Some(Self::Nursery),
            _ => None,
        }
    }
}

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Correctness => write!(f, "Correctness"),
            Self::Suspicious => write!(f, "Suspicious"),
            Self::Pedantic => write!(f, "Pedantic"),
            Self::Restriction => write!(f, "Restriction"),
            Self::Nursery => write!(f, "Nursery"),
        }
    }
}
