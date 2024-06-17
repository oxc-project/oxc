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
mod rule;
mod rules;
mod service;
mod utils;

pub mod partial_loader;
pub mod table;

use std::{io::Write, rc::Rc, sync::Arc};

use oxc_diagnostics::Error;
use oxc_semantic::AstNode;

pub use crate::{
    config::OxlintConfig,
    context::LintContext,
    options::{AllowWarnDeny, LintOptions},
    rule::{RuleCategory, RuleMeta, RuleWithSeverity},
    service::{LintService, LintServiceOptions},
};
use crate::{
    config::{OxlintEnv, OxlintGlobals, OxlintSettings},
    fixer::Fix,
    fixer::{Fixer, Message},
    rules::RuleEnum,
    table::RuleTable,
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
    eslint_config: Arc<OxlintConfig>,
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
    pub fn with_eslint_config(mut self, eslint_config: OxlintConfig) -> Self {
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
        let table = RuleTable::new();
        for section in table.sections {
            writeln!(writer, "{}", section.render_markdown_table()).unwrap();
        }
        writeln!(writer, "Default: {}", table.turned_on_by_default_count).unwrap();
        writeln!(writer, "Total: {}", table.total).unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::{Linter, OxlintConfig};

    #[test]
    fn print_rules() {
        let mut writer = Vec::new();
        Linter::print_rules(&mut writer);
        assert!(!writer.is_empty());
    }

    #[test]
    fn test_schema_json() {
        use project_root::get_project_root;
        use std::fs;
        let path = get_project_root().unwrap().join("npm/oxlint/configuration_schema.json");
        let schema = schemars::schema_for!(OxlintConfig);
        let json = serde_json::to_string_pretty(&schema).unwrap();
        let existing_json = fs::read_to_string(&path).unwrap_or_default();
        if existing_json != json {
            std::fs::write(&path, &json).unwrap();
        }
        insta::with_settings!({ prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!(json);
        });
    }
}
