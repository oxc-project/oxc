#![feature(let_chains, is_some_and)]

#[cfg(test)]
mod tester;

mod context;
mod rule;
mod rules;

use std::rc::Rc;

use oxc_diagnostics::Error;
pub(crate) use oxc_semantic::AstNode;
use oxc_semantic::Semantic;

use crate::{
    context::LintContext,
    rules::{RuleEnum, RULES},
};

#[derive(Debug)]
pub struct Linter {
    rules: Vec<RuleEnum>,
}

impl Linter {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { rules: RULES.to_vec() }
    }

    #[must_use]
    pub fn from_rules(rules: Vec<RuleEnum>) -> Self {
        Self { rules }
    }

    #[must_use]
    pub fn run(&self, semantic: &Rc<Semantic>) -> Vec<Error> {
        let ctx = LintContext::new(semantic.clone());

        for node in semantic.nodes().iter() {
            for rule in &self.rules {
                rule.run(node, &ctx);
            }
        }

        ctx.into_diagnostics()
    }
}
