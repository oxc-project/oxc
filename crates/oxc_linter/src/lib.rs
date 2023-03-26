#![allow(clippy::self_named_module_files)] // for rules.rs
#![feature(let_chains, is_some_and, const_trait_impl, const_slice_index)]

#[cfg(test)]
mod tester;

mod ast_util;
mod context;
mod disable_directives;
mod fixer;
mod globals;
pub mod rule;
mod rules;

use std::{fs, rc::Rc};

pub use fixer::{Fixer, Message};
pub(crate) use oxc_semantic::AstNode;
use oxc_semantic::Semantic;

use crate::{
    context::LintContext, rule::Rule, rules::early_error::javascript::EarlyErrorJavaScript,
};
pub use crate::{
    rule::RuleCategory,
    rules::{RuleEnum, RULES},
};

#[derive(Debug)]
pub struct Linter {
    rules: Vec<RuleEnum>,

    early_error_javascript: EarlyErrorJavaScript,

    fix: bool,
}

impl Linter {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let rules = RULES
            .iter()
            .cloned()
            .filter(|rule| rule.category() == RuleCategory::Correctness)
            .collect::<Vec<_>>();
        Self::from_rules(rules)
    }

    #[must_use]
    pub fn from_rules(rules: Vec<RuleEnum>) -> Self {
        Self { rules, early_error_javascript: EarlyErrorJavaScript, fix: false }
    }

    #[must_use]
    pub fn has_fix(&self) -> bool {
        self.fix
    }

    #[must_use]
    pub fn with_fix(mut self, yes: bool) -> Self {
        self.fix = yes;
        self
    }

    #[must_use]
    pub fn from_json_str(s: &str) -> Self {
        let rules = serde_json::from_str(s)
            .ok()
            .and_then(|v: serde_json::Value| v.get("rules").cloned())
            .and_then(|v| v.as_object().cloned())
            .map_or_else(
                || RULES.to_vec(),
                |rules_config| {
                    RULES
                        .iter()
                        .map(|rule| {
                            let value = rules_config.get(rule.name());
                            rule.read_json(value.cloned())
                        })
                        .collect()
                },
            );

        Self::from_rules(rules)
    }

    #[must_use]
    pub fn run<'a>(&self, semantic: &Rc<Semantic<'a>>) -> Vec<Message<'a>> {
        let mut ctx = LintContext::new(semantic, self.fix);

        for node in semantic.nodes().iter() {
            self.early_error_javascript.run(node, &ctx);
            for rule in &self.rules {
                ctx.with_rule_name(rule.name());
                rule.run(node, &ctx);
            }
        }

        for symbol in semantic.symbols().iter() {
            for rule in &self.rules {
                rule.run_on_symbol(symbol, &ctx);
            }
        }

        ctx.into_message()
    }

    #[must_use]
    pub fn run_early_error<'a>(&self, semantic: &Rc<Semantic<'a>>, fix: bool) -> Vec<Message<'a>> {
        let ctx = LintContext::new(semantic, fix);
        for node in semantic.nodes().iter() {
            self.early_error_javascript.run(node, &ctx);
        }
        ctx.into_message()
    }

    #[allow(unused)]
    fn read_rules_configuration() -> Option<serde_json::Map<String, serde_json::Value>> {
        fs::read_to_string(".eslintrc.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .and_then(|v: serde_json::Value| v.get("rules").cloned())
            .and_then(|v| v.as_object().cloned())
    }
}
