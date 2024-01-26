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

use rustc_hash::FxHashMap;
use std::{io::Write, rc::Rc, sync::Arc};

use oxc_diagnostics::Report;

use crate::{
    config::{ESLintEnv, ESLintSettings, JsxA11y},
    fixer::Fix,
    fixer::{Fixer, Message},
    rule::RuleCategory,
    rules::{RuleEnum, RULES},
};
pub use crate::{
    context::LintContext,
    options::{AllowWarnDeny, LintOptions},
    service::LintService,
};
use oxc_semantic::AstNode;

#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    use oxc_index::assert_eq_size;

    // `RuleEnum` runs in a really tight loop, make sure it is small for CPU cache.
    // A reduction from 168 bytes to 16 results 15% performance improvement.
    // See codspeed in https://github.com/oxc-project/oxc/pull/1783
    assert_eq_size!(RuleEnum, [u8; 16]);
}

#[derive(Debug)]
pub struct Linter {
    rules: Vec<(/* rule name */ &'static str, RuleEnum)>,
    options: LintOptions,
    settings: Arc<ESLintSettings>,
    env: Arc<ESLintEnv>,
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
        let (rules, settings, env) = options.derive_rules_and_settings_and_env()?;
        let rules = rules.into_iter().map(|rule| (rule.name(), rule)).collect();
        Ok(Self { rules, options, settings: Arc::new(settings), env: Arc::new(env) })
    }

    #[must_use]
    pub fn with_rules(mut self, rules: Vec<RuleEnum>) -> Self {
        self.rules = rules.into_iter().map(|rule| (rule.name(), rule)).collect();
        self
    }

    #[must_use]
    pub fn with_settings(mut self, settings: ESLintSettings) -> Self {
        self.settings = Arc::new(settings);
        self
    }

    #[must_use]
    pub fn with_envs(mut self, env: ESLintEnv) -> Self {
        self.env = Arc::new(env);
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
        let semantic = Rc::clone(ctx.semantic());
        let mut ctx =
            ctx.with_fix(self.options.fix).with_settings(&self.settings).with_env(&self.env);

        for (rule_name, rule) in &self.rules {
            ctx.with_rule_name(rule_name);
            rule.run_once(&ctx);
        }

        for symbol in semantic.symbols().iter() {
            for (rule_name, rule) in &self.rules {
                ctx.with_rule_name(rule_name);
                rule.run_on_symbol(symbol, &ctx);
            }
        }

        for node in semantic.nodes().iter() {
            for (rule_name, rule) in &self.rules {
                ctx.with_rule_name(rule_name);
                rule.run(node, &ctx);
            }
        }

        ctx.into_message()
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
