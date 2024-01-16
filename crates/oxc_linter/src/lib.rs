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
mod rules;
mod service;
mod settings;
mod utils;

use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::{io::Write, rc::Rc, sync::Arc};

use oxc_diagnostics::Report;

pub use crate::{
    context::LintContext,
    disable_directives::DisableDirectivesBuilder,
    fixer::Fix,
    fixer::{FixResult, Fixer, Message},
    options::{AllowWarnDeny, LintOptions},
    rule::RuleCategory,
    service::LintService,
    settings::LintSettings,
};
pub(crate) use crate::{
    rules::{RuleEnum, RULES},
    settings::JsxA11y,
};
pub(crate) use oxc_semantic::AstNode;

#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    use oxc_index::assert_eq_size;

    // `RuleEnum` runs in a really tight loop, make sure it is small for CPU cache.
    // A reduction from 168 bytes to 16 results 15% performance improvement.
    // See codspeed in https://github.com/oxc-project/oxc/pull/1783
    assert_eq_size!(RuleEnum, [u8; 16]);
}

pub struct Linter {
    rules: Vec<RuleEnum>,
    options: LintOptions,
    settings: Arc<LintSettings>,
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
    pub fn from_options(options: LintOptions) -> Result<Self, Report> {
        let (rules, settings) = options.derive_rules_and_settings()?;
        Ok(Self { rules, options, settings: Arc::new(settings) })
    }

    #[must_use]
    pub fn with_rules(mut self, rules: Vec<RuleEnum>) -> Self {
        self.rules = rules;
        self
    }

    #[must_use]
    pub fn with_settings(mut self, settings: LintSettings) -> Self {
        self.settings = Arc::new(settings);
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

    pub fn run<'a>(&self, ctx: LintContext<'a>) -> Vec<Message<'a>> {
        let disable_directives =
            Rc::new(DisableDirectivesBuilder::from_semantic(ctx.semantic()).build());

        // Initialize context for each rule to avoid mutations inside all the hot loops below
        let ctx = ctx.with_fix(self.options.fix).with_settings(&self.settings);
        let rules = self
            .rules
            .iter()
            .map(|rule| {
                let ctx = ctx
                    .clone_without_diagnostics()
                    .with_disable_directives(&disable_directives)
                    .with_rule_name(rule.name());
                (ctx, rule)
            })
            .collect::<Vec<_>>();

        for (ctx, rule) in &rules {
            rule.run_once(ctx);
        }

        for symbol in ctx.semantic().symbols().iter() {
            for (ctx, rule) in &rules {
                rule.run_on_symbol(symbol, ctx);
            }
        }

        for node in ctx.semantic().nodes().iter() {
            for (ctx, rule) in &rules {
                rule.run(node, ctx);
            }
        }

        rules.into_iter().map(|(ctx, _)| ctx.into_message()).concat()
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
