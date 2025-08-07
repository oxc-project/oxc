#![expect(clippy::self_named_module_files)] // for rules.rs
#![allow(clippy::literal_string_with_formatting_args)]

use std::{path::Path, rc::Rc, sync::Arc};

use oxc_allocator::Allocator;
use oxc_semantic::{AstNode, Semantic};

#[cfg(all(feature = "oxlint2", not(feature = "disable_oxlint2")))]
use oxc_ast_macros::ast;

#[cfg(test)]
mod tester;

mod ast_util;
mod config;
mod context;
mod disable_directives;
mod external_linter;
mod external_plugin_store;
mod fixer;
mod frameworks;
mod globals;
mod module_graph_visitor;
mod module_record;
mod options;
mod rule;
mod service;
mod utils;

pub mod loader;
pub mod rules;
pub mod table;

#[cfg(all(feature = "oxlint2", not(feature = "disable_oxlint2")))]
mod generated {
    #[cfg(debug_assertions)]
    pub mod assert_layouts;
}

pub use crate::{
    config::{
        BuiltinLintPlugins, Config, ConfigBuilderError, ConfigStore, ConfigStoreBuilder,
        ESLintRule, LintPlugins, Oxlintrc, ResolvedLinterState,
    },
    context::LintContext,
    external_linter::{
        ExternalLinter, ExternalLinterLintFileCb, ExternalLinterLoadPluginCb, LintFileResult,
        PluginLoadResult,
    },
    external_plugin_store::{ExternalPluginStore, ExternalRuleId},
    fixer::FixKind,
    frameworks::FrameworkFlags,
    loader::LINTABLE_EXTENSIONS,
    module_record::ModuleRecord,
    options::LintOptions,
    options::{AllowWarnDeny, InvalidFilterKind, LintFilter, LintFilterKind},
    rule::{RuleCategory, RuleFixMeta, RuleMeta},
    service::{LintService, LintServiceOptions, RuntimeFileSystem},
    utils::read_to_arena_str,
    utils::read_to_string,
};
use crate::{
    config::{LintConfig, OxlintEnv, OxlintGlobals, OxlintSettings},
    context::ContextHost,
    fixer::{Fixer, Message},
    rules::RuleEnum,
    utils::iter_possible_jest_call_node,
};

#[cfg(feature = "language_server")]
pub use crate::fixer::{FixWithPosition, MessageWithPosition, PossibleFixesWithPosition};

#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    // `RuleEnum` runs in a really tight loop, make sure it is small for CPU cache.
    // A reduction from 168 bytes to 16 results 15% performance improvement.
    // See codspeed in https://github.com/oxc-project/oxc/pull/1783
    assert_eq!(size_of::<RuleEnum>(), 16);
}

#[derive(Debug, Clone)]
#[expect(clippy::struct_field_names)]
pub struct Linter {
    options: LintOptions,
    config: ConfigStore,
    #[cfg_attr(not(all(feature = "oxlint2", not(feature = "disable_oxlint2"))), expect(dead_code))]
    external_linter: Option<ExternalLinter>,
}

impl Linter {
    pub fn new(
        options: LintOptions,
        config: ConfigStore,
        external_linter: Option<ExternalLinter>,
    ) -> Self {
        Self { options, config, external_linter }
    }

    /// Set the kind of auto fixes to apply.
    #[must_use]
    pub fn with_fix(mut self, kind: FixKind) -> Self {
        self.options.fix = kind;
        self
    }

    #[must_use]
    pub fn with_report_unused_directives(mut self, report_config: Option<AllowWarnDeny>) -> Self {
        self.options.report_unused_directive = report_config;
        self
    }

    pub(crate) fn options(&self) -> &LintOptions {
        &self.options
    }

    /// Returns the number of rules that will are being used, unless there
    /// nested configurations in use, in which case it returns `None` since the
    /// number of rules depends on which file is being linted.
    pub fn number_of_rules(&self) -> Option<usize> {
        self.config.number_of_rules()
    }

