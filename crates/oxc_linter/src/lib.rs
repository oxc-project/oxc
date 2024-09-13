#![allow(clippy::self_named_module_files)] // for rules.rs

#[cfg(test)]
mod tester;

mod ast_util;
mod config;
mod context;
mod disable_directives;
mod fixer;
mod frameworks;
mod globals;
mod javascript_globals;
mod options;
mod rule;
mod rules;
mod service;
mod utils;

pub mod partial_loader;
pub mod table;

use std::{io::Write, path::Path, rc::Rc, sync::Arc};

use config::LintConfig;
use options::LintOptions;
use oxc_diagnostics::Error;
use oxc_semantic::{Node, Semantic};

pub use crate::{
    config::OxlintConfig,
    context::LintContext,
    fixer::FixKind,
    frameworks::FrameworkFlags,
    options::{AllowWarnDeny, InvalidFilterKind, LintFilter, OxlintOptions},
    rule::{RuleCategory, RuleFixMeta, RuleMeta, RuleWithSeverity},
    service::{LintService, LintServiceOptions},
};
use crate::{
    config::{OxlintEnv, OxlintGlobals, OxlintSettings},
    fixer::{Fixer, Message},
    rules::RuleEnum,
    table::RuleTable,
};

#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    // `RuleEnum` runs in a really tight loop, make sure it is small for CPU cache.
    // A reduction from 168 bytes to 16 results 15% performance improvement.
    // See codspeed in https://github.com/oxc-project/oxc/pull/1783
    assert!(std::mem::size_of::<RuleEnum>() == 16);
}

#[derive(Debug)]
pub struct Linter {
    rules: Vec<RuleWithSeverity>,
    options: LintOptions,
    config: Arc<LintConfig>,
}

impl Default for Linter {
    fn default() -> Self {
        Self::from_options(OxlintOptions::default()).unwrap()
    }
}

impl Linter {
    /// # Errors
    ///
    /// Returns `Err` if there are any errors parsing the configuration file.
    pub fn from_options(options: OxlintOptions) -> Result<Self, Error> {
        let (rules, config) = options.derive_rules_and_config()?;
        Ok(Self { rules, options: options.into(), config: Arc::new(config) })
    }

    #[cfg(test)]
    #[must_use]
    pub fn with_rules(mut self, rules: Vec<RuleWithSeverity>) -> Self {
        self.rules = rules;
        self
    }

    /// Used for testing
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_eslint_config(mut self, config: LintConfig) -> Self {
        self.config = Arc::new(config);
        self
    }

    /// Set the kind of auto fixes to apply.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_linter::{Linter, FixKind};
    ///
    /// // turn off all auto fixes. This is default behavior.
    /// Linter::default().with_fix(FixKind::None);
    /// ```
    #[must_use]
    pub fn with_fix(mut self, kind: FixKind) -> Self {
        self.options.fix = kind;
        self
    }

    pub(crate) fn options(&self) -> &LintOptions {
        &self.options
    }

    pub fn number_of_rules(&self) -> usize {
        self.rules.len()
    }

    // pub fn run<'a>(&self, ctx: LintContext<'a>) -> Vec<Message<'a>> {
    pub fn run<'a>(&self, path: &Path, semantic: Rc<Semantic<'a>>) -> Vec<Message<'a>> {
        let ctx = self.create_ctx(path, semantic);
        let semantic = Rc::clone(ctx.semantic());

        let rules = self
            .rules
            .iter()
            .filter(|rule| rule.should_run(&ctx))
            .map(|rule| (rule, self.ctx_for_rule(&ctx, rule)))
            .collect::<Vec<_>>();

        for (rule, ctx) in &rules {
            rule.run_once(ctx);
        }

        for symbol in semantic.symbols().symbol_ids() {
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
            writeln!(writer, "{}", section.render_markdown_table(None)).unwrap();
        }
        writeln!(writer, "Default: {}", table.turned_on_by_default_count).unwrap();
        writeln!(writer, "Total: {}", table.total).unwrap();
    }

    fn create_ctx<'a>(&self, path: &Path, semantic: Rc<Semantic<'a>>) -> LintContext<'a> {
        let mut ctx = LintContext::new(path.to_path_buf().into_boxed_path(), semantic)
            .with_fix(self.options.fix)
            .with_config(&self.config)
            .with_frameworks(self.options.framework_hints);

        // set file-specific jest/vitest flags
        if self.options.plugins.has_test() {
            let mut test_flags = FrameworkFlags::empty();

            if frameworks::has_vitest_imports(ctx.module_record()) {
                test_flags.set(FrameworkFlags::Vitest, true);
            } else if frameworks::is_jestlike_file(path)
                || frameworks::has_jest_imports(ctx.module_record())
            {
                test_flags.set(FrameworkFlags::Jest, true);
            }

            ctx = ctx.and_frameworks(test_flags);
        }

        ctx
    }

    fn ctx_for_rule<'a>(&self, ctx: &LintContext<'a>, rule: &RuleWithSeverity) -> LintContext<'a> {
        let rule_name = rule.name();
        let plugin_name = self.map_jest(rule.plugin_name(), rule_name);

        #[cfg(debug_assertions)]
        let ctx = ctx.clone().with_rule_fix_capabilities(rule.rule.fix());
        #[cfg(not(debug_assertions))]
        let ctx = ctx.clone();

        ctx.with_plugin_name(plugin_name).with_rule_name(rule_name).with_severity(rule.severity)
    }

    fn map_jest(&self, plugin_name: &'static str, rule_name: &str) -> &'static str {
        if self.options.plugins.has_vitest()
            && plugin_name == "jest"
            && utils::is_jest_rule_adapted_to_vitest(rule_name)
        {
            "vitest"
        } else {
            plugin_name
        }
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
        use std::fs;

        use project_root::get_project_root;
        let path = get_project_root().unwrap().join("npm/oxlint/configuration_schema.json");
        let schema = schemars::schema_for!(OxlintConfig);
        let json = serde_json::to_string_pretty(&schema).unwrap();
        let existing_json = fs::read_to_string(&path).unwrap_or_default();
        if existing_json.trim() != json.trim() {
            std::fs::write(&path, &json).unwrap();
        }
        insta::with_settings!({ prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!(json);
        });
    }
}
