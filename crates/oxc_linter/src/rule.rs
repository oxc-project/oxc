use std::fmt::Debug;

use crate::{context::LintContext, AstNode};

pub trait Rule: Sized + Default + Debug {
    const NAME: &'static str;

    /// Initialize from eslint json configuration
    fn from_configuration(_value: serde_json::Value) -> Self {
        Self::default()
    }

    fn run<'a>(&self, node: &AstNode<'a>, _ctx: &LintContext<'a>);
}
