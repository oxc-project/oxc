#![expect(clippy::self_named_module_files)]
// for rules.rs
// RuleEnum contains rule configs with interior mutability (e.g. Regex),
// but Hash/Eq/Ord are based only on the rule id, so it's safe as a map key.
#![expect(clippy::mutable_key_type)]
// Rule::from_configuration returns Result but documenting errors is not useful here.
#![expect(clippy::missing_errors_doc)]

use std::{
    mem,
    path::Path,
    ptr::{self, NonNull},
    rc::Rc,
};

use oxc_allocator::{Allocator, AllocatorPool, CloneIn};
use oxc_ast::{ast::Program, ast_kind::AST_TYPE_MAX};
use oxc_ast_macros::ast;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_data_structures::box_macros::boxed_array;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::Span;

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
mod tsgolint;
mod utils;

pub mod loader;
pub mod rules;
pub mod table;

mod generated {
    #[cfg(debug_assertions)]
    mod assert_layouts;
    mod rule_runner_impls;
    pub mod rules_enum;
}

#[cfg(test)]
mod tester;

mod lint_runner;

pub use crate::config::plugins::normalize_plugin_name;
pub use crate::disable_directives::{
    DisableDirectives, DisableRuleComment, RuleCommentRule, RuleCommentType,
    create_unused_directives_diagnostics,
};
pub use crate::{
    config::{
        Config, ConfigBuilderError, ConfigStore, ConfigStoreBuilder, ESLintRule, LintIgnoreMatcher,
        LintPlugins, Oxlintrc, ResolvedLinterState,
    },
    context::{ContextSubHost, LintContext},
    external_linter::{
        ExternalLinter, ExternalLinterCreateWorkspaceCb, ExternalLinterDestroyWorkspaceCb,
        ExternalLinterLintFileCb, ExternalLinterLoadPluginCb, ExternalLinterSetupRuleConfigsCb,
        JsFix, LintFileResult, LoadPluginResult,
    },
    external_plugin_store::{ExternalOptionsId, ExternalPluginStore, ExternalRuleId},
    fixer::{Fix, FixKind, Message, PossibleFixes},
    frameworks::FrameworkFlags,
    lint_runner::{DirectivesStore, LintRunner, LintRunnerBuilder},
    loader::LINTABLE_EXTENSIONS,
    module_record::ModuleRecord,
    options::LintOptions,
    options::{AllowWarnDeny, InvalidFilterKind, LintFilter, LintFilterKind},
    rule::{RuleCategory, RuleFixMeta, RuleMeta, RuleRunFunctionsImplemented, RuleRunner},
    service::{LintService, LintServiceOptions, OsFileSystem, RuntimeFileSystem},
    tsgolint::TsGoLintState,
    utils::{read_to_arena_str, read_to_string},
};
use crate::{
    config::{LintConfig, OxlintEnv, OxlintGlobals, OxlintSettings},
    context::ContextHost,
    external_linter::GlobalsAndEnvs,
    fixer::{CompositeFix, Fixer},
    loader::LINT_PARTIAL_LOADER_EXTENSIONS,
    rules::RuleEnum,
    utils::iter_possible_jest_call_node,
};

#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    // `RuleEnum` runs in a really tight loop, make sure it is small for CPU cache.
    // A reduction from 168 bytes to 16 results 15% performance improvement.
    // See codspeed in https://github.com/oxc-project/oxc/pull/1783
    assert_eq!(size_of::<RuleEnum>(), 16);
}

/// Base URL for the documentation, used to generate rule documentation URLs when a diagnostic is reported.
const WEBSITE_BASE_RULES_URL: &str = "https://oxc.rs/docs/guide/usage/linter/rules";

#[derive(Debug)]
#[expect(clippy::struct_field_names)]
pub struct Linter {
    options: LintOptions,
    config: ConfigStore,
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
    pub fn number_of_rules(&self, type_aware: bool) -> Option<usize> {
        self.config.number_of_rules(type_aware)
    }

    /// Return `true` if `Linter` has an external linter (JS plugins).
    pub fn has_external_linter(&self) -> bool {
        self.external_linter.is_some()
    }