    pub fn run<'a>(
        &self,
        path: &Path,
        semantic: Rc<Semantic<'a>>,
        module_record: Arc<ModuleRecord>,
        allocator: &Allocator,
    ) -> Vec<Message<'a>> {
        let ResolvedLinterState { rules, config, external_rules } = self.config.resolve(path);

        let ctx_host =
            Rc::new(ContextHost::new(path, semantic, module_record, self.options, config));

        let rules = rules
            .iter()
            .filter(|(rule, _)| rule.should_run(&ctx_host))
            .map(|(rule, severity)| (rule, Rc::clone(&ctx_host).spawn(rule, *severity)));

        let semantic = ctx_host.semantic();

        let should_run_on_jest_node =
            ctx_host.plugins().has_test() && ctx_host.frameworks().is_test();

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
        if semantic.nodes().len() > 200_000 {
            // Collect rules into a Vec so that we can iterate over the rules multiple times
            let rules = rules.collect::<Vec<_>>();

            for (rule, ctx) in &rules {
                rule.run_once(ctx);
            }

            for symbol in semantic.scoping().symbol_ids() {
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

                for symbol in semantic.scoping().symbol_ids() {
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

        #[cfg(all(feature = "oxlint2", not(feature = "disable_oxlint2")))]
        self.run_external_rules(&external_rules, path, semantic, &ctx_host, allocator);

        // Stop clippy complaining about unused vars
        #[cfg(not(all(feature = "oxlint2", not(feature = "disable_oxlint2"))))]
        let (_, _) = (external_rules, allocator);

        if let Some(severity) = self.options.report_unused_directive {
            if severity.is_warn_deny() {
                ctx_host.report_unused_directives(severity.into());
            }
        }

        ctx_host.take_diagnostics()
    }

    #[cfg(all(feature = "oxlint2", not(feature = "disable_oxlint2")))]
    fn run_external_rules(
        &self,
        external_rules: &[(ExternalRuleId, AllowWarnDeny)],
        path: &Path,
        semantic: &Semantic<'_>,
        ctx_host: &ContextHost,
        allocator: &Allocator,
    ) {
        use std::ptr;

        use oxc_diagnostics::OxcDiagnostic;
        use oxc_span::Span;

        use crate::fixer::PossibleFixes;

        if external_rules.is_empty() {
            return;
        }

        // `external_linter` always exists when `oxlint2` feature is enabled
        let external_linter = self.external_linter.as_ref().unwrap();

        // Write offset of `Program` in metadata at end of buffer
        let program = semantic.nodes().program();
        let program_offset = ptr::from_ref(program) as u32;

        let metadata = RawTransferMetadata::new(program_offset);
        let metadata_ptr = allocator.end_ptr().cast::<RawTransferMetadata>();
        // SAFETY: `Allocator` was created by `FixedSizeAllocator` which reserved space after `end_ptr`
        // for a `RawTransferMetadata`. `end_ptr` is aligned for `RawTransferMetadata`.
        unsafe { metadata_ptr.write(metadata) };

        // Pass AST and rule IDs to JS
        let result = (external_linter.lint_file)(
            path.to_str().unwrap().to_string(),
            external_rules.iter().map(|(rule_id, _)| rule_id.raw()).collect(),
            allocator,
        );
        match result {
            Ok(diagnostics) => {
                for diagnostic in diagnostics {
                    let (external_rule_id, severity) =
                        external_rules[diagnostic.rule_index as usize];
                    let (plugin_name, rule_name) =
                        self.config.resolve_plugin_rule_names(external_rule_id);

                    ctx_host.push_diagnostic(Message::new(
                        OxcDiagnostic::error(diagnostic.message)
                            .with_label(Span::new(diagnostic.loc.start, diagnostic.loc.end))
                            .with_error_code(plugin_name.to_string(), rule_name.to_string())
                            .with_severity(severity.into()),
                        PossibleFixes::None,
                    ));
                }
            }
            Err(_err) => {
                // TODO: report diagnostic
            }
        }
    }
}

#[cfg(all(feature = "oxlint2", not(feature = "disable_oxlint2")))]
/// Metadata written to end of buffer.
///
/// Duplicate of `RawTransferMetadata` in `napi/parser/src/raw_transfer_types.rs`.
/// Any changes made here also need to be made there.
/// `oxc_ast_tools` checks that the 2 copies are identical.
#[ast]
struct RawTransferMetadata2 {
    /// Offset of `Program` within buffer.
    /// Note: In `RawTransferMetadata` (in `napi/parser`), this field is offset of `RawTransferData`,
    /// but here it's offset of `Program`.
    pub data_offset: u32,
    /// `true` if AST is TypeScript.
    pub is_ts: bool,
    /// Padding to pad struct to size 16.
    pub(crate) _padding: u64,
}

#[cfg(all(feature = "oxlint2", not(feature = "disable_oxlint2")))]
use RawTransferMetadata2 as RawTransferMetadata;

#[cfg(all(feature = "oxlint2", not(feature = "disable_oxlint2")))]
impl RawTransferMetadata {
    pub fn new(data_offset: u32) -> Self {
        Self { data_offset, is_ts: false, _padding: 0 }
    }
}

#[cfg(test)]
mod test {
    use super::Oxlintrc;

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
