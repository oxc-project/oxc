#![warn(clippy::print_stdout)]
#![allow(clippy::self_named_module_files)] // for rules.rs

#[cfg(test)]
mod tester;

mod ast_util;
mod config;
mod context;
mod disable_directives;
mod fixer;
mod globals;
mod options;
pub mod partial_loader;
pub mod rule;
mod rule_timer;
mod rules;
mod service;
mod utils;

use std::{self, fs, io::Write, rc::Rc, time::Duration};

use oxc_diagnostics::Report;
pub(crate) use oxc_semantic::AstNode;
use rustc_hash::FxHashMap;

pub use crate::{
    context::LintContext,
    fixer::Fix,
    fixer::{FixResult, Fixer, Message},
    options::{AllowWarnDeny, LintOptions},
    rule::RuleCategory,
    service::LintService,
};
pub(crate) use rules::{RuleEnum, RULES};

#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    use oxc_index::assert_eq_size;

    // `RuleEnum` runs in a really tight loop, make sure it is small for CPU cache.
    // A reduction from 168 bytes to 16 results 15% performance improvement.
    // See codspeed in https://github.com/oxc-project/oxc/pull/1783
    assert_eq_size!(RuleEnum, [u8; 16]);
}

#[derive(Debug, Clone)]
pub struct LintSettings {
    jsx_a11y: JsxA11y,
}

impl Default for LintSettings {
    fn default() -> Self {
        Self { jsx_a11y: JsxA11y { polymorphic_prop_name: None, components: FxHashMap::default() } }
    }
}

#[derive(Debug, Clone)]
pub struct JsxA11y {
    polymorphic_prop_name: Option<String>,
    components: FxHashMap<String, String>,
}

impl JsxA11y {
    pub fn set_components(&mut self, components: FxHashMap<String, String>) {
        self.components = components;
    }

    pub fn set_polymorphic_prop_name(&mut self, name: Option<String>) {
        self.polymorphic_prop_name = name;
    }
}

#[derive(Debug)]
pub struct Linter {
    rules: Vec<(/* rule name */ &'static str, RuleEnum)>,
    options: LintOptions,
    settings: LintSettings,
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter {
    pub fn new() -> Self {
        let rules = RULES
            .iter()
            .filter(|&rule| rule.category() == RuleCategory::Correctness)
            .cloned()
            .map(|rule| (rule.name(), rule))
            .collect::<Vec<_>>();
        Self { rules, options: LintOptions::default(), settings: LintSettings::default() }
    }

    /// # Errors
    ///
    /// Returns `Err` if there are any errors parsing the configuration file.
    pub fn from_options(options: LintOptions) -> Result<Self, Report> {
        let (rules, settings) = options.derive_rules_and_settings()?;
        let rules = rules.into_iter().map(|rule| (rule.name(), rule)).collect();
        Ok(Self { rules, options, settings })
    }

    #[must_use]
    pub fn with_rules(mut self, rules: Vec<RuleEnum>) -> Self {
        self.rules = rules.into_iter().map(|rule| (rule.name(), rule)).collect();
        self
    }

    #[must_use]
    pub fn with_settings(mut self, settings: LintSettings) -> Self {
        self.settings = settings;
        self
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

        for (rule_name, rule) in &self.rules {
            ctx.with_rule_name(rule_name);
            rule.run_once(&ctx, timing);
        }

        for symbol in semantic.symbols().iter() {
            for (rule_name, rule) in &self.rules {
                ctx.with_rule_name(rule_name);
                rule.run_on_symbol(symbol, &ctx, timing);
            }
        }

        for node in semantic.nodes().iter() {
            for (rule_name, rule) in &self.rules {
                ctx.with_rule_name(rule_name);
                rule.run(node, &ctx, timing);
            }
        }

        ctx.into_message()
    }

    pub fn get_settings(&self) -> LintSettings {
        self.settings.clone()
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
        let rules_by_category = RULES.iter().fold(
            FxHashMap::default(),
            |mut map: FxHashMap<RuleCategory, Vec<&RuleEnum>>, rule| {
                map.entry(rule.category()).or_default().push(rule);
                map
            },
        );

        for (category, rules) in rules_by_category {
            writeln!(writer, "{} ({}):", category, rules.len()).unwrap();
            for rule in rules {
                // Separate the category and rule name so people don't copy the combination as a whole for `--allow` and `--deny`,
                // resulting invalid rule names.
                writeln!(writer, "• {}: {}", rule.plugin_name(), rule.name()).unwrap();
            }
        }
        writeln!(writer, "Total: {}", RULES.len()).unwrap();
    }

    #[allow(clippy::print_stdout)]
    pub fn print_execution_times_if_enable(&self) {
        if !self.options.timing {
            return;
        }
        let mut timings = self
            .rules
            .iter()
            .map(|(rule_name, rule)| (rule_name, rule.execute_time()))
            .collect::<Vec<_>>();

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
