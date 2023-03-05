use std::fmt::Debug;

use crate::{context::LintContext, AstNode};

pub trait Rule: Sized + Default + Debug {
    /// Initialize from eslint json configuration
    #[must_use]
    fn from_configuration(_value: serde_json::Value) -> Self {
        Self::default()
    }

    fn run<'a>(&self, node: &AstNode<'a>, _ctx: &LintContext<'a>);
}

pub trait RuleMeta {
    const NAME: &'static str;

    const CATEGORY: &'static str;

    #[must_use]
    fn documentation() -> Option<&'static str> {
        None
    }
}
