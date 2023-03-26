use std::fmt::Debug;

use oxc_semantic::Symbol;

use crate::{context::LintContext, AstNode};

pub trait Rule: Sized + Default + Debug {
    /// Initialize from eslint json configuration
    #[must_use]
    fn from_configuration(_value: serde_json::Value) -> Self {
        Self::default()
    }

    fn run_on_symbol(&self, _symbol: &Symbol, _ctx: &LintContext<'_>) {}

    fn run<'a>(&self, _node: &AstNode<'a>, _ctx: &LintContext<'a>) {}
}

pub trait RuleMeta {
    const NAME: &'static str;

    const CATEGORY: RuleCategory;

    #[must_use]
    fn documentation() -> Option<&'static str> {
        None
    }
}

/// Rule categories defined by rust-clippy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    /// Code that is outright wrong or useless
    Correctness,
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
    #[must_use]
    pub fn from(input: &str) -> Option<Self> {
        match input {
            "correctness" => Some(Self::Correctness),
            "restriction" => Some(Self::Restriction),
            "nursery" => Some(Self::Nursery),
            _ => None,
        }
    }
}
