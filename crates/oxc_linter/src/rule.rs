use std::fmt::Debug;

use crate::{context::LintContext, AstNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    Correctness,
    Nursery,
}

pub trait Rule: Sized + Default + Debug {
    /// Initialize from eslint json configuration
    #[must_use]
    fn from_configuration(_value: serde_json::Value) -> Self {
        Self::default()
    }

    fn run_once(&self, _ctx: &LintContext<'_>) {}

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
