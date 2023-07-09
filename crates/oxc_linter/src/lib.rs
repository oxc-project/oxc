#![allow(clippy::self_named_module_files)] // for rules.rs
#![feature(let_chains, const_trait_impl, const_slice_index)]

#[cfg(test)]
mod tester;

mod ast_util;
mod context;
mod disable_directives;
mod fixer;
mod globals;
mod jest_ast_util;
pub mod rule;
mod rules;

use std::{fs, io::Write, rc::Rc};

pub use fixer::{FixResult, Fixer, Message};
pub(crate) use oxc_semantic::AstNode;
use rustc_hash::FxHashMap;

pub use crate::{
    context::LintContext,
    rule::RuleCategory,
    rules::{RuleEnum, RULES},
};

#[derive(Debug)]
pub struct Linter {
    rules: Vec<RuleEnum>,
    fix: bool,
}

impl Linter {
    pub fn new() -> Self {
        let rules = RULES
            .iter()
            .cloned()
            .filter(|rule| rule.category() == RuleCategory::Correctness)
            .collect::<Vec<_>>();
        Self::from_rules(rules)
    }

    pub fn from_rules(rules: Vec<RuleEnum>) -> Self {
        Self { rules, fix: false }
    }

    pub fn has_fix(&self) -> bool {
        self.fix
    }

    pub fn number_of_rules(&self) -> usize {
        self.rules.len()
    }

    #[must_use]
    pub fn with_fix(mut self, yes: bool) -> Self {
        self.fix = yes;
        self
    }

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

    pub fn run<'a>(&self, ctx: LintContext<'a>) -> Vec<Message<'a>> {
        let semantic = Rc::clone(ctx.semantic());
        let mut ctx = ctx.with_fix(self.fix);
        for node in semantic.nodes().iter() {
            for rule in &self.rules {
                ctx.with_rule_name(rule.name());
                rule.run(node, &ctx);
            }
        }

        for symbol in semantic.symbols().iter() {
            for rule in &self.rules {
                ctx.with_rule_name(rule.name());
                rule.run_on_symbol(symbol, &ctx);
            }
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

    pub fn print_rules<W: Write>(writer: &mut W) {
        let rules_by_category = RULES.iter().fold(FxHashMap::default(), |mut map, rule| {
            map.entry(rule.category()).or_insert_with(Vec::new).push(rule);
            map
        });

        for (category, rules) in rules_by_category {
            writeln!(writer, "{} ({}):", category, rules.len()).unwrap();
            for rule in rules {
                writeln!(writer, "• {}/{}", rule.plugin_name(), rule.name()).unwrap();
            }
        }
        writeln!(writer, "Total: {}", RULES.len()).unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::Linter;

    #[test]
    fn print_rules() {
        let mut writer = Vec::new();
        Linter::print_rules(&mut writer);
        assert!(!writer.is_empty());
    }
}
