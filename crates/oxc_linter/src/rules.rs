mod no_debugger;
mod no_empty;

pub use no_debugger::NoDebugger;
pub use no_empty::NoEmpty;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Clone)]
pub enum RuleEnum {
    NoDebugger(NoDebugger),
    NoEmpty(NoEmpty),
}

impl RuleEnum {
    pub fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match self {
            Self::NoDebugger(rule) => rule.run(node, ctx),
            Self::NoEmpty(rule) => rule.run(node, ctx),
        }
    }
}
