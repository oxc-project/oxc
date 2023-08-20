#![allow(clippy::self_named_module_files)] // for rules.rs

#[cfg(test)]
mod tester;

mod ast_util;
mod context;
mod disable_directives;
mod fixer;
mod globals;
mod jest_ast_util;
mod options;
pub mod rule;
mod rule_timer;
mod rules;
mod service;

use std::{self, fs, io::Write, rc::Rc, time::Duration};

pub use fixer::{FixResult, Fixer, Message};
pub(crate) use oxc_semantic::AstNode;
use rustc_hash::FxHashMap;

pub use crate::{
    context::LintContext,
    options::{AllowWarnDeny, LintOptions},
    rule::RuleCategory,
    service::LintService,
};
pub(crate) use rules::{RuleEnum, RULES};

#[derive(Debug)]
pub struct Linter {
    rules: Vec<RuleEnum>,
    options: LintOptions,
}

impl Linter {
    pub fn new() -> Self {
        let rules = RULES
            .iter()
            .cloned()
            .filter(|rule| rule.category() == RuleCategory::Correctness)
            .collect::<Vec<_>>();
        Self { rules, options: LintOptions::default() }
    }

    pub fn from_options(options: LintOptions) -> Self {
        let rules = options.derive_rules();
        Self { rules, options }
    }

    #[must_use]
    pub fn with_rules(mut self, rules: Vec<RuleEnum>) -> Self {
        self.rules = rules;
        self
    }

    pub fn rules(&self) -> &Vec<RuleEnum> {
        &self.rules
    }

    pub fn options(&self) -> &LintOptions {
        &self.options
    }

    pub fn number_of_rules(&self) -> usize {
        self.rules.len()
    }

    #[must_use]
    pub fn with_fix(mut self, yes: bool) -> Self {
        self.options.fix = yes;
        self
    }

    #[must_use]
    pub fn with_print_execution_times(mut self, yes: bool) -> Self {
        self.options.timing = yes;
        self
    }

    pub fn run<'a>(&self, ctx: LintContext<'a>) -> Vec<Message<'a>> {
        let timing = self.options.timing;
        let semantic = Rc::clone(ctx.semantic());
        let mut ctx = ctx.with_fix(self.options.fix);

        for rule in &self.rules {
            ctx.with_rule_name(rule.name());
            rule.run_once(&ctx, timing);
        }

        for node in semantic.nodes().iter() {
            for rule in &self.rules {
                ctx.with_rule_name(rule.name());
                rule.run(node, &ctx, timing);
            }
        }

        for symbol in semantic.symbols().iter() {
            for rule in &self.rules {
                ctx.with_rule_name(rule.name());
                rule.run_on_symbol(symbol, &ctx, timing);
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

    pub fn print_execution_times_if_enable(&self) {
        if !self.options.timing {
            return;
        }
        let mut timings =
            self.rules().iter().map(|rule| (rule.name(), rule.execute_time())).collect::<Vec<_>>();

        timings.sort_by_key(|x| x.1);
        let total = timings.iter().map(|x| x.1).sum::<Duration>().as_secs_f64();

        println!("Rule timings in milliseconds:");
        println!("Total: {:.2}ms", total * 1000.0);
        println!("{:>7} | {:>5} | Rule", "Time", "%");
        for (name, duration) in timings.iter().rev() {
            let millis = duration.as_secs_f64() * 1000.0;
            let relative = duration.as_secs_f64() / total * 100.0;
            println!("{millis:>7.2} | {relative:>4.1}% | {name}");
        }
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