    /// # Panics
    /// Panics if running in debug mode and the number of diagnostics does not match when running with/without optimizations
    pub fn run<'a>(
        &self,
        path: &Path,
        context_sub_hosts: Vec<ContextSubHost<'a>>,
        allocator: &'a Allocator,
    ) -> Vec<Message> {
        self.run_with_disable_directives(path, context_sub_hosts, allocator, None).0
    }

    /// Same as `run` but also returns the disable directives for the file
    ///
    /// # Parameters
    /// - `js_allocator_pool`: Optional pool of fixed-size allocators for copying AST before JS transfer.
    ///   When `Some`, the AST will be copied into a fixed-size allocator before passing to JS plugins,
    ///   allowing the main allocator to be a standard (non-fixed-size) allocator.
    ///
    /// # Panics
    /// Panics in debug mode if running with and without optimizations produces different diagnostic counts.
    pub fn run_with_disable_directives<'a>(
        &self,
        path: &Path,
        context_sub_hosts: Vec<ContextSubHost<'a>>,
        allocator: &'a Allocator,
        js_allocator_pool: Option<&AllocatorPool>,
    ) -> (Vec<Message>, Option<DisableDirectives>) {
        let ResolvedLinterState { rules, config, external_rules } = self.config.resolve(path);

        let mut ctx_host = Rc::new(ContextHost::new(path, context_sub_hosts, self.options, config));

        #[cfg(debug_assertions)]
        let mut current_diagnostic_index = 0;

        let is_partial_loader_file = ctx_host
            .file_extension()
            .is_some_and(|ext| LINT_PARTIAL_LOADER_EXTENSIONS.iter().any(|e| e == &ext));

        loop {
            let semantic = ctx_host.semantic();
            let rules = rules
                .iter()
                .filter(|(rule, _)| {
                    if rule.is_tsgolint_rule() {
                        return false;
                    }

                    // If only the `run` function is implemented, we can skip running the file entirely if the current
                    // file does not contain any of the relevant AST node types.
                    if rule.run_info() == RuleRunFunctionsImplemented::Run
                        && let Some(ast_types) = rule.types_info()
                        && !semantic.nodes().contains_any(ast_types)
                    {
                        return false;
                    }

                    rule.should_run(&ctx_host)
                })
                .map(|(rule, severity)| (rule, Rc::clone(&ctx_host).spawn(rule, *severity)))
                .collect::<Vec<_>>();

            let should_run_on_jest_node =
                ctx_host.plugins().has_test() && ctx_host.frameworks().is_test();

            let execute_rules = |with_runtime_optimization: bool| {
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
                    // TODO: It seems like there is probably a more intelligent way to preallocate space here. This will
                    // likely incur quite a few unnecessary reallocs currently. We theoretically could compute this at
                    // compile-time since we know all of the rules and their AST node type information ahead of time.
                    //
                    // Use boxed array to help compiler see that indexing into it with an `AstType`
                    // cannot go out of bounds, and remove bounds checks.
                    let mut rules_by_ast_type = boxed_array![Vec::new(); AST_TYPE_MAX as usize + 1];
                    // TODO: Compute needed capacity. This is a slight overestimate as not 100% of rules will need to run on all
                    // node types, but it at least guarantees we won't need to realloc.
                    let mut rules_any_ast_type = Vec::with_capacity(rules.len());

                    for (rule, ctx) in &rules {
                        let rule = *rule;
                        let run_info = rule.run_info();
                        // Collect node type information for rules. In large files, benchmarking showed it was worth
                        // collecting rules into buckets by AST node type to avoid iterating over all rules for each node.
                        if with_runtime_optimization
                            && let Some(ast_types) = rule.types_info()
                            && run_info.is_run_implemented()
                        {
                            for ty in ast_types {
                                rules_by_ast_type[ty as usize].push((rule, ctx));
                            }
                        } else {
                            rules_any_ast_type.push((rule, ctx));
                        }

                        if !with_runtime_optimization || run_info.is_run_once_implemented() {
                            rule.run_once(ctx);
                        }
                    }

                    // Run rules on nodes
                    for node in semantic.nodes() {
                        for (rule, ctx) in &rules_by_ast_type[node.kind().ty() as usize] {
                            rule.run(node, ctx);
                        }
                        for (rule, ctx) in &rules_any_ast_type {
                            rule.run(node, ctx);
                        }
                    }

                    if should_run_on_jest_node {
                        for jest_node in iter_possible_jest_call_node(semantic) {
                            for (rule, ctx) in &rules {
                                if !with_runtime_optimization
                                    || rule.run_info().is_run_on_jest_node_implemented()
                                {
                                    rule.run_on_jest_node(&jest_node, ctx);
                                }
                            }
                        }
                    }
                } else {
                    for (rule, ctx) in &rules {
                        let run_info = rule.run_info();
                        if !with_runtime_optimization || run_info.is_run_once_implemented() {
                            rule.run_once(ctx);
                        }

                        if !with_runtime_optimization || run_info.is_run_implemented() {
                            // For smaller files, benchmarking showed it was faster to iterate over all rules and just check the
                            // node types as we go, rather than pre-bucketing rules by AST node type and doing extra allocations.
                            if with_runtime_optimization && let Some(ast_types) = rule.types_info()
                            {
                                for node in semantic.nodes() {
                                    if ast_types.has(node.kind().ty()) {
                                        rule.run(node, ctx);
                                    }
                                }
                            } else {
                                for node in semantic.nodes() {
                                    rule.run(node, ctx);
                                }
                            }
                        }

                        if should_run_on_jest_node
                            && (!with_runtime_optimization
                                || run_info.is_run_on_jest_node_implemented())
                        {
                            for jest_node in iter_possible_jest_call_node(semantic) {
                                rule.run_on_jest_node(&jest_node, ctx);
                            }
                        }
                    }
                }
            };

            execute_rules(true);

            #[cfg(debug_assertions)]
            {
                let diagnostics_after_optimized = ctx_host.diagnostic_count();
                execute_rules(false);
                let diagnostics_after_unoptimized = ctx_host.diagnostic_count();
                ctx_host.get_diagnostics(|diagnostics| {
                    let optimized_diagnostics = &diagnostics[current_diagnostic_index..diagnostics_after_optimized];
                    let unoptimized_diagnostics = &diagnostics[diagnostics_after_optimized..diagnostics_after_unoptimized];

                    // Check that we have the same number of diagnostics
                    assert_eq!(
                        optimized_diagnostics.len(),
                        unoptimized_diagnostics.len(),
                        "Running with and without optimizations produced different diagnostic counts: {} vs {}.\nThis can be caused by a mismatch between the rule definition and generated RuleRunner impl. Try `cargo run -p oxc_linter_codegen` to regenerate.",
                        optimized_diagnostics.len(),
                        unoptimized_diagnostics.len()
                    );


                    let mut sorted_optimized = optimized_diagnostics.to_vec();
                    let mut sorted_unoptimized = unoptimized_diagnostics.to_vec();
                    let sort = |m: &Message| { (m.error.labels.as_ref().and_then(|l| l.first()).map(|l| (l.offset(), l.len())), m.error.code.clone()) };
                    sorted_optimized.sort_unstable_by_key(sort);
                    sorted_unoptimized.sort_unstable_by_key(sort);

                    for (opt_diag, unopt_diag) in sorted_optimized.iter().zip(sorted_unoptimized.iter()){
                        assert_eq!(
                            opt_diag,
                            unopt_diag,
                            "Diagnostic differs between optimized and unoptimized runs",
                        );
                    }

                    diagnostics.truncate(current_diagnostic_index + optimized_diagnostics.len());
                });
            }

            // Drop `rules` to release its `Rc` clones of `ctx_host`, ensuring `run_external_rules`
            // can mutably access `ctx_host` via `Rc::get_mut` without panicking due to multiple references.
            drop(rules);

            self.run_external_rules(
                &external_rules,
                path,
                &mut ctx_host,
                allocator,
                js_allocator_pool,
            );

            // Report unused directives is now handled differently with type-aware linting

            if let Some(severity) = self.options.report_unused_directive
                && severity.is_warn_deny()
                && is_partial_loader_file
            {
                ctx_host.report_unused_directives(severity.into());
            }

            // no next `<script>` block found, the complete file is finished linting
            if !ctx_host.next_sub_host() {
                break;
            }

            #[cfg(debug_assertions)]
            {
                current_diagnostic_index = ctx_host.diagnostic_count();
            }
        }

        let diagnostics = ctx_host.take_diagnostics();
        let disable_directives = if is_partial_loader_file {
            None
        } else {
            Rc::try_unwrap(ctx_host).unwrap().into_disable_directives()
        };

        (diagnostics, disable_directives)
    }

    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    fn run_external_rules<'a>(
        &self,
        external_rules: &[(ExternalRuleId, ExternalOptionsId, AllowWarnDeny)],
        path: &Path,
        ctx_host: &mut Rc<ContextHost<'a>>,
        allocator: &'a Allocator,
        js_allocator_pool: Option<&AllocatorPool>,
    ) {
        if external_rules.is_empty() {
            return;
        }

        // Extract `Semantic` from `ContextHost`, and get a mutable reference to `Program`.
        //
        // It's not possible to obtain a `&mut Program` while `Semantic` exists, because `Semantic`
        // contains `AstNodes`, which contains `AstKind`s for every AST nodes, each of which contains
        // an immutable `&` ref to an AST node.
        // Obtaining a `&mut Program` while `Semantic` exists would be illegal aliasing.
        //
        // So instead we get a pointer to `Program`.
        // The pointer is obtained initially from `&Program` in `Semantic`, but that pointer
        // has no provenance for mutation, so can't be converted to `&mut Program`.
        // So create a new pointer to `Program` which inherits `data_end_ptr`'s provenance,
        // which does allow mutation.
        //
        // We then drop `Semantic`, after which no references to any AST nodes remain.
        // We can then safety convert the pointer to `&mut Program`.
        //
        // `Program` was created in `allocator`, and that allocator is a `FixedSizeAllocator`,
        // so only has 1 chunk. So `data_end_ptr` and `Program` are within the same allocation.
        // All callers of `Linter::run` obtain `allocator` and `Semantic` from `ModuleContent`,
        // which ensure they are in same allocation.
        // However, we have no static guarantee of this, so strictly speaking it's unsound.
        // TODO: It would be better to avoid the need for a `&mut Program` here, and so avoid this
        // sketchy behavior.
        let ctx_host = Rc::get_mut(ctx_host).unwrap();
        let semantic = mem::take(ctx_host.semantic_mut());
        let program_addr = NonNull::from(semantic.nodes().program()).addr();
        let mut program_ptr =
            allocator.data_end_ptr().cast::<Program<'a>>().with_addr(program_addr);
        drop(semantic);
        // SAFETY: Now that we've dropped `Semantic`, no references to any AST nodes remain,
        // so can get a mutable reference to `Program` without aliasing violations.
        let program = unsafe { program_ptr.as_mut() };

        // If `js_allocator_pool` is provided, use clone-into-fixed-allocator approach
        if let Some(js_allocator_pool) = js_allocator_pool {
            self.clone_into_fixed_size_allocator_and_run_external_rules(
                external_rules,
                path,
                ctx_host,
                program,
                js_allocator_pool,
            );
            return;
        }

        // `allocator` is a fixed-size allocator, so no need to clone AST into a new one
        self.convert_and_call_external_linter(external_rules, path, ctx_host, program, allocator);
    }

    #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
    fn run_external_rules<'a>(
        &self,
        _external_rules: &[(ExternalRuleId, ExternalOptionsId, AllowWarnDeny)],
        _path: &Path,
        _ctx_host: &mut Rc<ContextHost<'a>>,
        _allocator: &'a Allocator,
        _js_allocator_pool: Option<&AllocatorPool>,
    ) {
        // External rules (JS plugins) are not supported on non-64-bit or big-endian platforms
    }

    /// Clone AST into a fixed-size allocator and run external rules.
    ///
    /// This copies the AST and source text from the standard allocator into a fixed-size
    /// allocator before passing to JS plugins. This allows using standard allocators for
    /// parsing/linting while still supporting JS plugin raw transfer.
    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    fn clone_into_fixed_size_allocator_and_run_external_rules(
        &self,
        external_rules: &[(ExternalRuleId, ExternalOptionsId, AllowWarnDeny)],
        path: &Path,
        ctx_host: &ContextHost<'_>,
        original_program: &mut Program<'_>,
        js_allocator_pool: &AllocatorPool,
    ) {
        let js_allocator = js_allocator_pool.get();

        // Get the original source text from the `Program`, and replace it with an empty string.
        // This avoids cloning the original source text, which can be large.
        let original_source_text = original_program.source_text;
        original_program.source_text = "";

        // Copy source text to the START of the fixed-size allocator.
        // This is critical - the JS deserializer expects source text at offset 0.
        // SAFETY: `js_allocator` is from a fixed-size allocator pool, which wraps the allocator
        // in a custom `Drop` that doesn't actually drop it (it returns it to the pool), so the
        // memory remains valid. This matches the safety requirements of `alloc_bytes_start`.
        let new_source_text: &str = unsafe {
            let bytes = original_source_text.as_bytes();
            let ptr = js_allocator.alloc_bytes_start(bytes.len());
            ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.as_ptr(), bytes.len());
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr.as_ptr(), bytes.len()))
        };

        // Clone `Program` into fixed-size allocator.
        // We need to allocate the `Program` struct ITSELF in the allocator, not just its contents.
        // `clone_in` returns a value on the stack, but we need it in the allocator for raw transfer.
        let program = {
            let mut program = original_program.clone_in(&js_allocator);
            program.source_text = new_source_text;
            js_allocator.alloc(program)
        };

        self.convert_and_call_external_linter(
            external_rules,
            path,
            ctx_host,
            program,
            &js_allocator,
        );

        // The `AllocatorGuard` (`js_allocator`) is dropped here, returning the allocator to the pool.
        // This ensures that we never have too many allocators in play at once, avoiding OOM.
    }

    /// Convert spans to UTF-16, write metadata, call external linter, and process diagnostics.
    ///
    /// This is the common code path shared by both `run_external_rules` and
    /// `clone_into_fixed_size_allocator_and_run_external_rules`.
    fn convert_and_call_external_linter(
        &self,
        external_rules: &[(ExternalRuleId, ExternalOptionsId, AllowWarnDeny)],
        path: &Path,
        ctx_host: &ContextHost<'_>,
        program: &mut Program<'_>,
        allocator: &Allocator,
    ) {
        // If has BOM, remove it
        const BOM: &str = "\u{feff}";
        const BOM_LEN: usize = BOM.len();

        let mut source_text = program.source_text;
        let has_bom = source_text.starts_with(BOM);
        if has_bom {
            source_text = &source_text[BOM_LEN..];
            program.source_text = source_text;
        }

        // Convert spans to UTF-16.
        // If source starts with BOM, create converter which ignores the BOM.
        let span_converter = if has_bom {
            #[expect(clippy::cast_possible_truncation)]
            Utf8ToUtf16::new_with_offset(source_text, BOM_LEN as u32)
        } else {
            Utf8ToUtf16::new(source_text)
        };

        span_converter.convert_program(program);
        span_converter.convert_comments(&mut program.comments);

        // Get offset of `Program` within buffer (bottom 32 bits of pointer)
        let program_offset = ptr::from_ref(program) as u32;

        // Write offset of `Program` in metadata at end of buffer
        let is_ts = program.source_type.is_typescript();
        let is_jsx = program.source_type.is_jsx();
        let metadata = RawTransferMetadata::new(program_offset, is_ts, is_jsx, has_bom);
        let metadata_ptr = allocator.end_ptr().cast::<RawTransferMetadata>();
        // SAFETY: `Allocator` was created by `FixedSizeAllocator` which reserved space after `end_ptr`
        // for a `RawTransferMetadata`. `end_ptr` is aligned for `RawTransferMetadata`.
        unsafe { metadata_ptr.write(metadata) };

        let path = path.to_string_lossy();
        let path = path.as_ref();

        let settings_json = match &ctx_host.settings().json {
            Some(json) => serde_json::to_string(&json).unwrap_or_else(|e| {
                let message = format!("Error serializing settings.\nFile path: {path}\n{e}");
                ctx_host.push_diagnostic(Message::new(
                    OxcDiagnostic::error(message),
                    PossibleFixes::None,
                ));
                "{}".to_string()
            }),
            None => "{}".to_string(),
        };

        let globals_and_envs = GlobalsAndEnvs::new(ctx_host);
        let globals_json = serde_json::to_string(&globals_and_envs).unwrap_or_else(|e| {
            let message = format!("Error serializing globals.\nFile path: {path}\n{e}");
            ctx_host
                .push_diagnostic(Message::new(OxcDiagnostic::error(message), PossibleFixes::None));
            "{}".to_string()
        });

        // `external_linter` always exists when `external_rules` is not empty
        let external_linter = self.external_linter.as_ref().unwrap();

        // Pass AST and rule IDs + options IDs to JS
        let result = (external_linter.lint_file)(
            path.to_owned(),
            external_rules.iter().map(|(rule_id, _, _)| rule_id.raw()).collect(),
            external_rules.iter().map(|(_, options_id, _)| options_id.raw()).collect(),
            settings_json,
            globals_json,
            allocator,
        );
        match result {
            Ok(diagnostics) => {
                for diagnostic in diagnostics {
                    // Convert UTF-16 offsets back to UTF-8.
                    // TODO: Validate span offsets are within bounds and `start <= end`.
                    // Also make sure offsets do not fall in middle of a multi-byte UTF-8 character.
                    // That's possible if UTF-16 offset points to middle of a surrogate pair.
                    let mut span = Span::new(diagnostic.start, diagnostic.end);
                    span_converter.convert_span_back(&mut span);

                    let (external_rule_id, _options_id, severity) =
                        external_rules[diagnostic.rule_index as usize];
                    let (plugin_name, rule_name) =
                        self.config.resolve_plugin_rule_names(external_rule_id);

                    if ctx_host
                        .disable_directives()
                        .contains(&format!("{plugin_name}/{rule_name}"), span)
                    {
                        continue;
                    }

                    // Convert `JSFix`s fixes to `PossibleFixes`, including converting spans back to UTF-8
                    let fix = if let Some(fixes) = diagnostic.fixes {
                        debug_assert!(!fixes.is_empty()); // JS should send `None` instead of `Some([])`

                        let is_single = fixes.len() == 1;

                        let fixes = fixes.into_iter().map(|fix| {
                            // TODO: Validate span offsets are within bounds and `start <= end`.
                            // Also make sure offsets do not fall in middle of a multi-byte UTF-8 character.
                            // That's possible if UTF-16 offset points to middle of a surrogate pair.
                            let mut span = Span::new(fix.range[0], fix.range[1]);
                            span_converter.convert_span_back(&mut span);
                            Fix::new(fix.text, span).with_kind(FixKind::Fix)
                        });

                        if is_single {
                            PossibleFixes::Single(fixes.into_iter().next().unwrap())
                        } else {
                            let fixes = fixes.collect::<Vec<_>>();
                            match CompositeFix::merge_fixes_fallible(fixes, source_text) {
                                Ok(fix) => PossibleFixes::Single(fix),
                                Err(err) => {
                                    let message = format!(
                                        "Plugin `{plugin_name}/{rule_name}` returned invalid fixes.\nFile path: {path}\n{err}"
                                    );
                                    ctx_host.push_diagnostic(Message::new(
                                        OxcDiagnostic::error(message),
                                        PossibleFixes::None,
                                    ));
                                    PossibleFixes::None
                                }
                            }
                        }
                    } else {
                        PossibleFixes::None
                    };

                    ctx_host.push_diagnostic(Message::new(
                        OxcDiagnostic::error(diagnostic.message)
                            .with_label(span)
                            .with_error_code(plugin_name.to_string(), rule_name.to_string())
                            .with_severity(severity.into()),
                        fix,
                    ));
                }
            }
            Err(err) => {
                let message = format!("Error running JS plugin.\nFile path: {path}\n{err}");
                ctx_host.push_diagnostic(Message::new(
                    OxcDiagnostic::error(message),
                    PossibleFixes::None,
                ));
            }
        }
    }
}

/// Metadata written to end of buffer.
///
/// Duplicate of `RawTransferMetadata` in `napi/parser/src/raw_transfer_types.rs`.
/// Any changes made here also need to be made there.
/// `oxc_ast_tools` checks that the 2 copies are identical.
#[ast]
pub struct RawTransferMetadata2 {
    /// Offset of `Program` within buffer.
    /// Note: In `RawTransferMetadata` (in `napi/parser`), this field is offset of `RawTransferData`,
    /// but here it's offset of `Program`.
    pub data_offset: u32,
    /// `true` if AST is TypeScript.
    pub is_ts: bool,
    /// `true` if AST is JSX.
    pub is_jsx: bool,
    /// `true` if source text has a BOM.
    pub has_bom: bool,
    /// Padding to pad struct to size 16.
    pub(crate) _padding: u64,
}

use RawTransferMetadata2 as RawTransferMetadata;

impl RawTransferMetadata {
    pub fn new(data_offset: u32, is_ts: bool, is_jsx: bool, has_bom: bool) -> Self {
        Self { data_offset, is_ts, is_jsx, has_bom, _padding: 0 }
    }
}
