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
mod javascript_globals;
mod options;
pub mod partial_loader;
pub mod rule;
mod rules;
mod service;
mod utils;

use std::{io::Write, rc::Rc, sync::Arc};

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_diagnostics::Error;
use oxc_semantic::AstNode;

pub use crate::{
    config::ESLintConfig,
    context::LintContext,
    options::{AllowWarnDeny, LintOptions},
    rule::RuleWithSeverity,
    service::{LintService, LintServiceOptions},
};
use crate::{
    config::{ESLintEnv, ESLintGlobals, ESLintSettings},
    fixer::Fix,
    fixer::{Fixer, Message},
    rule::RuleCategory,
    rules::{RuleEnum, RULES},
};

#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    use static_assertions::assert_eq_size;

    // `RuleEnum` runs in a really tight loop, make sure it is small for CPU cache.
    // A reduction from 168 bytes to 16 results 15% performance improvement.
    // See codspeed in https://github.com/oxc-project/oxc/pull/1783
    assert_eq_size!(RuleEnum, [u8; 16]);
}

pub struct Linter {
    rules: Vec<RuleWithSeverity>,
    options: LintOptions,
    eslint_config: Arc<ESLintConfig>,
}

impl Default for Linter {
    fn default() -> Self {
        Self::from_options(LintOptions::default()).unwrap()
    }
}

impl Linter {
    /// # Errors
    ///
    /// Returns `Err` if there are any errors parsing the configuration file.
    pub fn from_options(options: LintOptions) -> Result<Self, Error> {
        let (rules, eslint_config) = options.derive_rules_and_config()?;
        Ok(Self { rules, options, eslint_config: Arc::new(eslint_config) })
    }

    #[cfg(test)]
    #[must_use]
    pub fn with_rules(mut self, rules: Vec<RuleWithSeverity>) -> Self {
        self.rules = rules;
        self
    }

    #[must_use]
    pub fn with_eslint_config(mut self, eslint_config: ESLintConfig) -> Self {
        self.eslint_config = Arc::new(eslint_config);
        self
    }

    #[must_use]
    pub fn with_fix(mut self, yes: bool) -> Self {
        self.options.fix = yes;
        self
    }

    pub fn options(&self) -> &LintOptions {
        &self.options
    }

    pub fn number_of_rules(&self) -> usize {
        self.rules.len()
    }

    pub fn run<'a>(&self, ctx: LintContext<'a>) -> Vec<Message<'a>> {
        let semantic = Rc::clone(ctx.semantic());

        let ctx = ctx.with_fix(self.options.fix).with_eslint_config(&self.eslint_config);
        let rules = self
            .rules
            .iter()
            .map(|rule| {
                (rule, ctx.clone().with_rule_name(rule.name()).with_severity(rule.severity))
            })
            .collect::<Vec<_>>();

        for (rule, ctx) in &rules {
            rule.run_once(ctx);
        }

        for symbol in semantic.symbols().iter() {
            for (rule, ctx) in &rules {
                rule.run_on_symbol(symbol, ctx);
            }
        }

        for node in semantic.nodes().iter() {
            for (rule, ctx) in &rules {
                rule.run(node, ctx);
            }
        }

        rules.into_iter().flat_map(|(_, ctx)| ctx.into_message()).collect::<Vec<_>>()
    }

    /// # Panics
    pub fn print_rules<W: Write>(writer: &mut W) {
        let default_rules = Linter::default()
            .rules
            .into_iter()
            .map(|rule| rule.name())
            .collect::<FxHashSet<&str>>();

        let rules_by_category = RULES.iter().fold(
            FxHashMap::default(),
            |mut map: FxHashMap<RuleCategory, Vec<&RuleEnum>>, rule| {
                map.entry(rule.category()).or_default().push(rule);
                map
            },
        );

        let mut default_count = 0;

        for (category, rules) in rules_by_category {
            writeln!(writer, "## {} ({}):", category, rules.len()).unwrap();

            let rule_width = rules.iter().map(|r| r.name().len()).max().unwrap();
            let plugin_width = rules.iter().map(|r| r.plugin_name().len()).max().unwrap();
            let x = "";
            writeln!(
                writer,
                "| {:<rule_width$} | {:<plugin_width$} | Default |",
                "Rule name", "Source"
            )
            .unwrap();
            writeln!(writer, "| {x:-<rule_width$} | {x:-<plugin_width$} | {x:-<7} |").unwrap();

            for rule in rules {
                let rule_name = rule.name();
                let plugin_name = rule.plugin_name();
                let (default, default_width) = if default_rules.contains(rule_name) {
                    default_count += 1;
                    ("✅", 6)
                } else {
                    ("", 7)
                };
                writeln!(writer, "| {rule_name:<rule_width$} | {plugin_name:<plugin_width$} | {default:<default_width$} |").unwrap();
            }
            writeln!(writer).unwrap();
        }
        writeln!(writer, "Default: {default_count}").unwrap();
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
