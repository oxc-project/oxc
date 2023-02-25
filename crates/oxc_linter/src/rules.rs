mod no_debugger;

pub use no_debugger::NoDebugger;
use oxc_ast::AstKind;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug)]
pub enum RuleEnum {
    NoDebugger(NoDebugger),
}

impl RuleEnum {
    pub fn run<'a>(&self, kind: AstKind<'a>, ctx: &LintContext<'a>) {
        match self {
            Self::NoDebugger(rule) => rule.run(kind, ctx),
        }
    }
}
