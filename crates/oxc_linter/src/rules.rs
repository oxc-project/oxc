mod no_debugger;
mod no_empty;

pub use no_debugger::NoDebugger;
pub use no_empty::NoEmpty;
use oxc_ast::AstKind;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Clone)]
pub enum RuleEnum {
    NoDebugger(NoDebugger),
    NoEmpty(NoEmpty),
}

impl RuleEnum {
    pub fn run<'a>(&self, kind: AstKind<'a>, ctx: &LintContext<'a>) {
        match self {
            Self::NoDebugger(rule) => rule.run(kind, ctx),
            Self::NoEmpty(rule) => rule.run(kind, ctx),
        }
    }
}
