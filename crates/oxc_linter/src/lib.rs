#![allow(clippy::self_named_module_files)] // for rules.rs

#[cfg(test)]
mod tester;

mod ast_util;
mod builder;
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

pub mod loader;
pub mod table;

use std::{io::Write, path::Path, rc::Rc, sync::Arc};

use config::LintConfig;
use context::ContextHost;
use options::LintOptions;
use oxc_semantic::{AstNode, Semantic};
use utils::iter_possible_jest_call_node;

pub use crate::{
    builder::LinterBuilder,
    config::{LintPlugins, Oxlintrc},
    context::LintContext,
    fixer::FixKind,
    frameworks::FrameworkFlags,
    options::{AllowWarnDeny, InvalidFilterKind, LintFilter, LintFilterKind},
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
        LinterBuilder::default().build()
    }
}

impl Linter {
    pub(crate) fn new(
        rules: Vec<RuleWithSeverity>,
        options: LintOptions,
        config: LintConfig,
    ) -> Self {
        Self { rules, options, config: Arc::new(config) }
    }

    #[cfg(test)]
    #[must_use]
    pub fn with_rules(mut self, rules: Vec<RuleWithSeverity>) -> Self {
        self.rules = rules;
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

    #[cfg(test)]
    pub(crate) fn rules(&self) -> &Vec<RuleWithSeverity> {
        &self.rules
    }

    pub fn run<'a>(&self, path: &Path, semantic: Rc<Semantic<'a>>) -> Vec<Message<'a>> {
        let ctx_host =
            Rc::new(ContextHost::new(path, semantic, self.options, Arc::clone(&self.config)));

        let rules = self
            .rules
            .iter()
            .filter(|rule| rule.should_run(&ctx_host))
            .map(|rule| (rule, Rc::clone(&ctx_host).spawn(rule)));

        let semantic = ctx_host.semantic();

        let should_run_on_jest_node =
            self.config.plugins.has_test() && ctx_host.frameworks().is_test();

        // IMPORTANT: We have two branches here for performance reasons:
        //
        // 1) Branch where we iterate over each node, then each rule
        // 2) Branch where we iterate over each rule, then each node
        //
        // When the number of nodes is relatively small, most of them can fit
        // in the cache and we can save iterating over the rules multiple times.
        // But for large files, the number of nodes can be so large that it
        // starts to not fit into the cache and pushes out other data, like the rules.
        // So we end up thrashing the cache with each rule iteration. In this case,
        // it's better to put rules in the inner loop, as the rules data is smaller
        // and is more likely to fit in the cache.
        //
        // The threshold here is chosen to balance between performance improvement
        // from not iterating over rules multiple times, but also ensuring that we
        // don't thrash the cache too much. Feel free to tweak based on benchmarking.
        //
        // See https://github.com/oxc-project/oxc/pull/6600 for more context.
        if semantic.stats().nodes > 200_000 {
            // Collect rules into a Vec so that we can iterate over the rules multiple times
            let rules = rules.collect::<Vec<_>>();

            for (rule, ctx) in &rules {
                rule.run_once(ctx);
            }

            for symbol in semantic.symbols().symbol_ids() {
                for (rule, ctx) in &rules {
                    rule.run_on_symbol(symbol, ctx);
                }
            }

            for node in semantic.nodes() {
                for (rule, ctx) in &rules {
                    rule.run(node, ctx);
                }
            }

            if should_run_on_jest_node {
                for jest_node in iter_possible_jest_call_node(semantic) {
                    for (rule, ctx) in &rules {
                        rule.run_on_jest_node(&jest_node, ctx);
                    }
                }
            }
        } else {
            for (rule, ref ctx) in rules {
                rule.run_once(ctx);

                for symbol in semantic.symbols().symbol_ids() {
                    rule.run_on_symbol(symbol, ctx);
                }

                for node in semantic.nodes() {
                    rule.run(node, ctx);
                }

                if should_run_on_jest_node {
                    for jest_node in iter_possible_jest_call_node(semantic) {
                        rule.run_on_jest_node(&jest_node, ctx);
                    }
                }
            }
        }

        ctx_host.take_diagnostics()
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
}

#[cfg(test)]
mod test {
    use super::{Linter, Oxlintrc};

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
        let schema = schemars::schema_for!(Oxlintrc);
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
