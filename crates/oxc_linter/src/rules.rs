mod no_debugger;
mod no_empty;

pub use no_debugger::NoDebugger;
pub use no_empty::NoEmpty;

use crate::{context::LintContext, rule::Rule, AstNode};

lazy_static::lazy_static! {
    pub static ref RULES: Vec<RuleEnum> = vec![
        RuleEnum::NoDebugger(NoDebugger::default()),
        RuleEnum::NoEmpty(NoEmpty::default())
    ];
}

#[derive(Debug, Clone)]
pub enum RuleEnum {
    NoDebugger(NoDebugger),
    NoEmpty(NoEmpty),
}

impl RuleEnum {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::NoDebugger(_) => NoDebugger::NAME,
            Self::NoEmpty(_) => NoEmpty::NAME,
        }
    }

    pub fn from_json(&self, maybe_value: Option<serde_json::Value>) -> Self {
        match self {
            Self::NoDebugger(_) => {
                RuleEnum::NoDebugger(maybe_value.map(NoDebugger::from_json).unwrap_or_default())
            }
            Self::NoEmpty(_) => {
                RuleEnum::NoEmpty(maybe_value.map(NoEmpty::from_json).unwrap_or_default())
            }
        }
    }

    pub fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match self {
            Self::NoDebugger(rule) => rule.run(node, ctx),
            Self::NoEmpty(rule) => rule.run(node, ctx),
        }
    }
}
