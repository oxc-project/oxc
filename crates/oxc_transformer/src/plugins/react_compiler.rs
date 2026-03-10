use oxc_allocator::{Box as ABox, TakeIn, Vec as AVec};
use oxc_ast::NONE;
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_react_compiler::{
    compiler_error::{CompilerError, CompilerErrorEntry, SourceLocation},
    entrypoint::{
        gating::{
            FunctionKind as GatingFunctionKind, GatingOutput, ParamInfo, ParentContext,
            TernaryWrap, build_gating_output,
        },
        imports::{ProgramContext, validate_restricted_imports},
        options::{CompilationMode, CompilerReactTarget, DynamicGatingOptions, PanicThreshold},
        pipeline::{resolve_output_mode, run_codegen, run_pipeline},
        program::{
            ErrorAction, find_directive_disabling_memoization, get_react_compiler_runtime_module,
            handle_compilation_error, has_memo_cache_function_import,
            parse_dynamic_gating_directive, should_compile_function,
        },
        suppression::{
            DEFAULT_ESLINT_SUPPRESSION_RULES, SuppressionRange,
            filter_suppressions_that_affect_function, find_program_suppressions,
            suppressions_to_compiler_error,
        },
    },
    hir::{
        NonLocalBinding, ReactFunctionType,
        build_hir::{LowerableFunction, collect_import_bindings, lower},
        environment::{CompilerOutputMode, Environment, EnvironmentConfig, ExternalFunction},
    },
    reactive_scopes::codegen_reactive_function::{CodegenOutput, OutlinedOutput},
};
use oxc_semantic::{NodeId, ScopeFlags, ScopeId, SymbolFlags};
use oxc_span::{Atom, SPAN, Span};
use oxc_traverse::BoundIdentifier;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserialize;

use crate::context::TraverseCtx;

/// Options for the React Compiler transform.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReactCompilerOptions {
    /// Whether to enable the React Compiler transform.
    pub enabled: bool,
    /// Compilation mode: "infer", "annotation", "all", "syntax"
    pub compilation_mode: Option<String>,
    /// Panic threshold: "all_errors", "critical_errors", "none"
    pub panic_threshold: Option<String>,
    /// Target React version: "react-17", "react-18", "react-19" (default)
    ///
    /// Controls which runtime module to import:
    /// - "react-17" / "react-18" -> "react-compiler-runtime" (npm package)
    /// - "react-19" (default)    -> "react/compiler-runtime" (from react namespace)
    pub target: Option<String>,
    /// ESLint suppression rules to check for when scanning for suppression comments.
    /// Defaults to `["react-hooks/rules-of-hooks", "react-hooks/exhaustive-deps"]`.
    pub eslint_suppression_rules: Option<Vec<String>>,
    /// Whether to bail on Flow suppression comments. Defaults to `true`.
    pub flow_suppressions: Option<bool>,
    /// Whether to validate hooks usage (Rules of Hooks).
    /// Defaults to `true`.
    pub validate_hooks_usage: Option<bool>,
    /// Whether to validate ref access during render.
    /// Defaults to `true`.
    pub validate_ref_access_during_render: Option<bool>,
    /// Whether to validate no setState in render.
    /// Defaults to `true`.
    pub validate_no_set_state_in_render: Option<bool>,
    /// Output mode: "client", "ssr", "lint"
    /// When not set, defaults to "client" (or "lint" if `no_emit` is true).
    pub output_mode: Option<String>,
    /// When true, the compiler still runs validation but does not emit compiled output.
    /// Equivalent to setting `output_mode` to "lint".
    pub no_emit: Option<bool>,
    /// Whether to ignore "use no forget" / "use no memo" directives.
    pub ignore_use_no_forget: Option<bool>,
    /// Custom opt-out directives (in addition to "use no memo" / "use no forget").
    pub custom_opt_out_directives: Option<Vec<String>>,
    /// Gating function config: `{ source, importSpecifierName }`.
    /// When set, emits gated output that wraps compiled + original functions.
    pub gating: Option<ExternalFunctionConfig>,
    /// Dynamic gating config: `{ source }`.
    /// When set, enables `use memo if(...)` directives.
    pub dynamic_gating: Option<DynamicGatingConfig>,
    /// Array of filename patterns to filter which files get compiled.
    /// When set, only files whose path contains at least one pattern will be compiled.
    pub sources: Option<Vec<String>>,
    /// Enable optional dependency tracking for optional chain expressions.
    /// Defaults to `true`.
    pub enable_optional_dependencies: Option<bool>,
    /// Enable transitive freezing of function expression captures.
    /// Defaults to `true`.
    pub enable_transitively_freeze_function_expressions: Option<bool>,
    /// Enable treating ref-like identifiers as refs for type inference.
    /// Defaults to `true`.
    pub enable_treat_ref_like_identifiers_as_refs: Option<bool>,
    /// Validate that useMemo/useCallback results are not void.
    /// Defaults to `true`.
    pub validate_no_void_use_memo: Option<bool>,
    /// Validate exhaustive memoization dependencies.
    /// Defaults to `true`.
    pub validate_exhaustive_memoization_dependencies: Option<bool>,
}

/// Configuration for an external function import (gating, instrumentation, etc.).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ExternalFunctionConfig {
    /// The module source to import from.
    pub source: String,
    /// The import specifier name.
    pub import_specifier_name: String,
}

/// Configuration for dynamic gating via `use memo if(...)` directives.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DynamicGatingConfig {
    /// The module source to import from.
    pub source: String,
}

/// React Compiler transformer plugin.
///
/// Runs the React Compiler analysis and codegen pipeline in the transformer,
/// replacing compiled functions in the output AST and injecting the
/// `react/compiler-runtime` import when memoization is used.
pub struct ReactCompiler {
    options: ReactCompilerOptions,
    panic_threshold: PanicThreshold,
    /// The runtime module name for the target (e.g. "react/compiler-runtime").
    /// Computed once from `target` and stored to avoid repeated allocation.
    runtime_module: String,
    environment_config: EnvironmentConfig,
    output_mode: CompilerOutputMode,
    ignore_use_no_forget: bool,
    custom_opt_out_directives: Option<Vec<String>>,
    has_module_scope_opt_out: bool,
    gating: Option<ExternalFunction>,
    dynamic_gating: Option<DynamicGatingOptions>,
    program_context: ProgramContext,
    outer_bindings: FxHashMap<String, NonLocalBinding>,
    suppressions: Vec<SuppressionRange>,
}

/// Result of compiling a single function.
struct CompileResult<'a> {
    /// Index of the statement in `program.body` that was compiled.
    index: usize,
    /// The codegen output for the compiled function.
    output: CodegenOutput<'a>,
    /// The name of the original function (if any), used for gating.
    function_name: Option<String>,
    /// For multi-declarator VariableDeclarations, which declarator this result
    /// corresponds to. `None` for non-VarDecl statements.
    declarator_index: Option<usize>,
}

impl ReactCompiler {
    pub fn new(options: ReactCompilerOptions) -> Self {
        let panic_threshold = parse_panic_threshold(options.panic_threshold.as_deref());
        let target = parse_target(options.target.as_deref());
        let output_mode = resolve_output_mode(
            parse_output_mode(options.output_mode.as_deref()),
            options.no_emit.unwrap_or(false),
        );
        let ignore_use_no_forget = options.ignore_use_no_forget.unwrap_or(false);
        let custom_opt_out_directives = options.custom_opt_out_directives.clone();
        let gating = options.gating.as_ref().map(|g| ExternalFunction {
            source: g.source.clone(),
            import_specifier_name: g.import_specifier_name.clone(),
        });
        let dynamic_gating = options
            .dynamic_gating
            .as_ref()
            .map(|d| DynamicGatingOptions { source: d.source.clone() });
        let mut environment_config = EnvironmentConfig::default();
        if let Some(v) = options.validate_hooks_usage {
            environment_config.validate_hooks_usage = v;
        }
        if let Some(v) = options.validate_ref_access_during_render {
            environment_config.validate_ref_access_during_render = v;
        }
        if let Some(v) = options.validate_no_set_state_in_render {
            environment_config.validate_no_set_state_in_render = v;
        }
        if let Some(v) = options.enable_optional_dependencies {
            environment_config.enable_optional_dependencies = v;
        }
        if let Some(v) = options.enable_transitively_freeze_function_expressions {
            environment_config.enable_transitively_freeze_function_expressions = v;
        }
        if let Some(v) = options.enable_treat_ref_like_identifiers_as_refs {
            environment_config.enable_treat_ref_like_identifiers_as_refs = v;
        }
        if let Some(v) = options.validate_no_void_use_memo {
            environment_config.validate_no_void_use_memo = v;
        }
        if let Some(v) = options.validate_exhaustive_memoization_dependencies {
            environment_config.validate_exhaustive_memoization_dependencies = v;
        }
        let runtime_module = get_react_compiler_runtime_module(&target).to_string();
        Self {
            options,
            panic_threshold,
            runtime_module,
            environment_config,
            output_mode,
            ignore_use_no_forget,
            custom_opt_out_directives,
            has_module_scope_opt_out: false,
            gating,
            dynamic_gating,
            program_context: ProgramContext::new(),
            outer_bindings: FxHashMap::default(),
            suppressions: Vec::new(),
        }
    }

    pub fn enter_program<'a>(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.options.enabled {
            return;
        }

        // Sources filtering: if `sources` is configured, only compile files
        // whose path matches at least one source pattern (substring match).
        // When not configured, exclude node_modules by default (matching TS upstream).
        let source_path = ctx.state.source_path.to_string_lossy();
        if let Some(ref sources) = self.options.sources {
            if source_path.is_empty() {
                // TS upstream: "Expected a filename but found none."
                let diagnostic = OxcDiagnostic::error(
                    "React Compiler: Expected a filename but found none. When the 'sources' config option is specified, the React compiler will only compile files with a name",
                );
                ctx.state.error(diagnostic);
                return;
            }
            if !sources.iter().any(|pattern| source_path.contains(pattern.as_str())) {
                return;
            }
        } else {
            // Default: exclude node_modules (matches TS upstream behavior where
            // the default sources filter is `(fn) => fn.indexOf("node_modules") === -1`).
            if source_path.contains("node_modules") {
                return;
            }
        }

        // Check for already-compiled marker import: `import { c } from "<runtime_module>"`.
        // If found, skip compilation entirely to prevent double-compilation.
        // Port of `hasMemoCacheFunctionImport` from Program.ts.
        if has_memo_cache_function_import(&program.body, &self.runtime_module) {
            return;
        }

        // Check for module-level opt-out directives.
        // Port of TS Program.ts line 411-413: set hasModuleScopeOptOut flag but
        // continue compilation so that validation/lint errors are still reported.
        // The compiled results are discarded post-compilation if this flag is set.
        if !self.ignore_use_no_forget {
            let program_directives: Vec<String> =
                program.directives.iter().map(|d| d.directive.to_string()).collect();
            self.has_module_scope_opt_out = find_directive_disabling_memoization(
                &program_directives,
                self.custom_opt_out_directives.as_deref(),
            )
            .is_some();
        }

        // Validate restricted imports (port of validateRestrictedImports call in Program.ts).
        if let Some(error) = validate_restricted_imports(
            &program.body,
            self.environment_config.validate_blocklisted_imports.as_deref(),
        ) {
            Self::report_compiler_error(&error, program.span, self.panic_threshold, ctx);
            return;
        }

        // Find program-level suppression ranges from eslint-disable comments.
        // Port of findProgramSuppressions call in Program.ts (lines 396-400).
        // When both validateExhaustiveMemoizationDependencies and validateHooksUsage
        // are enabled, pass None (the compiler's own validation handles those cases).
        let suppress_rules = if self.environment_config.validate_exhaustive_memoization_dependencies
            && self.environment_config.validate_hooks_usage
        {
            None
        } else {
            let rule_names: Vec<String> =
                self.options.eslint_suppression_rules.clone().unwrap_or_else(|| {
                    DEFAULT_ESLINT_SUPPRESSION_RULES.iter().map(|s| (*s).to_string()).collect()
                });
            Some(rule_names)
        };
        self.suppressions = find_program_suppressions(
            &program.comments,
            program.source_text,
            suppress_rules.as_deref(),
            self.options.flow_suppressions.unwrap_or(true),
        );

        self.outer_bindings = collect_import_bindings(&program.body);

        // Seed ProgramContext.known_referenced_names with all top-level binding
        // names so that ProgramContext::new_uid avoids collisions with real scope
        // bindings. This matches upstream Imports.ts which receives the program
        // scope and checks it for existing names.
        let root_scope_id = ctx.scoping().root_scope_id();
        for (name, _) in ctx.scoping().get_bindings(root_scope_id) {
            self.program_context.add_reference(name);
        }

        // Pre-generate the cache function UID before compiling any functions.
        // This ensures the same name (e.g. "_c" or "_c2") is used in both the
        // import binding and the codegen body references.
        let cache_binding = ctx.generate_uid_in_root_scope("c", SymbolFlags::Import);
        let cache_identifier_name = cache_binding.name.to_string();

        // Phase 1: Compile all candidate functions, collecting results by statement index.
        let mut compiled_results: Vec<CompileResult<'a>> = Vec::new();
        for (index, statement) in program.body.iter().enumerate() {
            let results = self.compile_statement(statement, &cache_identifier_name, ctx);
            for (output, function_name, declarator_index) in results {
                compiled_results.push(CompileResult {
                    index,
                    output,
                    function_name,
                    declarator_index,
                });
            }
        }

        // Track whether any compiled function needs memo import (updated across phases).
        let mut needs_memo_import = compiled_results.iter().any(|r| r.output.memo_slots_used > 0);

        let has_top_level_results = !compiled_results.is_empty();

        // Pre-gating: compute referenced-before-declared set if gating is active.
        let referenced_before_declared = if self.gating.is_some() && has_top_level_results {
            Some(get_functions_referenced_before_declaration(&compiled_results, &program.body))
        } else {
            None
        };

        // Track whether any gating was actually applied (for import injection).
        let mut gating_was_used = false;

        // Phase 2 (conditional): Rebuild program.body, replacing compiled functions
        // and inserting outlined functions after the replaced statement.
        // Track which indices in new_body correspond to replaced/outlined statements
        // that need scope IDs assigned.
        let mut replaced_indices: Vec<usize> = Vec::new();
        // Track which indices in new_body correspond to compiled top-level statements
        // (used by nested discovery to skip already-compiled functions).
        let mut compiled_stmt_indices: FxHashSet<usize> = FxHashSet::default();

        let mut new_body = if has_top_level_results {
            type ResultEntry<'b> = (CodegenOutput<'b>, Option<String>, Option<usize>);
            let mut result_map: FxHashMap<usize, Vec<ResultEntry<'a>>> = FxHashMap::default();
            for result in compiled_results {
                result_map.entry(result.index).or_default().push((
                    result.output,
                    result.function_name,
                    result.declarator_index,
                ));
            }

            let old_body = program.body.take_in(ctx.ast);
            let mut new_body = ctx.ast.vec_with_capacity(old_body.len());

            for (i, stmt) in old_body.into_iter().enumerate() {
                if let Some(results_for_stmt) = result_map.remove(&i) {
                    // Pre-compute gating directive info once per statement (shared
                    // across all results for this statement index).
                    let stmt_gating_directive =
                        if self.gating.is_some() || self.dynamic_gating.is_some() {
                            let directives = get_statement_directives(&stmt);
                            match extract_dynamic_gating_directive(
                                &directives,
                                self.dynamic_gating.as_ref(),
                            ) {
                                Ok(dg) => Some(dg),
                                Err(errors) => {
                                    for msg in &errors {
                                        let diagnostic =
                                            OxcDiagnostic::error(format!("React Compiler: {msg}"));
                                        ctx.state.error(diagnostic.with_label(SPAN));
                                    }
                                    Some(None)
                                }
                            }
                        } else {
                            None
                        };

                    // Collect all outlined functions across all results for this statement.
                    let mut all_outlined: Vec<OutlinedOutput<'a>> = Vec::new();
                    // Use Option to allow gating to take ownership of the statement.
                    let mut stmt_opt = Some(stmt);

                    for (mut compiled, fn_name, declarator_index) in results_for_stmt {
                        // Extract outlined functions before consuming compiled for replacement.
                        all_outlined.extend(std::mem::take(&mut compiled.outlined));

                        // Determine if gating should be applied for this function.
                        // For VarDecl declarators (declarator_index.is_some()), compute
                        // gating per-declarator since each function can have different
                        // directives. For non-VarDecl results, use the pre-computed
                        // stmt_gating_directive.
                        let gating_output = if let Some(decl_idx) = declarator_index {
                            // Per-declarator gating for VarDecl results.
                            if self.gating.is_some() || self.dynamic_gating.is_some() {
                                let stmt_ref = stmt_opt.as_ref().expect(
                                    "stmt should not have been consumed before gating check",
                                );
                                let directives = get_declarator_directives(stmt_ref, decl_idx);
                                let dynamic_gating = match extract_dynamic_gating_directive(
                                    &directives,
                                    self.dynamic_gating.as_ref(),
                                ) {
                                    Ok(dg) => dg,
                                    Err(errors) => {
                                        for msg in &errors {
                                            let diagnostic = OxcDiagnostic::error(format!(
                                                "React Compiler: {msg}"
                                            ));
                                            ctx.state.error(diagnostic.with_label(SPAN));
                                        }
                                        None
                                    }
                                };
                                let effective_gating =
                                    dynamic_gating.as_ref().or(self.gating.as_ref());
                                if let Some(gating) = effective_gating {
                                    let fn_kind = get_declarator_function_kind(stmt_ref, decl_idx);
                                    let parent_context = get_gating_parent_context(stmt_ref);
                                    let is_ref_before_decl = referenced_before_declared
                                        .as_ref()
                                        .is_some_and(|set| set.contains(&i));
                                    let param_info = get_declarator_param_info(stmt_ref, decl_idx);
                                    Some(build_gating_output(
                                        fn_name.as_deref(),
                                        fn_kind,
                                        parent_context,
                                        gating,
                                        is_ref_before_decl,
                                        &param_info,
                                        &mut self.program_context,
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            // Non-VarDecl: use pre-computed stmt_gating_directive.
                            if let Some(ref dynamic_gating) = stmt_gating_directive {
                                let effective_gating =
                                    dynamic_gating.as_ref().or(self.gating.as_ref());
                                if let Some(gating) = effective_gating {
                                    let stmt_ref = stmt_opt.as_ref().expect(
                                        "stmt should not have been consumed before gating check",
                                    );
                                    let fn_kind = get_gating_function_kind(stmt_ref);
                                    let parent_context = get_gating_parent_context(stmt_ref);
                                    let is_ref_before_decl = referenced_before_declared
                                        .as_ref()
                                        .is_some_and(|set| set.contains(&i));
                                    let param_info = get_statement_param_info(stmt_ref);
                                    Some(build_gating_output(
                                        fn_name.as_deref(),
                                        fn_kind,
                                        parent_context,
                                        gating,
                                        is_ref_before_decl,
                                        &param_info,
                                        &mut self.program_context,
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        };

                        if let Some(gating_output) = gating_output {
                            gating_was_used = true;

                            if let Some(decl_idx) = declarator_index {
                                // In-place ternary gating for VarDecl declarators.
                                // Replaces just the specific declarator's init with the
                                // ternary, preserving the statement for other declarators.
                                // This matches upstream where fnPath.replaceWith() only
                                // replaces the function expression, not the enclosing statement.
                                let gating_fn_name = match &gating_output {
                                    GatingOutput::Ternary { gating_fn_name, .. }
                                    | GatingOutput::Hoisted { gating_fn_name, .. } => {
                                        gating_fn_name
                                    }
                                };
                                let stmt_ref = stmt_opt.as_mut().expect(
                                    "stmt should not have been consumed before in-place gating",
                                );
                                apply_ternary_gating_in_place(
                                    stmt_ref,
                                    compiled,
                                    gating_fn_name,
                                    decl_idx,
                                    ctx,
                                );
                                // Continue to process remaining results for this statement.
                            } else {
                                // Statement-level gating: consumes the entire statement.
                                let gating_stmt = stmt_opt.take().expect(
                                    "stmt should not have been consumed before gating replacement",
                                );
                                match &gating_output {
                                    GatingOutput::Ternary { gating_fn_name, wrap } => {
                                        let replacement_stmts = build_ternary_gating_replacement(
                                            gating_stmt,
                                            compiled,
                                            gating_fn_name,
                                            wrap,
                                            ctx,
                                        );
                                        for replacement in replacement_stmts {
                                            let new_idx = new_body.len();
                                            replaced_indices.push(new_idx);
                                            compiled_stmt_indices.insert(new_idx);
                                            new_body.push(replacement);
                                        }
                                    }
                                    GatingOutput::Hoisted { .. } => {
                                        let replacement_stmts = build_hoisted_gating_replacement(
                                            gating_stmt,
                                            compiled,
                                            &gating_output,
                                            ctx,
                                        );
                                        for replacement in replacement_stmts {
                                            let new_idx = new_body.len();
                                            replaced_indices.push(new_idx);
                                            compiled_stmt_indices.insert(new_idx);
                                            new_body.push(replacement);
                                        }
                                    }
                                }
                                break;
                            }
                        } else {
                            // No gating: mutate the statement in-place for this declarator.
                            let stmt_ref = stmt_opt
                                .as_mut()
                                .expect("stmt should not have been consumed in non-gating path");
                            replace_statement_function_in_place(
                                stmt_ref,
                                compiled,
                                declarator_index,
                                ctx,
                            );
                        }
                    }

                    if let Some(final_stmt) = stmt_opt {
                        // Push the (potentially mutated) statement.
                        let new_idx = new_body.len();
                        replaced_indices.push(new_idx);
                        compiled_stmt_indices.insert(new_idx);
                        new_body.push(final_stmt);
                    }

                    // Process outlined functions using a queue, matching TS Program.ts
                    // lines 426-454. Outlined functions with fn_type are requeued for
                    // full compilation; their own outlined outputs are then processed.
                    let mut outlined_queue: Vec<OutlinedOutput<'a>> = all_outlined;
                    while let Some(outlined) = outlined_queue.pop() {
                        let requeue_fn_type = outlined.fn_type;
                        let mut outlined_stmt = build_outlined_function_statement(outlined, ctx);
                        if let Some(fn_type) = requeue_fn_type
                            && let Some(mut recompiled) = self.compile_outlined_function(
                                &outlined_stmt,
                                fn_type,
                                &cache_identifier_name,
                                ctx,
                            )
                        {
                            // Collect outlined functions from the requeued compilation
                            // and add them to the queue for processing.
                            let nested_outlined = std::mem::take(&mut recompiled.outlined);
                            outlined_stmt =
                                replace_statement_function(outlined_stmt, recompiled, ctx);
                            for nested in nested_outlined {
                                debug_assert!(
                                    nested.fn_.outlined.is_empty(),
                                    "Unexpected nested outlined functions",
                                );
                                outlined_queue.push(nested);
                            }
                        }
                        let outlined_idx = new_body.len();
                        replaced_indices.push(outlined_idx);
                        compiled_stmt_indices.insert(outlined_idx);
                        new_body.push(outlined_stmt);
                    }
                } else {
                    new_body.push(stmt);
                }
            }
            new_body
        } else {
            // No top-level results — we still need to check for nested functions.
            // Use the existing body directly (take ownership for potential mutation).
            program.body.take_in(ctx.ast)
        };

        // Phase 3: Discover and compile nested functions within non-compiled statements.
        // Port of TS Program.ts `findFunctionsToCompile` which uses `program.traverse()`
        // to find ALL function nodes at any depth, not just top-level ones.
        //
        // Key behaviors matching TS:
        // - Skip class bodies (ClassDeclaration/ClassExpression)
        // - Skip walking into already-compiled function bodies (fn.skip() in TS)
        // - `alreadyCompiled` set prevents double-compilation
        // - In `all` mode, only top-level functions are compiled (TS Program.ts:501-508
        //   checks `fn.scope.getProgramParent() !== fn.scope.parent` and returns early).
        //   So we skip Phase 3 entirely in `all` mode.
        let compilation_mode = parse_compilation_mode(self.options.compilation_mode.as_deref());
        let mut compiled_any_nested = false;
        // In `all` mode, upstream only compiles top-level functions (Program.ts:501-508),
        // so skip Phase 3 nested discovery entirely.
        if compilation_mode != CompilationMode::All {
            let mut nested_scope_assign_indices: Vec<usize> = Vec::new();
            let mut nested_outlined: Vec<OutlinedOutput<'a>> = Vec::new();
            for (idx, stmt) in new_body.iter_mut().enumerate() {
                // Skip statements that were already compiled at the top level.
                // This matches TS `fn.skip()` — the compiler handles inner functions
                // of compiled functions internally.
                if compiled_stmt_indices.contains(&idx) {
                    continue;
                }
                if self.compile_nested_functions_in_statement(
                    stmt,
                    &cache_identifier_name,
                    &mut needs_memo_import,
                    &mut nested_outlined,
                    ctx,
                ) {
                    nested_scope_assign_indices.push(idx);
                    compiled_any_nested = true;
                }
            }
            // Assign scope IDs for nested compiled functions.
            for &idx in &nested_scope_assign_indices {
                let parent_scope_id = program.scope_id();
                assign_scope_ids_to_statement(&mut new_body[idx], parent_scope_id, ctx);
            }
            // Process outlined functions from nested compilations.
            // These are added as top-level statements, matching TS behavior
            // where outlined functions are always inserted at the program level.
            let mut outlined_queue: Vec<OutlinedOutput<'a>> = nested_outlined;
            while let Some(outlined) = outlined_queue.pop() {
                let requeue_fn_type = outlined.fn_type;
                let mut outlined_stmt = build_outlined_function_statement(outlined, ctx);
                if let Some(fn_type) = requeue_fn_type
                    && let Some(mut recompiled) = self.compile_outlined_function(
                        &outlined_stmt,
                        fn_type,
                        &cache_identifier_name,
                        ctx,
                    )
                {
                    let nested_outlined_inner = std::mem::take(&mut recompiled.outlined);
                    outlined_stmt = replace_statement_function(outlined_stmt, recompiled, ctx);
                    for nested in nested_outlined_inner {
                        outlined_queue.push(nested);
                    }
                }
                let outlined_idx = new_body.len();
                replaced_indices.push(outlined_idx);
                new_body.push(outlined_stmt);
            }
        } // end if not All mode

        // If nothing was compiled (neither top-level nor nested), restore the
        // program body and return without modifications.
        if !has_top_level_results && !compiled_any_nested {
            program.body = new_body;
            return;
        }

        // Phase 4: Inject `import { c as _c } from "<runtime_module>"` if any
        // compiled function (top-level or nested) uses memo slots.
        if needs_memo_import {
            ctx.state.module_imports.add_named_import(
                ctx.ast.atom(&self.runtime_module),
                Atom::from("c"),
                cache_binding,
                false,
            );
        }

        // Phase 4b: Inject gating function imports if gating was used.
        // Use the exact local names computed by ProgramContext::new_uid (which
        // already checked against real scope bindings via known_referenced_names)
        // instead of calling generate_uid_in_root_scope, which would rename them
        // and cause a mismatch with the names already emitted in gating codegen.
        if gating_was_used {
            // Clone the import data to avoid borrowing self while mutating ctx.
            let gating_imports: Vec<(String, Vec<(String, String)>)> = self
                .program_context
                .imports
                .iter()
                .map(|(source, specifiers)| {
                    (
                        source.clone(),
                        specifiers.iter().map(|s| (s.local.clone(), s.imported.clone())).collect(),
                    )
                })
                .collect();
            let root_scope_id = ctx.scoping().root_scope_id();
            for (source, specifiers) in gating_imports {
                for (local, imported) in specifiers {
                    // Create a binding in the root scope with the exact local
                    // name that ProgramContext already used in codegen output.
                    let name = oxc_span::Ident::from(ctx.ast.allocator.alloc_str(&local));
                    let symbol_id = ctx.scoping_mut().create_symbol(
                        SPAN,
                        name,
                        SymbolFlags::Import,
                        root_scope_id,
                        NodeId::DUMMY,
                    );
                    ctx.scoping_mut().add_binding(root_scope_id, name, symbol_id);
                    let local_binding = BoundIdentifier::new(name, symbol_id);
                    let source_atom = ctx.ast.atom(&source);
                    let imported_atom = ctx.ast.atom(&imported);
                    ctx.state.module_imports.add_named_import(
                        source_atom,
                        imported_atom,
                        local_binding,
                        false,
                    );
                }
            }
        }

        // Phase 5: Assign scope IDs to newly created AST nodes in compiled output.
        // The React Compiler codegen creates BlockStatement and other scope-creating
        // nodes without scope IDs. The subsequent transformer traversal (e.g. JSX
        // transform) requires all such nodes to have valid scope IDs.
        for &idx in &replaced_indices {
            let parent_scope_id =
                get_function_scope_id(&new_body[idx]).unwrap_or_else(|| program.scope_id());
            assign_scope_ids_to_statement(&mut new_body[idx], parent_scope_id, ctx);
        }

        program.body = new_body;
    }

    /// Try to compile the function(s) within a statement, returning the codegen
    /// output(s) and the original function name(s) on success.
    ///
    /// Returns a `Vec` of `(CodegenOutput, Option<String>, Option<usize>)` where
    /// the third element is the declarator index for multi-declarator VarDecl
    /// statements (`None` for non-VarDecl statements).
    fn compile_statement<'a>(
        &self,
        statement: &Statement<'a>,
        cache_identifier_name: &str,
        ctx: &mut TraverseCtx<'a>,
    ) -> Vec<(CodegenOutput<'a>, Option<String>, Option<usize>)> {
        match statement {
            Statement::FunctionDeclaration(function) => {
                let directives = function_directives(function);
                let fn_name = function.id.as_ref().map(|id| id.name.to_string());
                let lowerable_function = LowerableFunction::Function(function);
                self.compile_function(
                    &lowerable_function,
                    function.id.as_ref().map(|id| id.name.as_str()),
                    &directives,
                    function.span,
                    false,
                    cache_identifier_name,
                    ctx,
                )
                .map(|output| vec![(output, fn_name, None)])
                .unwrap_or_default()
            }
            Statement::VariableDeclaration(declaration) => {
                let mut results = Vec::new();
                for (decl_idx, declarator) in declaration.declarations.iter().enumerate() {
                    let binding_name = match &declarator.id {
                        BindingPattern::BindingIdentifier(identifier) => {
                            Some(identifier.name.as_str())
                        }
                        _ => None,
                    };

                    let Some(initializer) = &declarator.init else {
                        continue;
                    };

                    match initializer {
                        Expression::FunctionExpression(function) => {
                            let directives = function_directives(function);
                            let function_name =
                                function.id.as_ref().map(|id| id.name.as_str()).or(binding_name);
                            let fn_name_owned = function_name.map(str::to_string);
                            let lowerable_function = LowerableFunction::Function(function);
                            if let Some(output) = self.compile_function(
                                &lowerable_function,
                                function_name,
                                &directives,
                                function.span,
                                false,
                                cache_identifier_name,
                                ctx,
                            ) {
                                results.push((output, fn_name_owned, Some(decl_idx)));
                            }
                        }
                        Expression::ArrowFunctionExpression(arrow) => {
                            let directives = arrow_directives(arrow);
                            let fn_name_owned = binding_name.map(str::to_string);
                            let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                            if let Some(output) = self.compile_function(
                                &lowerable_function,
                                binding_name,
                                &directives,
                                arrow.span,
                                false,
                                cache_identifier_name,
                                ctx,
                            ) {
                                results.push((output, fn_name_owned, Some(decl_idx)));
                            }
                        }
                        Expression::CallExpression(call)
                            if is_memo_or_forwardref_call(&call.callee) =>
                        {
                            let fn_name_owned = binding_name.map(str::to_string);
                            if let Some(output) = self.compile_memo_or_forwardref_arg(
                                call,
                                binding_name,
                                cache_identifier_name,
                                ctx,
                            ) {
                                results.push((output, fn_name_owned, Some(decl_idx)));
                            }
                        }
                        _ => {}
                    }
                }
                results
            }
            Statement::ExportDefaultDeclaration(export_default) => {
                match &export_default.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(function)
                    | ExportDefaultDeclarationKind::FunctionExpression(function) => {
                        let directives = function_directives(function);
                        let fn_name = function.id.as_ref().map(|id| id.name.to_string());
                        let lowerable_function = LowerableFunction::Function(function);
                        self.compile_function(
                            &lowerable_function,
                            function.id.as_ref().map(|id| id.name.as_str()),
                            &directives,
                            function.span,
                            false,
                            cache_identifier_name,
                            ctx,
                        )
                        .map(|output| vec![(output, fn_name, None)])
                        .unwrap_or_default()
                    }
                    ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
                        let directives = arrow_directives(arrow);
                        let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                        self.compile_function(
                            &lowerable_function,
                            None,
                            &directives,
                            arrow.span,
                            false,
                            cache_identifier_name,
                            ctx,
                        )
                        .map(|output| vec![(output, None, None)])
                        .unwrap_or_default()
                    }
                    ExportDefaultDeclarationKind::CallExpression(call)
                        if is_memo_or_forwardref_call(&call.callee) =>
                    {
                        self.compile_memo_or_forwardref_arg(call, None, cache_identifier_name, ctx)
                            .map(|output| vec![(output, None, None)])
                            .unwrap_or_default()
                    }
                    _ => vec![],
                }
            }
            Statement::ExportNamedDeclaration(export_named) => {
                let Some(declaration) = &export_named.declaration else {
                    return vec![];
                };
                match declaration {
                    Declaration::FunctionDeclaration(function) => {
                        let directives = function_directives(function);
                        let fn_name = function.id.as_ref().map(|id| id.name.to_string());
                        let lowerable_function = LowerableFunction::Function(function);
                        self.compile_function(
                            &lowerable_function,
                            function.id.as_ref().map(|id| id.name.as_str()),
                            &directives,
                            function.span,
                            false,
                            cache_identifier_name,
                            ctx,
                        )
                        .map(|output| vec![(output, fn_name, None)])
                        .unwrap_or_default()
                    }
                    Declaration::VariableDeclaration(var_decl) => {
                        let mut results = Vec::new();
                        for (decl_idx, declarator) in var_decl.declarations.iter().enumerate() {
                            let binding_name = match &declarator.id {
                                BindingPattern::BindingIdentifier(identifier) => {
                                    Some(identifier.name.as_str())
                                }
                                _ => None,
                            };

                            let Some(initializer) = &declarator.init else {
                                continue;
                            };

                            match initializer {
                                Expression::FunctionExpression(function) => {
                                    let directives = function_directives(function);
                                    let function_name = function
                                        .id
                                        .as_ref()
                                        .map(|id| id.name.as_str())
                                        .or(binding_name);
                                    let fn_name_owned = function_name.map(str::to_string);
                                    let lowerable_function = LowerableFunction::Function(function);
                                    if let Some(output) = self.compile_function(
                                        &lowerable_function,
                                        function_name,
                                        &directives,
                                        function.span,
                                        false,
                                        cache_identifier_name,
                                        ctx,
                                    ) {
                                        results.push((output, fn_name_owned, Some(decl_idx)));
                                    }
                                }
                                Expression::ArrowFunctionExpression(arrow) => {
                                    let directives = arrow_directives(arrow);
                                    let fn_name_owned = binding_name.map(str::to_string);
                                    let lowerable_function =
                                        LowerableFunction::ArrowFunction(arrow);
                                    if let Some(output) = self.compile_function(
                                        &lowerable_function,
                                        binding_name,
                                        &directives,
                                        arrow.span,
                                        false,
                                        cache_identifier_name,
                                        ctx,
                                    ) {
                                        results.push((output, fn_name_owned, Some(decl_idx)));
                                    }
                                }
                                Expression::CallExpression(call)
                                    if is_memo_or_forwardref_call(&call.callee) =>
                                {
                                    let fn_name_owned = binding_name.map(str::to_string);
                                    if let Some(output) = self.compile_memo_or_forwardref_arg(
                                        call,
                                        binding_name,
                                        cache_identifier_name,
                                        ctx,
                                    ) {
                                        results.push((output, fn_name_owned, Some(decl_idx)));
                                    }
                                }
                                _ => {}
                            }
                        }
                        results
                    }
                    _ => vec![],
                }
            }
            _ => vec![],
        }
    }

    /// Run the full React Compiler pipeline (analysis + codegen) on a single function.
    ///
    /// Returns `Some(CodegenOutput)` on success, or `None` if the function should
    /// not be compiled or if compilation fails (errors are reported as diagnostics).
    fn compile_function<'a>(
        &self,
        function: &LowerableFunction<'_>,
        name: Option<&str>,
        directives: &[String],
        fallback_span: Span,
        is_memo_or_forwardref_arg: bool,
        cache_identifier_name: &str,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<CodegenOutput<'a>> {
        let fn_type = should_compile_function(
            function,
            name,
            directives,
            parse_compilation_mode(self.options.compilation_mode.as_deref()),
            is_memo_or_forwardref_arg,
            self.dynamic_gating.is_some(),
        )?;

        // Check if any eslint-disable suppression range covers this function.
        // Port of filterSuppressionsThatAffectFunction from Suppression.ts.
        let affecting_suppressions =
            filter_suppressions_that_affect_function(&self.suppressions, fallback_span);
        if !affecting_suppressions.is_empty() {
            let error = suppressions_to_compiler_error(&affecting_suppressions);
            Self::report_compiler_error(&error, fallback_span, self.panic_threshold, ctx);
            return None;
        }

        let environment =
            match Environment::new(fn_type, self.output_mode, self.environment_config.clone()) {
                Ok(env) => env,
                Err(error) => {
                    Self::report_compiler_error(&error, fallback_span, self.panic_threshold, ctx);
                    return None;
                }
            };

        let mut hir_function =
            match lower(&environment, fn_type, function, self.outer_bindings.clone()) {
                Ok(hir_function) => hir_function,
                Err(error) => {
                    Self::report_compiler_error(&error, fallback_span, self.panic_threshold, ctx);
                    return None;
                }
            };

        let pipeline_output = match run_pipeline(&mut hir_function, &environment) {
            Ok(output) => output,
            Err(error) => {
                Self::report_compiler_error(&error, fallback_span, self.panic_threshold, ctx);
                // Report accumulated diagnostics even on pipeline failure.
                for diagnostic in hir_function.env.take_diagnostics() {
                    Self::report_compiler_error(
                        &diagnostic,
                        fallback_span,
                        self.panic_threshold,
                        ctx,
                    );
                }
                return None;
            }
        };

        // Report accumulated non-fatal diagnostics from the pipeline.
        for diagnostic in hir_function.env.take_diagnostics() {
            Self::report_compiler_error(&diagnostic, fallback_span, self.panic_threshold, ctx);
        }

        let codegen_output = match run_codegen(
            pipeline_output,
            &environment,
            ctx.ast,
            cache_identifier_name,
            None,
        ) {
            Ok(output) => output,
            Err(error) => {
                Self::report_compiler_error(&error, fallback_span, self.panic_threshold, ctx);
                return None;
            }
        };

        // Port of processFn (Program.ts lines 634-672): check for function-level
        // and module-level opt-out AFTER compilation. This allows validation/lint
        // errors to be reported even for opted-out functions.
        //
        // Function-level opt-out: if 'use no memo'/'use no forget' (or custom opt-out)
        // is present and ignoreUseNoForget is false, discard the compiled output.
        if !self.ignore_use_no_forget {
            let has_fn_opt_out = find_directive_disabling_memoization(
                directives,
                self.custom_opt_out_directives.as_deref(),
            )
            .is_some();
            if has_fn_opt_out {
                return None;
            }
        }

        // Module-level opt-out: discard compiled output (TS line 657-658).
        if self.has_module_scope_opt_out {
            return None;
        }

        // Lint mode: compile for validation but don't emit (TS line 659-660).
        if self.output_mode == CompilerOutputMode::Lint {
            return None;
        }

        Some(codegen_output)
    }

    /// Try to compile the inner function of a memo/forwardRef call expression.
    fn compile_memo_or_forwardref_arg<'a>(
        &self,
        call: &CallExpression<'a>,
        binding_name: Option<&str>,
        cache_identifier_name: &str,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<CodegenOutput<'a>> {
        let arg = call.arguments.first()?;
        let expr = arg.as_expression()?;
        match expr {
            Expression::FunctionExpression(function) => {
                let directives = function_directives(function);
                let function_name =
                    function.id.as_ref().map(|id| id.name.as_str()).or(binding_name);
                let lowerable_function = LowerableFunction::Function(function);
                self.compile_function(
                    &lowerable_function,
                    function_name,
                    &directives,
                    function.span,
                    true,
                    cache_identifier_name,
                    ctx,
                )
            }
            Expression::ArrowFunctionExpression(arrow) => {
                let directives = arrow_directives(arrow);
                let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                self.compile_function(
                    &lowerable_function,
                    binding_name,
                    &directives,
                    arrow.span,
                    true,
                    cache_identifier_name,
                    ctx,
                )
            }
            _ => None,
        }
    }

    /// Compile an outlined function that was requeued for full compilation.
    ///
    /// This is the Rust equivalent of TS Program.ts lines 448-454 where outlined
    /// functions with `type !== null` are pushed back onto the compilation queue.
    /// Unlike `compile_function`, this skips `should_compile_function` (the fn_type
    /// is already known) and suppression checks (the function is compiler-generated).
    fn compile_outlined_function<'a>(
        &self,
        statement: &Statement<'a>,
        fn_type: ReactFunctionType,
        cache_identifier_name: &str,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<CodegenOutput<'a>> {
        let Statement::FunctionDeclaration(function) = statement else {
            return None;
        };

        let environment =
            match Environment::new(fn_type, self.output_mode, self.environment_config.clone()) {
                Ok(env) => env,
                Err(error) => {
                    Self::report_compiler_error(&error, function.span, self.panic_threshold, ctx);
                    return None;
                }
            };

        let lowerable_function = LowerableFunction::Function(function);
        let mut hir_function =
            match lower(&environment, fn_type, &lowerable_function, self.outer_bindings.clone()) {
                Ok(hir_function) => hir_function,
                Err(error) => {
                    Self::report_compiler_error(&error, function.span, self.panic_threshold, ctx);
                    return None;
                }
            };

        let pipeline_output = match run_pipeline(&mut hir_function, &environment) {
            Ok(output) => output,
            Err(error) => {
                Self::report_compiler_error(&error, function.span, self.panic_threshold, ctx);
                for diagnostic in hir_function.env.take_diagnostics() {
                    Self::report_compiler_error(
                        &diagnostic,
                        function.span,
                        self.panic_threshold,
                        ctx,
                    );
                }
                return None;
            }
        };

        for diagnostic in hir_function.env.take_diagnostics() {
            Self::report_compiler_error(&diagnostic, function.span, self.panic_threshold, ctx);
        }

        let codegen_output = match run_codegen(
            pipeline_output,
            &environment,
            ctx.ast,
            cache_identifier_name,
            None,
        ) {
            Ok(output) => output,
            Err(error) => {
                Self::report_compiler_error(&error, function.span, self.panic_threshold, ctx);
                return None;
            }
        };

        // Lint mode: compile for validation but don't emit.
        if self.output_mode == CompilerOutputMode::Lint {
            return None;
        }

        Some(codegen_output)
    }

    fn report_compiler_error(
        error: &CompilerError,
        fallback_span: Span,
        panic_threshold: PanicThreshold,
        ctx: &mut TraverseCtx<'_>,
    ) {
        let action = handle_compilation_error(error, panic_threshold);
        for entry in &error.details {
            let span = compiler_error_entry_span(entry).unwrap_or(fallback_span);
            let diagnostic = match action {
                ErrorAction::Panic => OxcDiagnostic::error(format!("React Compiler: {entry}")),
                ErrorAction::Skip => OxcDiagnostic::warn(format!("React Compiler: {entry}")),
            };
            ctx.state.error(diagnostic.with_label(span));
        }
    }

    /// Recursively walk a statement to find and compile nested function declarations,
    /// function expressions, and arrow function expressions.
    ///
    /// Port of TS `findFunctionsToCompile` (Program.ts lines 495-559) which uses
    /// `program.traverse()` to visit all function nodes at any depth, skipping class
    /// bodies and already-compiled functions.
    ///
    /// Returns `true` if any nested function was compiled (for scope ID assignment).
    /// Any outlined functions produced by nested compilations are collected in
    /// `outlined_outputs` for later insertion as top-level statements.
    fn compile_nested_functions_in_statement<'a>(
        &self,
        stmt: &mut Statement<'a>,
        cache_identifier_name: &str,
        needs_memo_import: &mut bool,
        outlined_outputs: &mut Vec<OutlinedOutput<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        let mut compiled_any = false;
        self.walk_statement_for_nested_functions(
            stmt,
            cache_identifier_name,
            needs_memo_import,
            &mut compiled_any,
            outlined_outputs,
            false, // not inside class
            ctx,
        );
        compiled_any
    }

    /// Recursive walker for nested function discovery.
    ///
    /// Walks into all AST nodes looking for function declarations/expressions,
    /// skipping class bodies (matching TS `ClassDeclaration.skip()` /
    /// `ClassExpression.skip()`).
    #[expect(clippy::too_many_arguments)]
    fn walk_statement_for_nested_functions<'a>(
        &self,
        stmt: &mut Statement<'a>,
        cache_identifier_name: &str,
        needs_memo_import: &mut bool,
        compiled_any: &mut bool,
        outlined_outputs: &mut Vec<OutlinedOutput<'a>>,
        inside_class: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match stmt {
            // Function declarations at any depth: try to compile them.
            Statement::FunctionDeclaration(function) => {
                if !inside_class
                    && let Some(output) =
                        self.try_compile_nested_function_decl(function, cache_identifier_name, ctx)
                {
                    if output.memo_slots_used > 0 {
                        *needs_memo_import = true;
                    }
                    // Collect outlined functions for top-level insertion.
                    let mut output = output;
                    outlined_outputs.append(&mut output.outlined);
                    // Replace body/params in-place.
                    function.params = build_formal_params_from_codegen(&output.params, ctx);
                    function.body = Some(build_compiled_body(&mut output, ctx));
                    *compiled_any = true;
                    // Don't recurse into this function — it's been compiled
                    // (matches TS fn.skip()).
                    return;
                }
                // If not compiled, recurse into the function body to find
                // deeper nested functions.
                if let Some(body) = &mut function.body {
                    for inner_stmt in &mut body.statements {
                        self.walk_statement_for_nested_functions(
                            inner_stmt,
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            false,
                            ctx,
                        );
                    }
                }
            }
            // Variable declarations: check initializers for function expressions,
            // and also walk binding pattern defaults (e.g. `const {useHook = () => {}} = {}`).
            Statement::VariableDeclaration(declaration) => {
                for declarator in &mut declaration.declarations {
                    if let Some(init) = &mut declarator.init {
                        let binding_name = match &declarator.id {
                            BindingPattern::BindingIdentifier(id) => Some(id.name),
                            _ => None,
                        };
                        self.walk_expression_for_nested_functions(
                            init,
                            binding_name.as_deref(),
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            inside_class,
                            ctx,
                        );
                    }
                    // Walk binding pattern defaults for destructuring with default values.
                    // Port of getFunctionName context for AssignmentPattern (Program.ts:1103-1116).
                    self.walk_binding_pattern_for_nested_functions(
                        &mut declarator.id,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
            }
            // Expression statements: walk the expression.
            Statement::ExpressionStatement(expr_stmt) => {
                self.walk_expression_for_nested_functions(
                    &mut expr_stmt.expression,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            // Block statements: recurse into body.
            Statement::BlockStatement(block) => {
                for inner_stmt in &mut block.body {
                    self.walk_statement_for_nested_functions(
                        inner_stmt,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
            }
            // If statements: walk test expression and recurse into consequent and alternate.
            Statement::IfStatement(if_stmt) => {
                self.walk_expression_for_nested_functions(
                    &mut if_stmt.test,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
                self.walk_statement_for_nested_functions(
                    &mut if_stmt.consequent,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
                if let Some(alternate) = &mut if_stmt.alternate {
                    self.walk_statement_for_nested_functions(
                        alternate,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
            }
            // For/while/do-while: walk expressions and recurse into body.
            Statement::ForStatement(for_stmt) => {
                // Walk init expression (may contain function expressions).
                if let Some(init) = &mut for_stmt.init {
                    match init {
                        ForStatementInit::VariableDeclaration(var_decl) => {
                            for declarator in &mut var_decl.declarations {
                                if let Some(init_expr) = &mut declarator.init {
                                    let binding_name = match &declarator.id {
                                        BindingPattern::BindingIdentifier(id) => Some(id.name),
                                        _ => None,
                                    };
                                    self.walk_expression_for_nested_functions(
                                        init_expr,
                                        binding_name.as_deref(),
                                        cache_identifier_name,
                                        needs_memo_import,
                                        compiled_any,
                                        outlined_outputs,
                                        inside_class,
                                        ctx,
                                    );
                                }
                                self.walk_binding_pattern_for_nested_functions(
                                    &mut declarator.id,
                                    cache_identifier_name,
                                    needs_memo_import,
                                    compiled_any,
                                    outlined_outputs,
                                    inside_class,
                                    ctx,
                                );
                            }
                        }
                        init_expr => {
                            // ForStatementInit inherits Expression variants
                            if let Some(expr) = init_expr.as_expression_mut() {
                                self.walk_expression_for_nested_functions(
                                    expr,
                                    None,
                                    cache_identifier_name,
                                    needs_memo_import,
                                    compiled_any,
                                    outlined_outputs,
                                    inside_class,
                                    ctx,
                                );
                            }
                        }
                    }
                }
                if let Some(test) = &mut for_stmt.test {
                    self.walk_expression_for_nested_functions(
                        test,
                        None,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
                if let Some(update) = &mut for_stmt.update {
                    self.walk_expression_for_nested_functions(
                        update,
                        None,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
                self.walk_statement_for_nested_functions(
                    &mut for_stmt.body,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            Statement::ForInStatement(for_in) => {
                self.walk_statement_for_nested_functions(
                    &mut for_in.body,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            Statement::ForOfStatement(for_of) => {
                self.walk_statement_for_nested_functions(
                    &mut for_of.body,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            Statement::WhileStatement(while_stmt) => {
                self.walk_expression_for_nested_functions(
                    &mut while_stmt.test,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
                self.walk_statement_for_nested_functions(
                    &mut while_stmt.body,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            Statement::DoWhileStatement(do_while) => {
                self.walk_statement_for_nested_functions(
                    &mut do_while.body,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
                self.walk_expression_for_nested_functions(
                    &mut do_while.test,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            // Switch: walk discriminant and recurse into case consequents.
            Statement::SwitchStatement(switch_stmt) => {
                self.walk_expression_for_nested_functions(
                    &mut switch_stmt.discriminant,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
                for case in &mut switch_stmt.cases {
                    for inner_stmt in &mut case.consequent {
                        self.walk_statement_for_nested_functions(
                            inner_stmt,
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            inside_class,
                            ctx,
                        );
                    }
                }
            }
            // Try/catch/finally: recurse into blocks.
            Statement::TryStatement(try_stmt) => {
                for inner_stmt in &mut try_stmt.block.body {
                    self.walk_statement_for_nested_functions(
                        inner_stmt,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
                if let Some(handler) = &mut try_stmt.handler {
                    for inner_stmt in &mut handler.body.body {
                        self.walk_statement_for_nested_functions(
                            inner_stmt,
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            inside_class,
                            ctx,
                        );
                    }
                }
                if let Some(finalizer) = &mut try_stmt.finalizer {
                    for inner_stmt in &mut finalizer.body {
                        self.walk_statement_for_nested_functions(
                            inner_stmt,
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            inside_class,
                            ctx,
                        );
                    }
                }
            }
            // Labeled statements: recurse into body.
            Statement::LabeledStatement(labeled) => {
                self.walk_statement_for_nested_functions(
                    &mut labeled.body,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            // Return statement: walk the argument expression.
            Statement::ReturnStatement(ret) => {
                if let Some(arg) = &mut ret.argument {
                    self.walk_expression_for_nested_functions(
                        arg,
                        None,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
            }
            // Export declarations: recurse into inner declarations.
            Statement::ExportDefaultDeclaration(export) => match &mut export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function)
                | ExportDefaultDeclarationKind::FunctionExpression(function) => {
                    if !inside_class
                        && let Some(output) = self.try_compile_nested_function_decl(
                            function,
                            cache_identifier_name,
                            ctx,
                        )
                    {
                        if output.memo_slots_used > 0 {
                            *needs_memo_import = true;
                        }
                        let mut output = output;
                        outlined_outputs.append(&mut output.outlined);
                        function.params = build_formal_params_from_codegen(&output.params, ctx);
                        function.body = Some(build_compiled_body(&mut output, ctx));
                        *compiled_any = true;
                        return;
                    }
                    if let Some(body) = &mut function.body {
                        for inner_stmt in &mut body.statements {
                            self.walk_statement_for_nested_functions(
                                inner_stmt,
                                cache_identifier_name,
                                needs_memo_import,
                                compiled_any,
                                outlined_outputs,
                                false,
                                ctx,
                            );
                        }
                    }
                }
                ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
                    if !inside_class {
                        let directives = arrow_directives(arrow);
                        let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                        if let Some(output) = self.compile_function(
                            &lowerable_function,
                            None,
                            &directives,
                            arrow.span,
                            false,
                            cache_identifier_name,
                            ctx,
                        ) {
                            if output.memo_slots_used > 0 {
                                *needs_memo_import = true;
                            }
                            let mut output = output;
                            outlined_outputs.append(&mut output.outlined);
                            arrow.params = build_formal_params_from_codegen(&output.params, ctx);
                            let compiled_directives = build_directives(&output.directives, ctx);
                            let body = std::mem::replace(&mut output.body, ctx.ast.vec());
                            arrow.body =
                                ctx.ast.alloc_function_body(SPAN, compiled_directives, body);
                            arrow.expression = false;
                            *compiled_any = true;
                            return;
                        }
                    }
                    for inner_stmt in &mut arrow.body.statements {
                        self.walk_statement_for_nested_functions(
                            inner_stmt,
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            false,
                            ctx,
                        );
                    }
                }
                _ => {}
            },
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = &mut export.declaration {
                    self.walk_declaration_for_nested_functions(
                        declaration,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
            }
            _ => {}
        }
    }

    /// Walk a declaration node for nested function discovery.
    #[expect(clippy::too_many_arguments)]
    fn walk_declaration_for_nested_functions<'a>(
        &self,
        decl: &mut Declaration<'a>,
        cache_identifier_name: &str,
        needs_memo_import: &mut bool,
        compiled_any: &mut bool,
        outlined_outputs: &mut Vec<OutlinedOutput<'a>>,
        inside_class: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match decl {
            Declaration::FunctionDeclaration(function) => {
                if !inside_class
                    && let Some(output) =
                        self.try_compile_nested_function_decl(function, cache_identifier_name, ctx)
                {
                    if output.memo_slots_used > 0 {
                        *needs_memo_import = true;
                    }
                    let mut output = output;
                    outlined_outputs.append(&mut output.outlined);
                    function.params = build_formal_params_from_codegen(&output.params, ctx);
                    function.body = Some(build_compiled_body(&mut output, ctx));
                    *compiled_any = true;
                    return;
                }
                if let Some(body) = &mut function.body {
                    for inner_stmt in &mut body.statements {
                        self.walk_statement_for_nested_functions(
                            inner_stmt,
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            false,
                            ctx,
                        );
                    }
                }
            }
            Declaration::VariableDeclaration(var_decl) => {
                for declarator in &mut var_decl.declarations {
                    if let Some(init) = &mut declarator.init {
                        let binding_name = match &declarator.id {
                            BindingPattern::BindingIdentifier(id) => Some(id.name),
                            _ => None,
                        };
                        self.walk_expression_for_nested_functions(
                            init,
                            binding_name.as_deref(),
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            inside_class,
                            ctx,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    /// Walk an expression node for nested function discovery.
    ///
    /// Handles FunctionExpression, ArrowFunctionExpression, and recurses into
    /// other expression types that may contain functions. Skips ClassExpression
    /// bodies (matching TS `ClassExpression.skip()`).
    #[expect(clippy::too_many_arguments)]
    fn walk_expression_for_nested_functions<'a>(
        &self,
        expr: &mut Expression<'a>,
        binding_name: Option<&str>,
        cache_identifier_name: &str,
        needs_memo_import: &mut bool,
        compiled_any: &mut bool,
        outlined_outputs: &mut Vec<OutlinedOutput<'a>>,
        inside_class: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match expr {
            // Function expressions: try to compile.
            Expression::FunctionExpression(function) => {
                if !inside_class {
                    let directives = function_directives(function);
                    let function_name =
                        function.id.as_ref().map(|id| id.name.as_str()).or(binding_name);
                    let lowerable_function = LowerableFunction::Function(function);
                    if let Some(output) = self.compile_function(
                        &lowerable_function,
                        function_name,
                        &directives,
                        function.span,
                        false,
                        cache_identifier_name,
                        ctx,
                    ) {
                        if output.memo_slots_used > 0 {
                            *needs_memo_import = true;
                        }
                        let mut output = output;
                        outlined_outputs.append(&mut output.outlined);
                        function.params = build_formal_params_from_codegen(&output.params, ctx);
                        function.body = Some(build_compiled_body(&mut output, ctx));
                        *compiled_any = true;
                        return;
                    }
                }
                // Not compiled: recurse into body.
                if let Some(body) = &mut function.body {
                    for inner_stmt in &mut body.statements {
                        self.walk_statement_for_nested_functions(
                            inner_stmt,
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            false,
                            ctx,
                        );
                    }
                }
            }
            // Arrow function expressions: try to compile.
            Expression::ArrowFunctionExpression(arrow) => {
                if !inside_class {
                    let directives = arrow_directives(arrow);
                    let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                    if let Some(output) = self.compile_function(
                        &lowerable_function,
                        binding_name,
                        &directives,
                        arrow.span,
                        false,
                        cache_identifier_name,
                        ctx,
                    ) {
                        if output.memo_slots_used > 0 {
                            *needs_memo_import = true;
                        }
                        let mut output = output;
                        outlined_outputs.append(&mut output.outlined);
                        arrow.params = build_formal_params_from_codegen(&output.params, ctx);
                        let compiled_directives = build_directives(&output.directives, ctx);
                        let body = std::mem::replace(&mut output.body, ctx.ast.vec());
                        arrow.body = ctx.ast.alloc_function_body(SPAN, compiled_directives, body);
                        arrow.expression = false;
                        *compiled_any = true;
                        return;
                    }
                }
                // Not compiled: recurse into body.
                for inner_stmt in &mut arrow.body.statements {
                    self.walk_statement_for_nested_functions(
                        inner_stmt,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        false,
                        ctx,
                    );
                }
            }
            // Call expressions: check for memo/forwardRef, then recurse into args.
            Expression::CallExpression(call) => {
                if !inside_class
                    && is_memo_or_forwardref_call(&call.callee)
                    && let Some(output) = self.compile_memo_or_forwardref_arg(
                        call,
                        binding_name,
                        cache_identifier_name,
                        ctx,
                    )
                {
                    if output.memo_slots_used > 0 {
                        *needs_memo_import = true;
                    }
                    let mut output = output;
                    outlined_outputs.append(&mut output.outlined);
                    replace_memo_inner_function_body(call, &mut output, ctx);
                    *compiled_any = true;
                    return;
                }
                // Recurse into callee and arguments.
                self.walk_expression_for_nested_functions(
                    &mut call.callee,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
                for arg in &mut call.arguments {
                    if let Some(e) = arg.as_expression_mut() {
                        self.walk_expression_for_nested_functions(
                            e,
                            None,
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            inside_class,
                            ctx,
                        );
                    }
                }
            }
            // Assignment expressions: infer name from LHS identifier.
            // Port of getFunctionName context 2 (Program.ts:1189-1195).
            Expression::AssignmentExpression(assign) => {
                let assign_name = match &assign.left {
                    AssignmentTarget::AssignmentTargetIdentifier(id) => Some(id.name),
                    _ => None,
                };
                self.walk_expression_for_nested_functions(
                    &mut assign.right,
                    assign_name.as_deref(),
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            // Conditional/ternary: recurse into all branches.
            Expression::ConditionalExpression(cond) => {
                self.walk_expression_for_nested_functions(
                    &mut cond.test,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
                self.walk_expression_for_nested_functions(
                    &mut cond.consequent,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
                self.walk_expression_for_nested_functions(
                    &mut cond.alternate,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            // Logical/binary: recurse into both sides.
            Expression::LogicalExpression(logical) => {
                self.walk_expression_for_nested_functions(
                    &mut logical.left,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
                self.walk_expression_for_nested_functions(
                    &mut logical.right,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            // Sequence expressions: recurse into all.
            Expression::SequenceExpression(seq) => {
                for e in &mut seq.expressions {
                    self.walk_expression_for_nested_functions(
                        e,
                        None,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
            }
            // Array/object: recurse into elements/properties.
            Expression::ArrayExpression(arr) => {
                for elem in &mut arr.elements {
                    if let Some(e) = elem.as_expression_mut() {
                        self.walk_expression_for_nested_functions(
                            e,
                            None,
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            inside_class,
                            ctx,
                        );
                    }
                }
            }
            Expression::ObjectExpression(obj) => {
                for prop in &mut obj.properties {
                    match prop {
                        ObjectPropertyKind::ObjectProperty(p) => {
                            // Port of getFunctionName context 3 (Program.ts:1197-1211):
                            // infer name from non-computed property key identifier.
                            let prop_name = if p.computed {
                                None
                            } else {
                                match &p.key {
                                    PropertyKey::StaticIdentifier(id) => Some(id.name),
                                    _ => None,
                                }
                            };
                            self.walk_expression_for_nested_functions(
                                &mut p.value,
                                prop_name.as_deref(),
                                cache_identifier_name,
                                needs_memo_import,
                                compiled_any,
                                outlined_outputs,
                                inside_class,
                                ctx,
                            );
                        }
                        ObjectPropertyKind::SpreadProperty(s) => {
                            self.walk_expression_for_nested_functions(
                                &mut s.argument,
                                None,
                                cache_identifier_name,
                                needs_memo_import,
                                compiled_any,
                                outlined_outputs,
                                inside_class,
                                ctx,
                            );
                        }
                    }
                }
            }
            // Parenthesized: recurse.
            Expression::ParenthesizedExpression(paren) => {
                self.walk_expression_for_nested_functions(
                    &mut paren.expression,
                    binding_name,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            // New expression: recurse into callee and arguments.
            Expression::NewExpression(new_expr) => {
                self.walk_expression_for_nested_functions(
                    &mut new_expr.callee,
                    None,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
                for arg in &mut new_expr.arguments {
                    if let Some(e) = arg.as_expression_mut() {
                        self.walk_expression_for_nested_functions(
                            e,
                            None,
                            cache_identifier_name,
                            needs_memo_import,
                            compiled_any,
                            outlined_outputs,
                            inside_class,
                            ctx,
                        );
                    }
                }
            }
            // ClassExpression and others: skip (don't recurse into class bodies).
            // Matches TS `ClassExpression(node) { node.skip(); }`.
            _ => {}
        }
    }

    /// Walk a binding pattern for nested function discovery in default values.
    ///
    /// Handles `AssignmentPattern` (destructuring defaults) where the default
    /// value may be a function expression. For example:
    /// - `const {useHook = () => {}} = {}`
    /// - `const [useHook = () => {}] = []`
    ///
    /// Port of `getFunctionName` context for `AssignmentPattern` (Program.ts:1103-1116).
    #[expect(clippy::too_many_arguments)]
    fn walk_binding_pattern_for_nested_functions<'a>(
        &self,
        pattern: &mut BindingPattern<'a>,
        cache_identifier_name: &str,
        needs_memo_import: &mut bool,
        compiled_any: &mut bool,
        outlined_outputs: &mut Vec<OutlinedOutput<'a>>,
        inside_class: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match pattern {
            BindingPattern::ObjectPattern(obj) => {
                for prop in &mut obj.properties {
                    self.walk_binding_pattern_for_nested_functions(
                        &mut prop.value,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
                if let Some(rest) = &mut obj.rest {
                    self.walk_binding_pattern_for_nested_functions(
                        &mut rest.argument,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
            }
            BindingPattern::ArrayPattern(arr) => {
                for pat in (&mut arr.elements).into_iter().flatten() {
                    self.walk_binding_pattern_for_nested_functions(
                        pat,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
                if let Some(rest) = &mut arr.rest {
                    self.walk_binding_pattern_for_nested_functions(
                        &mut rest.argument,
                        cache_identifier_name,
                        needs_memo_import,
                        compiled_any,
                        outlined_outputs,
                        inside_class,
                        ctx,
                    );
                }
            }
            BindingPattern::AssignmentPattern(assign) => {
                // Infer function name from the LHS of the assignment pattern.
                let binding_name = match &assign.left {
                    BindingPattern::BindingIdentifier(id) => Some(id.name),
                    _ => None,
                };
                self.walk_expression_for_nested_functions(
                    &mut assign.right,
                    binding_name.as_deref(),
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
                // Also recurse into the LHS pattern (could be nested destructuring with defaults).
                self.walk_binding_pattern_for_nested_functions(
                    &mut assign.left,
                    cache_identifier_name,
                    needs_memo_import,
                    compiled_any,
                    outlined_outputs,
                    inside_class,
                    ctx,
                );
            }
            BindingPattern::BindingIdentifier(_) => {
                // Leaf node, nothing to recurse into.
            }
        }
    }

    /// Try to compile a nested function declaration.
    ///
    /// Helper that extracts the function's name and directives, then delegates
    /// to `compile_function`.
    fn try_compile_nested_function_decl<'a>(
        &self,
        function: &Function<'a>,
        cache_identifier_name: &str,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<CodegenOutput<'a>> {
        let directives = function_directives(function);
        let name = function.id.as_ref().map(|id| id.name.as_str());
        let lowerable_function = LowerableFunction::Function(function);
        self.compile_function(
            &lowerable_function,
            name,
            &directives,
            function.span,
            false,
            cache_identifier_name,
            ctx,
        )
    }
}

/// Determine the set of compiled function indices that are referenced before
/// their declaration site at the top level of the program.
///
/// Port of `getFunctionReferencedBeforeDeclarationAtTopLevel` from Program.ts
/// lines 1124-1183.
///
/// For each compiled function that has a name, walks program body statements
/// top-to-bottom. If an identifier reference to the function name appears
/// before the function's declaration site (at the top-level scope, not inside
/// nested functions), the function's statement index is added to the returned set.
fn get_functions_referenced_before_declaration(
    compiled_fns: &[CompileResult<'_>],
    body: &[Statement<'_>],
) -> FxHashSet<usize> {
    // Build a map: function_name -> (declaration_statement_index, compile_result_index)
    let mut fn_names: FxHashMap<&str, usize> = FxHashMap::default();
    for result in compiled_fns {
        if let Some(ref name) = result.function_name {
            fn_names.insert(name.as_str(), result.index);
        }
    }

    if fn_names.is_empty() {
        return FxHashSet::default();
    }

    let mut referenced_before_declared: FxHashSet<usize> = FxHashSet::default();

    // Walk program body statements in order, checking only top-level references.
    for (stmt_idx, stmt) in body.iter().enumerate() {
        // Collect top-level identifier references in this statement
        // (not descending into function bodies).
        let refs = collect_top_level_identifier_refs(stmt);
        for name_ref in &refs {
            if let Some(&decl_idx) = fn_names.get(name_ref.as_str()) {
                // If we haven't reached the declaration site yet, it's referenced before declared
                if stmt_idx < decl_idx {
                    referenced_before_declared.insert(decl_idx);
                }
            }
        }
        // If this statement IS the declaration of a tracked function, remove it
        // from tracking (any further references are after the declaration).
        let stmt_fn_name = get_statement_function_name(stmt);
        if let Some(name) = stmt_fn_name {
            fn_names.remove(name);
        }
    }

    referenced_before_declared
}

/// Get the function name declared by a statement (if it's a function declaration).
fn get_statement_function_name<'a>(stmt: &'a Statement<'_>) -> Option<&'a str> {
    match stmt {
        Statement::FunctionDeclaration(f) => f.id.as_ref().map(|id| id.name.as_str()),
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(f)
            | ExportDefaultDeclarationKind::FunctionExpression(f) => {
                f.id.as_ref().map(|id| id.name.as_str())
            }
            _ => None,
        },
        Statement::ExportNamedDeclaration(export) => match &export.declaration {
            Some(Declaration::FunctionDeclaration(f)) => f.id.as_ref().map(|id| id.name.as_str()),
            _ => None,
        },
        _ => None,
    }
}

/// Collect top-level identifier references in a statement.
///
/// Only collects references at the top level scope -- does not descend into
/// function bodies, arrow functions, or class bodies. This matches the TS
/// behavior where `scope.getFunctionParent() === null` filters to top-level.
fn collect_top_level_identifier_refs(stmt: &Statement<'_>) -> Vec<String> {
    let mut refs = Vec::new();
    collect_top_level_refs_from_stmt(stmt, &mut refs);
    refs
}

fn collect_top_level_refs_from_stmt(stmt: &Statement<'_>, refs: &mut Vec<String>) {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            collect_top_level_refs_from_expr(&expr_stmt.expression, refs);
        }
        Statement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                if let Some(init) = &declarator.init {
                    collect_top_level_refs_from_expr(init, refs);
                }
            }
        }
        Statement::ReturnStatement(ret) => {
            if let Some(arg) = &ret.argument {
                collect_top_level_refs_from_expr(arg, refs);
            }
        }
        Statement::ExportDefaultDeclaration(export) => {
            if let ExportDefaultDeclarationKind::CallExpression(call) = &export.declaration {
                // Walk the call expression's callee and arguments for identifier refs.
                collect_top_level_refs_from_expr(&call.callee, refs);
                for arg in &call.arguments {
                    if let Some(e) = arg.as_expression() {
                        collect_top_level_refs_from_expr(e, refs);
                    }
                }
            }
            // Don't descend into function/arrow bodies
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(Declaration::VariableDeclaration(var_decl)) = &export.declaration {
                for declarator in &var_decl.declarations {
                    if let Some(init) = &declarator.init {
                        collect_top_level_refs_from_expr(init, refs);
                    }
                }
            }
            // Don't descend into function declarations
        }
        Statement::IfStatement(if_stmt) => {
            collect_top_level_refs_from_expr(&if_stmt.test, refs);
            // Don't descend into branches -- they're block statements at top level
        }
        _ => {}
    }
}

fn collect_top_level_refs_from_expr(expr: &Expression<'_>, refs: &mut Vec<String>) {
    match expr {
        Expression::Identifier(id) => {
            refs.push(id.name.to_string());
        }
        Expression::CallExpression(call) => {
            collect_top_level_refs_from_expr(&call.callee, refs);
            for arg in &call.arguments {
                if let Some(e) = arg.as_expression() {
                    collect_top_level_refs_from_expr(e, refs);
                }
            }
        }
        Expression::StaticMemberExpression(member) => {
            collect_top_level_refs_from_expr(&member.object, refs);
        }
        Expression::ComputedMemberExpression(member) => {
            collect_top_level_refs_from_expr(&member.object, refs);
            collect_top_level_refs_from_expr(&member.expression, refs);
        }
        Expression::ConditionalExpression(cond) => {
            collect_top_level_refs_from_expr(&cond.test, refs);
            collect_top_level_refs_from_expr(&cond.consequent, refs);
            collect_top_level_refs_from_expr(&cond.alternate, refs);
        }
        Expression::AssignmentExpression(assign) => {
            collect_top_level_refs_from_expr(&assign.right, refs);
        }
        Expression::LogicalExpression(logical) => {
            collect_top_level_refs_from_expr(&logical.left, refs);
            collect_top_level_refs_from_expr(&logical.right, refs);
        }
        Expression::BinaryExpression(binary) => {
            collect_top_level_refs_from_expr(&binary.left, refs);
            collect_top_level_refs_from_expr(&binary.right, refs);
        }
        Expression::SequenceExpression(seq) => {
            for e in &seq.expressions {
                collect_top_level_refs_from_expr(e, refs);
            }
        }
        Expression::ArrayExpression(arr) => {
            for elem in &arr.elements {
                if let Some(e) = elem.as_expression() {
                    collect_top_level_refs_from_expr(e, refs);
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                match prop {
                    ObjectPropertyKind::ObjectProperty(p) => {
                        collect_top_level_refs_from_expr(&p.value, refs);
                    }
                    ObjectPropertyKind::SpreadProperty(s) => {
                        collect_top_level_refs_from_expr(&s.argument, refs);
                    }
                }
            }
        }
        Expression::NewExpression(new_expr) => {
            collect_top_level_refs_from_expr(&new_expr.callee, refs);
            for arg in &new_expr.arguments {
                if let Some(e) = arg.as_expression() {
                    collect_top_level_refs_from_expr(e, refs);
                }
            }
        }
        Expression::UnaryExpression(unary) => {
            collect_top_level_refs_from_expr(&unary.argument, refs);
        }
        Expression::TemplateLiteral(tmpl) => {
            for e in &tmpl.expressions {
                collect_top_level_refs_from_expr(e, refs);
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            collect_top_level_refs_from_expr(&tagged.tag, refs);
        }
        Expression::ParenthesizedExpression(paren) => {
            collect_top_level_refs_from_expr(&paren.expression, refs);
        }
        // Don't descend into function expressions, arrow functions, or class expressions.
        // All other expression types are also ignored.
        _ => {}
    }
}

/// Extract dynamic gating from a function's directives.
///
/// Port of `findDirectivesDynamicGating` from Program.ts lines 72-129.
///
/// Looks for `use memo if(IDENT)` directives. If `dynamic_gating` config is set
/// and the directive is found, returns an `ExternalFunction` with the dynamic
/// gating source and the directive's identifier as the import specifier.
fn extract_dynamic_gating_directive(
    directives: &[String],
    dynamic_gating: Option<&DynamicGatingOptions>,
) -> Result<Option<ExternalFunction>, Vec<String>> {
    let Some(dynamic_gating_config) = dynamic_gating else {
        return Ok(None);
    };

    let mut errors: Vec<String> = Vec::new();
    // Store (ident, full_directive) pairs for error reporting.
    let mut matches: Vec<(String, String)> = Vec::new();

    for directive in directives {
        match parse_dynamic_gating_directive(directive) {
            Some(Ok(ident)) => {
                matches.push((ident.to_string(), directive.clone()));
            }
            Some(Err(raw)) => {
                errors.push(format!(
                    "Dynamic gating directive is not a valid JavaScript identifier. Found '{raw}'"
                ));
            }
            None => {}
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    match matches.len() {
        1 => {
            let (ident, _) = matches.into_iter().next().unwrap_or_default();
            Ok(Some(ExternalFunction {
                source: dynamic_gating_config.source.clone(),
                import_specifier_name: ident,
            }))
        }
        n if n > 1 => {
            let directive_list =
                matches.iter().map(|(_, d)| d.as_str()).collect::<Vec<_>>().join(", ");
            Err(vec![format!(
                "Multiple dynamic gating directives found. Expected a single directive but found [{directive_list}]"
            )])
        }
        _ => Ok(None),
    }
}

/// Determine the gating function kind from the statement and its function.
fn get_gating_function_kind(stmt: &Statement<'_>) -> GatingFunctionKind {
    match stmt {
        Statement::FunctionDeclaration(_) => GatingFunctionKind::FunctionDeclaration,
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(_) => {
                GatingFunctionKind::FunctionDeclaration
            }
            ExportDefaultDeclarationKind::ArrowFunctionExpression(_) => {
                GatingFunctionKind::ArrowFunction
            }
            ExportDefaultDeclarationKind::FunctionExpression(_) => {
                GatingFunctionKind::FunctionExpression
            }
            _ => GatingFunctionKind::FunctionExpression,
        },
        Statement::ExportNamedDeclaration(export) => match &export.declaration {
            Some(Declaration::FunctionDeclaration(_)) => GatingFunctionKind::FunctionDeclaration,
            _ => GatingFunctionKind::FunctionExpression,
        },
        Statement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                if let Some(Expression::ArrowFunctionExpression(_)) = &declarator.init {
                    return GatingFunctionKind::ArrowFunction;
                }
                if declarator.init.is_some() {
                    return GatingFunctionKind::FunctionExpression;
                }
            }
            GatingFunctionKind::FunctionExpression
        }
        _ => GatingFunctionKind::FunctionExpression,
    }
}

/// Determine the parent context for gating (export default vs other).
fn get_gating_parent_context(stmt: &Statement<'_>) -> ParentContext {
    match stmt {
        Statement::ExportDefaultDeclaration(_) => ParentContext::ExportDefault,
        _ => ParentContext::Other,
    }
}

/// Get the init expression from a specific declarator within a statement.
fn get_declarator_init<'a, 'b>(
    stmt: &'b Statement<'a>,
    declarator_index: usize,
) -> Option<&'b Expression<'a>> {
    match stmt {
        Statement::VariableDeclaration(decl) => {
            decl.declarations.get(declarator_index).and_then(|d| d.init.as_ref())
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(Declaration::VariableDeclaration(decl)) = &export.declaration {
                decl.declarations.get(declarator_index).and_then(|d| d.init.as_ref())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Determine the gating function kind for a specific declarator within a VarDecl.
fn get_declarator_function_kind(
    stmt: &Statement<'_>,
    declarator_index: usize,
) -> GatingFunctionKind {
    match get_declarator_init(stmt, declarator_index) {
        Some(Expression::ArrowFunctionExpression(_)) => GatingFunctionKind::ArrowFunction,
        _ => GatingFunctionKind::FunctionExpression,
    }
}

/// Get the directives from a specific declarator's function body.
fn get_declarator_directives(stmt: &Statement<'_>, declarator_index: usize) -> Vec<String> {
    match get_declarator_init(stmt, declarator_index) {
        Some(Expression::FunctionExpression(f)) => function_directives(f),
        Some(Expression::ArrowFunctionExpression(arrow)) => arrow_directives(arrow),
        _ => Vec::new(),
    }
}

/// Get the parameter info from a specific declarator's function.
fn get_declarator_param_info(stmt: &Statement<'_>, declarator_index: usize) -> Vec<ParamInfo> {
    match get_declarator_init(stmt, declarator_index) {
        Some(Expression::FunctionExpression(f)) => get_param_info_from_formal(&f.params),
        Some(Expression::ArrowFunctionExpression(arrow)) => {
            get_param_info_from_formal(&arrow.params)
        }
        _ => Vec::new(),
    }
}

/// Apply ternary gating in-place for a specific VarDecl declarator.
///
/// Replaces just the declarator's init expression with
/// `gatingFn() ? compiledExpr : originalExpr`, preserving the enclosing
/// statement and all other declarators. This matches upstream behavior where
/// `fnPath.replaceWith(gatingExpression)` replaces only the function expression.
fn apply_ternary_gating_in_place<'a>(
    stmt: &mut Statement<'a>,
    mut compiled: CodegenOutput<'a>,
    gating_fn_name: &str,
    declarator_index: usize,
    ctx: &TraverseCtx<'a>,
) {
    let compiled_expr = build_compiled_function_expr(&mut compiled, ctx);

    let declarations = match stmt {
        Statement::VariableDeclaration(decl) => &mut decl.declarations,
        Statement::ExportNamedDeclaration(export) => {
            if let Some(Declaration::VariableDeclaration(decl)) = &mut export.declaration {
                &mut decl.declarations
            } else {
                return;
            }
        }
        _ => return,
    };

    if let Some(declarator) = declarations.get_mut(declarator_index)
        && let Some(init) = &mut declarator.init
    {
        let original_expr = std::mem::replace(init, ctx.ast.expression_null_literal(SPAN));
        let ternary = build_gating_ternary(gating_fn_name, compiled_expr, original_expr, ctx);
        declarator.init = Some(ternary);
    }
}

/// Get the directives from a statement's function body, for dynamic gating.
fn get_statement_directives(stmt: &Statement<'_>) -> Vec<String> {
    match stmt {
        Statement::FunctionDeclaration(f) => function_directives(f),
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(f)
            | ExportDefaultDeclarationKind::FunctionExpression(f) => function_directives(f),
            ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => arrow_directives(arrow),
            _ => Vec::new(),
        },
        Statement::ExportNamedDeclaration(export) => match &export.declaration {
            Some(Declaration::FunctionDeclaration(f)) => function_directives(f),
            _ => Vec::new(),
        },
        Statement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                if let Some(init) = &declarator.init {
                    return match init {
                        Expression::FunctionExpression(f) => function_directives(f),
                        Expression::ArrowFunctionExpression(arrow) => arrow_directives(arrow),
                        _ => Vec::new(),
                    };
                }
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}

/// Get the parameter info (count and rest-ness) from a statement's function.
fn get_statement_param_info(stmt: &Statement<'_>) -> Vec<ParamInfo> {
    let params = match stmt {
        Statement::FunctionDeclaration(f) => Some(&f.params),
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(f)
            | ExportDefaultDeclarationKind::FunctionExpression(f) => Some(&f.params),
            ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => Some(&arrow.params),
            _ => None,
        },
        Statement::ExportNamedDeclaration(export) => match &export.declaration {
            Some(Declaration::FunctionDeclaration(f)) => Some(&f.params),
            _ => None,
        },
        Statement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                if let Some(init) = &declarator.init {
                    return match init {
                        Expression::FunctionExpression(f) => get_param_info_from_formal(&f.params),
                        Expression::ArrowFunctionExpression(arrow) => {
                            get_param_info_from_formal(&arrow.params)
                        }
                        _ => Vec::new(),
                    };
                }
            }
            return Vec::new();
        }
        _ => None,
    };

    params.map_or_else(Vec::new, |p| get_param_info_from_formal(p))
}

fn get_param_info_from_formal(params: &FormalParameters<'_>) -> Vec<ParamInfo> {
    let mut info: Vec<ParamInfo> =
        params.items.iter().map(|_| ParamInfo { is_rest: false }).collect();
    if params.rest.is_some() {
        info.push(ParamInfo { is_rest: true });
    }
    info
}

/// Build a function expression from a function declaration (for ternary gating).
///
/// Converts a FunctionDeclaration to a FunctionExpression AST node,
/// preserving params, body, async, generator, and id.
fn convert_function_decl_to_expr<'a>(
    function: &mut Function<'a>,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let params = std::mem::replace(
        &mut function.params,
        ctx.ast.alloc_formal_parameters(
            SPAN,
            FormalParameterKind::FormalParameter,
            ctx.ast.vec(),
            NONE,
        ),
    );
    let body = function.body.take();
    let id = function.id.take();
    let func = ctx.ast.alloc_function(
        SPAN,
        FunctionType::FunctionExpression,
        id,
        function.generator,
        function.r#async,
        false,
        NONE,
        NONE,
        params,
        NONE,
        body,
    );
    Expression::FunctionExpression(func)
}

/// Build a gating ternary expression: `gatingFn() ? compiled : original`
fn build_gating_ternary<'a>(
    gating_fn_name: &str,
    compiled_expr: Expression<'a>,
    original_expr: Expression<'a>,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let callee = ctx.ast.expression_identifier(SPAN, ctx.ast.atom(gating_fn_name));
    let test = ctx.ast.expression_call(SPAN, callee, NONE, ctx.ast.vec(), false);
    ctx.ast.expression_conditional(SPAN, test, compiled_expr, original_expr)
}

/// Build the compiled function as a function expression for gating.
///
/// Takes the CodegenOutput and builds a FunctionExpression from it.
fn build_compiled_function_expr<'a>(
    compiled: &mut CodegenOutput<'a>,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let directives = build_directives(&compiled.directives, ctx);
    let body = std::mem::replace(&mut compiled.body, ctx.ast.vec());
    let function_body = ctx.ast.alloc_function_body(SPAN, directives, body);
    let params = build_formal_params_from_codegen(&compiled.params, ctx);

    let id =
        compiled.id.as_deref().map(|name| ctx.ast.binding_identifier(SPAN, ctx.ast.atom(name)));

    let func = ctx.ast.alloc_function(
        SPAN,
        FunctionType::FunctionExpression,
        id,
        compiled.generator,
        compiled.is_async,
        false,
        NONE,
        NONE,
        params,
        NONE,
        Some(function_body),
    );
    Expression::FunctionExpression(func)
}

/// Build a gating-aware replacement for a statement using ternary (non-hoisted) mode.
///
/// Returns the replacement statement(s) to insert into the program body.
fn build_ternary_gating_replacement<'a>(
    mut stmt: Statement<'a>,
    mut compiled: CodegenOutput<'a>,
    gating_fn_name: &str,
    wrap: &TernaryWrap,
    ctx: &TraverseCtx<'a>,
) -> Vec<Statement<'a>> {
    let compiled_expr = build_compiled_function_expr(&mut compiled, ctx);

    // Build the original function as a function expression.
    let original_expr = extract_function_as_expr(&mut stmt, ctx);

    let ternary = build_gating_ternary(gating_fn_name, compiled_expr, original_expr, ctx);

    match wrap {
        TernaryWrap::ConstDeclaration { name } => {
            // `const Name = gatingFn() ? compiled : original;`
            let binding_id =
                ctx.ast.binding_pattern_binding_identifier(SPAN, ctx.ast.atom(name.as_str()));
            let declarator = ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Const,
                binding_id,
                NONE,
                Some(ternary),
                false,
            );
            let mut declarators = ctx.ast.vec_with_capacity(1);
            declarators.push(declarator);
            let var_decl = ctx.ast.alloc_variable_declaration(
                SPAN,
                VariableDeclarationKind::Const,
                declarators,
                false,
            );
            vec![Statement::VariableDeclaration(var_decl)]
        }
        TernaryWrap::ExportDefaultThenConst { name } => {
            // `const Name = gatingFn() ? compiled : original;`
            let binding_id =
                ctx.ast.binding_pattern_binding_identifier(SPAN, ctx.ast.atom(name.as_str()));
            let declarator = ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Const,
                binding_id,
                NONE,
                Some(ternary),
                false,
            );
            let mut declarators = ctx.ast.vec_with_capacity(1);
            declarators.push(declarator);
            let var_decl = ctx.ast.alloc_variable_declaration(
                SPAN,
                VariableDeclarationKind::Const,
                declarators,
                false,
            );
            let const_stmt = Statement::VariableDeclaration(var_decl);
            // `export default Name;`
            let default_expr = ctx.ast.expression_identifier(SPAN, ctx.ast.atom(name.as_str()));
            let export_decl = ctx.ast.alloc_export_default_declaration(SPAN, default_expr.into());
            let export_stmt = Statement::ExportDefaultDeclaration(export_decl);
            vec![const_stmt, export_stmt]
        }
        TernaryWrap::Inline => {
            // Replace the function in the statement with the ternary expression.
            // This is used for export default arrow functions, anonymous export defaults, etc.
            replace_function_in_statement_with_expr(stmt, ternary, ctx)
        }
    }
}

/// Extract the function from a statement as a function expression.
///
/// For function declarations, converts them to function expressions.
/// For arrow/function expressions in variable declarations, extracts the expression.
fn extract_function_as_expr<'a>(stmt: &mut Statement<'a>, ctx: &TraverseCtx<'a>) -> Expression<'a> {
    match stmt {
        Statement::FunctionDeclaration(function) => convert_function_decl_to_expr(function, ctx),
        Statement::VariableDeclaration(decl) => {
            for declarator in &mut decl.declarations {
                if let Some(init) = &mut declarator.init {
                    let dummy = ctx.ast.expression_null_literal(SPAN);
                    return std::mem::replace(init, dummy);
                }
            }
            ctx.ast.expression_null_literal(SPAN)
        }
        Statement::ExportDefaultDeclaration(export) => match &mut export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(function)
            | ExportDefaultDeclarationKind::FunctionExpression(function) => {
                convert_function_decl_to_expr(function, ctx)
            }
            ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
                let params = std::mem::replace(
                    &mut arrow.params,
                    ctx.ast.alloc_formal_parameters(
                        SPAN,
                        FormalParameterKind::FormalParameter,
                        ctx.ast.vec(),
                        NONE,
                    ),
                );
                let body = std::mem::replace(
                    &mut arrow.body,
                    ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), ctx.ast.vec()),
                );
                let arrow_expr = ctx.ast.alloc_arrow_function_expression(
                    SPAN,
                    arrow.expression,
                    arrow.r#async,
                    NONE,
                    params,
                    NONE,
                    body,
                );
                Expression::ArrowFunctionExpression(arrow_expr)
            }
            _ => ctx.ast.expression_null_literal(SPAN),
        },
        Statement::ExportNamedDeclaration(export) => {
            if let Some(decl) = &mut export.declaration {
                match decl {
                    Declaration::FunctionDeclaration(function) => {
                        convert_function_decl_to_expr(function, ctx)
                    }
                    Declaration::VariableDeclaration(var_decl) => {
                        for declarator in &mut var_decl.declarations {
                            if let Some(init) = &mut declarator.init {
                                let dummy = ctx.ast.expression_null_literal(SPAN);
                                return std::mem::replace(init, dummy);
                            }
                        }
                        ctx.ast.expression_null_literal(SPAN)
                    }
                    _ => ctx.ast.expression_null_literal(SPAN),
                }
            } else {
                ctx.ast.expression_null_literal(SPAN)
            }
        }
        _ => ctx.ast.expression_null_literal(SPAN),
    }
}

/// Replace the function in a statement with a raw expression (for Inline ternary mode).
fn replace_function_in_statement_with_expr<'a>(
    mut stmt: Statement<'a>,
    expr: Expression<'a>,
    ctx: &TraverseCtx<'a>,
) -> Vec<Statement<'a>> {
    match &mut stmt {
        Statement::ExportDefaultDeclaration(export) => {
            export.declaration = ExportDefaultDeclarationKind::from(expr);
            vec![stmt]
        }
        Statement::FunctionDeclaration(function) => {
            // `function Name() {}` → `const Name = <expr>;`
            if let Some(id) = &function.id {
                let name = id.name.as_str();
                let binding = ctx.ast.binding_pattern_binding_identifier(SPAN, ctx.ast.atom(name));
                let declarator = ctx.ast.variable_declarator(
                    SPAN,
                    VariableDeclarationKind::Const,
                    binding,
                    NONE,
                    Some(expr),
                    false,
                );
                let mut declarators = ctx.ast.vec_with_capacity(1);
                declarators.push(declarator);
                let var_decl = ctx.ast.alloc_variable_declaration(
                    SPAN,
                    VariableDeclarationKind::Const,
                    declarators,
                    false,
                );
                vec![Statement::VariableDeclaration(var_decl)]
            } else {
                // Anonymous function declaration — wrap as expression statement
                let expr_stmt = ctx.ast.alloc_expression_statement(SPAN, expr);
                vec![Statement::ExpressionStatement(expr_stmt)]
            }
        }
        Statement::VariableDeclaration(var_decl) => {
            // Replace the first function-like initializer with the ternary expression.
            for declarator in &mut var_decl.declarations {
                if let Some(init) = &declarator.init
                    && matches!(
                        init,
                        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
                    )
                {
                    declarator.init = Some(expr);
                    return vec![stmt];
                }
            }
            vec![stmt]
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(declaration) = &mut export.declaration {
                match declaration {
                    Declaration::FunctionDeclaration(function) => {
                        // `export function Name() {}` → `export const Name = <expr>;`
                        if let Some(id) = &function.id {
                            let name = id.name.as_str();
                            let binding = ctx
                                .ast
                                .binding_pattern_binding_identifier(SPAN, ctx.ast.atom(name));
                            let declarator = ctx.ast.variable_declarator(
                                SPAN,
                                VariableDeclarationKind::Const,
                                binding,
                                NONE,
                                Some(expr),
                                false,
                            );
                            let mut declarators = ctx.ast.vec_with_capacity(1);
                            declarators.push(declarator);
                            let var_decl = ctx.ast.alloc_variable_declaration(
                                SPAN,
                                VariableDeclarationKind::Const,
                                declarators,
                                false,
                            );
                            export.declaration = Some(Declaration::VariableDeclaration(var_decl));
                        }
                    }
                    Declaration::VariableDeclaration(var_decl) => {
                        // Replace the first function-like initializer with the ternary.
                        for declarator in &mut var_decl.declarations {
                            if let Some(init) = &declarator.init
                                && matches!(
                                    init,
                                    Expression::FunctionExpression(_)
                                        | Expression::ArrowFunctionExpression(_)
                                )
                            {
                                declarator.init = Some(expr);
                                return vec![stmt];
                            }
                        }
                    }
                    _ => {}
                }
            }
            vec![stmt]
        }
        _ => {
            // Fallback: wrap as expression statement
            let expr_stmt = ctx.ast.alloc_expression_statement(SPAN, expr);
            vec![Statement::ExpressionStatement(expr_stmt)]
        }
    }
}

/// Build a hoisted gating replacement for a function declaration.
///
/// Returns a Vec of statements:
/// 1. `const gating_result = gatingFn();`
/// 2. `function Name_optimized(...) { /* compiled */ }`
/// 3. The original statement (renamed to Name_unoptimized)
/// 4. `function Name(arg0, ...) { if (gating_result) return Name_optimized(arg0, ...); else return Name_unoptimized(arg0, ...); }`
fn build_hoisted_gating_replacement<'a>(
    mut stmt: Statement<'a>,
    mut compiled: CodegenOutput<'a>,
    gating_output: &GatingOutput,
    ctx: &TraverseCtx<'a>,
) -> Vec<Statement<'a>> {
    let GatingOutput::Hoisted {
        original_name,
        optimized_name,
        unoptimized_name,
        gating_result_name,
        gating_fn_name,
        params,
    } = gating_output
    else {
        unreachable!("build_hoisted_gating_replacement called with non-Hoisted gating output");
    };

    let mut result = Vec::with_capacity(4);

    // 1. `const gating_result = gatingFn();`
    let gating_callee = ctx.ast.expression_identifier(SPAN, ctx.ast.atom(gating_fn_name.as_str()));
    let gating_call = ctx.ast.expression_call(SPAN, gating_callee, NONE, ctx.ast.vec(), false);
    let gating_binding =
        ctx.ast.binding_pattern_binding_identifier(SPAN, ctx.ast.atom(gating_result_name.as_str()));
    let gating_declarator = ctx.ast.variable_declarator(
        SPAN,
        VariableDeclarationKind::Const,
        gating_binding,
        NONE,
        Some(gating_call),
        false,
    );
    let mut gating_declarators = ctx.ast.vec_with_capacity(1);
    gating_declarators.push(gating_declarator);
    let gating_var_decl = ctx.ast.alloc_variable_declaration(
        SPAN,
        VariableDeclarationKind::Const,
        gating_declarators,
        false,
    );
    result.push(Statement::VariableDeclaration(gating_var_decl));

    // 2. Build compiled function as `function Name_optimized(...) { ... }`
    let compiled_id = Some(ctx.ast.binding_identifier(SPAN, ctx.ast.atom(optimized_name.as_str())));
    let compiled_params = build_formal_params_from_codegen(&compiled.params, ctx);
    let compiled_directives = build_directives(&compiled.directives, ctx);
    let compiled_body_stmts = std::mem::replace(&mut compiled.body, ctx.ast.vec());
    let compiled_body = ctx.ast.alloc_function_body(SPAN, compiled_directives, compiled_body_stmts);
    let compiled_fn = ctx.ast.alloc_function(
        SPAN,
        FunctionType::FunctionDeclaration,
        compiled_id,
        compiled.generator,
        compiled.is_async,
        false,
        NONE,
        NONE,
        compiled_params,
        NONE,
        Some(compiled_body),
    );
    result.push(Statement::FunctionDeclaration(compiled_fn));

    // 3. Rename the original function to Name_unoptimized.
    rename_function_in_statement(&mut stmt, unoptimized_name, ctx);
    result.push(stmt);

    // 4. Build dispatcher: `function Name(arg0, ...) { if (gating_result) ... }`
    let mut dispatcher_params_items = ctx.ast.vec_with_capacity(params.len());
    let mut dispatcher_rest: Option<ABox<'a, FormalParameterRest<'a>>> = None;

    for (i, p) in params.iter().enumerate() {
        let arg_name = format!("arg{i}");
        if p.is_rest {
            let binding_pattern =
                ctx.ast.binding_pattern_binding_identifier(SPAN, ctx.ast.atom(arg_name.as_str()));
            let binding_rest = ctx.ast.binding_rest_element(SPAN, binding_pattern);
            dispatcher_rest =
                Some(ctx.ast.alloc_formal_parameter_rest(SPAN, ctx.ast.vec(), binding_rest, NONE));
        } else {
            let pattern =
                ctx.ast.binding_pattern_binding_identifier(SPAN, ctx.ast.atom(arg_name.as_str()));
            dispatcher_params_items.push(ctx.ast.formal_parameter(
                SPAN,
                ctx.ast.vec(),
                pattern,
                NONE,
                NONE,
                false,
                None,
                false,
                false,
            ));
        }
    }
    let dispatcher_params = ctx.ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        dispatcher_params_items,
        dispatcher_rest,
    );

    // Build args for calling optimized/unoptimized
    let build_call_args = |ctx: &TraverseCtx<'a>| -> AVec<'a, Argument<'a>> {
        let mut args = ctx.ast.vec_with_capacity(params.len());
        for (i, p) in params.iter().enumerate() {
            let arg_name = format!("arg{i}");
            let id = ctx.ast.expression_identifier(SPAN, ctx.ast.atom(arg_name.as_str()));
            if p.is_rest {
                args.push(Argument::SpreadElement(ctx.ast.alloc_spread_element(SPAN, id)));
            } else {
                args.push(Argument::from(id));
            }
        }
        args
    };

    // if (gating_result) return Name_optimized(args); else return Name_unoptimized(args);
    let gating_test =
        ctx.ast.expression_identifier(SPAN, ctx.ast.atom(gating_result_name.as_str()));

    let opt_callee = ctx.ast.expression_identifier(SPAN, ctx.ast.atom(optimized_name.as_str()));
    let opt_call = ctx.ast.expression_call(SPAN, opt_callee, NONE, build_call_args(ctx), false);
    let opt_return = ctx.ast.statement_return(SPAN, Some(opt_call));

    let unopt_callee = ctx.ast.expression_identifier(SPAN, ctx.ast.atom(unoptimized_name.as_str()));
    let unopt_call = ctx.ast.expression_call(SPAN, unopt_callee, NONE, build_call_args(ctx), false);
    let unopt_return = ctx.ast.statement_return(SPAN, Some(unopt_call));

    let if_stmt = ctx.ast.statement_if(SPAN, gating_test, opt_return, Some(unopt_return));
    let mut dispatcher_body_stmts = ctx.ast.vec_with_capacity(1);
    dispatcher_body_stmts.push(if_stmt);
    let dispatcher_body = ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), dispatcher_body_stmts);

    let dispatcher_id =
        Some(ctx.ast.binding_identifier(SPAN, ctx.ast.atom(original_name.as_str())));
    let dispatcher_fn = ctx.ast.alloc_function(
        SPAN,
        FunctionType::FunctionDeclaration,
        dispatcher_id,
        false, // not generator
        false, // not async
        false,
        NONE,
        NONE,
        dispatcher_params,
        NONE,
        Some(dispatcher_body),
    );
    result.push(Statement::FunctionDeclaration(dispatcher_fn));

    result
}

/// Rename the function inside a statement (for hoisted gating).
fn rename_function_in_statement<'a>(
    stmt: &mut Statement<'a>,
    new_name: &str,
    ctx: &TraverseCtx<'a>,
) {
    match stmt {
        Statement::FunctionDeclaration(f) => {
            f.id = Some(ctx.ast.binding_identifier(SPAN, ctx.ast.atom(new_name)));
        }
        Statement::ExportDefaultDeclaration(export) => match &mut export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(f)
            | ExportDefaultDeclarationKind::FunctionExpression(f) => {
                f.id = Some(ctx.ast.binding_identifier(SPAN, ctx.ast.atom(new_name)));
            }
            _ => {}
        },
        Statement::ExportNamedDeclaration(export) => {
            if let Some(Declaration::FunctionDeclaration(f)) = &mut export.declaration {
                f.id = Some(ctx.ast.binding_identifier(SPAN, ctx.ast.atom(new_name)));
            }
        }
        _ => {}
    }
}

/// Check if a call expression's callee is `memo`/`React.memo`/`forwardRef`/`React.forwardRef`.
fn is_memo_or_forwardref_call(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(id) => id.name == "memo" || id.name == "forwardRef",
        Expression::StaticMemberExpression(member) => {
            matches!(&member.object, Expression::Identifier(obj) if obj.name == "React")
                && (member.property.name == "memo" || member.property.name == "forwardRef")
        }
        _ => false,
    }
}

fn parse_compilation_mode(mode: Option<&str>) -> CompilationMode {
    match mode {
        Some("all") => CompilationMode::All,
        Some("annotation") => CompilationMode::Annotation,
        Some("syntax") => CompilationMode::Syntax,
        Some("infer") | None => CompilationMode::Infer,
        Some(invalid) => {
            panic!(
                "React Compiler: Invalid compilationMode \"{invalid}\". \
                 Expected \"infer\", \"annotation\", \"syntax\", or \"all\"."
            );
        }
    }
}

fn parse_output_mode(mode: Option<&str>) -> Option<CompilerOutputMode> {
    match mode {
        Some("client") => Some(CompilerOutputMode::Client),
        Some("ssr") => Some(CompilerOutputMode::Ssr),
        Some("lint") => Some(CompilerOutputMode::Lint),
        None => None,
        Some(invalid) => {
            panic!(
                "React Compiler: Invalid outputMode \"{invalid}\". \
                 Expected \"client\", \"ssr\", or \"lint\"."
            );
        }
    }
}

fn parse_panic_threshold(threshold: Option<&str>) -> PanicThreshold {
    match threshold {
        Some("all_errors") => PanicThreshold::AllErrors,
        Some("critical_errors") => PanicThreshold::CriticalErrors,
        Some("none") | None => PanicThreshold::None,
        Some(invalid) => {
            panic!(
                "React Compiler: Invalid panicThreshold \"{invalid}\". \
                 Expected \"all_errors\", \"critical_errors\", or \"none\"."
            );
        }
    }
}

/// Parse a target string into a `CompilerReactTarget`.
///
/// Maps string values to targets:
/// - "react-17" / "17" -> React17
/// - "react-18" / "18" -> React18
/// - "react-19" / "19" (default) -> React19
///
/// Panics on invalid values, matching upstream `parseTargetConfig` which calls
/// `CompilerError.throwInvalidConfig` (Options.ts:405-417).
fn parse_target(target: Option<&str>) -> CompilerReactTarget {
    match target {
        Some("react-17" | "17") => CompilerReactTarget::React17,
        Some("react-18" | "18") => CompilerReactTarget::React18,
        Some("react-19" | "19") | None => CompilerReactTarget::React19,
        Some(invalid) => {
            panic!(
                "React Compiler: Invalid target \"{invalid}\". \
                 Expected \"react-17\", \"react-18\", or \"react-19\"."
            );
        }
    }
}

fn function_directives(function: &Function<'_>) -> Vec<String> {
    function.body.as_ref().map_or_else(Vec::new, |body| {
        body.directives.iter().map(|directive| directive.directive.to_string()).collect()
    })
}

fn arrow_directives(function: &ArrowFunctionExpression<'_>) -> Vec<String> {
    function.body.directives.iter().map(|directive| directive.directive.to_string()).collect()
}

fn compiler_error_entry_span(entry: &CompilerErrorEntry) -> Option<Span> {
    let location = match entry {
        CompilerErrorEntry::Diagnostic(diagnostic) => diagnostic.primary_location(),
        CompilerErrorEntry::Detail(detail) => detail.primary_location(),
    };
    match location {
        Some(SourceLocation::Source(span)) => Some(span),
        _ => None,
    }
}

/// Build a `Vec<Directive>` from string directive names.
fn build_directives<'a>(
    directive_strings: &[String],
    ctx: &TraverseCtx<'a>,
) -> AVec<'a, Directive<'a>> {
    let mut directives = ctx.ast.vec_with_capacity(directive_strings.len());
    for d in directive_strings {
        let atom = ctx.ast.atom(d.as_str());
        directives.push(ctx.ast.directive(SPAN, ctx.ast.string_literal(SPAN, atom, None), atom));
    }
    directives
}

/// Build a new `FunctionBody` from compiled output.
fn build_compiled_body<'a>(
    compiled: &mut CodegenOutput<'a>,
    ctx: &TraverseCtx<'a>,
) -> ABox<'a, FunctionBody<'a>> {
    let directives = build_directives(&compiled.directives, ctx);
    let body = std::mem::replace(&mut compiled.body, ctx.ast.vec());
    ctx.ast.alloc_function_body(SPAN, directives, body)
}

/// Build `FormalParameters` from codegen param names.
///
/// Param names prefixed with `"..."` are emitted as rest elements.
fn build_formal_params_from_codegen<'a>(
    param_names: &[String],
    ctx: &TraverseCtx<'a>,
) -> ABox<'a, FormalParameters<'a>> {
    let mut rest: Option<ABox<'a, FormalParameterRest<'a>>> = None;
    let mut items = ctx.ast.vec_with_capacity(param_names.len());

    for param_name in param_names {
        if let Some(name) = param_name.strip_prefix("...") {
            // Rest element: `...name`
            let binding_pattern =
                ctx.ast.binding_pattern_binding_identifier(SPAN, ctx.ast.atom(name));
            let binding_rest = ctx.ast.binding_rest_element(SPAN, binding_pattern);
            rest = Some(ctx.ast.alloc_formal_parameter_rest(
                SPAN,
                ctx.ast.vec(),
                binding_rest,
                NONE, // type_annotation
            ));
        } else {
            let pattern =
                ctx.ast.binding_pattern_binding_identifier(SPAN, ctx.ast.atom(param_name.as_str()));
            items.push(ctx.ast.formal_parameter(
                SPAN,
                ctx.ast.vec(),
                pattern,
                NONE,  // type_annotation
                NONE,  // initializer
                false, // optional
                None,  // accessibility
                false, // readonly
                false, // override
            ));
        }
    }

    ctx.ast.alloc_formal_parameters(SPAN, FormalParameterKind::FormalParameter, items, rest)
}

/// Replace the inner function body of a memo/forwardRef call expression with compiled output.
fn replace_memo_inner_function_body<'a>(
    call: &mut CallExpression<'a>,
    compiled: &mut CodegenOutput<'a>,
    ctx: &TraverseCtx<'a>,
) {
    if let Some(arg) = call.arguments.first_mut()
        && let Some(expr) = arg.as_expression_mut()
    {
        match expr {
            Expression::FunctionExpression(function) => {
                function.params = build_formal_params_from_codegen(&compiled.params, ctx);
                function.body = Some(build_compiled_body(compiled, ctx));
            }
            Expression::ArrowFunctionExpression(arrow) => {
                arrow.params = build_formal_params_from_codegen(&compiled.params, ctx);
                let directives = build_directives(&compiled.directives, ctx);
                let body = std::mem::replace(&mut compiled.body, ctx.ast.vec());
                arrow.body = ctx.ast.alloc_function_body(SPAN, directives, body);
                arrow.expression = false;
            }
            _ => {}
        }
    }
}

/// Replace the function body within a statement with compiled output.
///
/// Handles `FunctionDeclaration`, `VariableDeclaration` (with function/arrow
/// expression initializers), `ExportDefaultDeclaration`, and
/// `ExportNamedDeclaration`.
fn replace_statement_function<'a>(
    mut stmt: Statement<'a>,
    mut compiled: CodegenOutput<'a>,
    ctx: &TraverseCtx<'a>,
) -> Statement<'a> {
    match &mut stmt {
        Statement::FunctionDeclaration(function) => {
            function.params = build_formal_params_from_codegen(&compiled.params, ctx);
            function.body = Some(build_compiled_body(&mut compiled, ctx));
        }
        Statement::VariableDeclaration(declaration) => {
            for declarator in &mut declaration.declarations {
                let Some(init) = &mut declarator.init else {
                    continue;
                };
                match init {
                    Expression::FunctionExpression(function) => {
                        function.params = build_formal_params_from_codegen(&compiled.params, ctx);
                        function.body = Some(build_compiled_body(&mut compiled, ctx));
                        break;
                    }
                    Expression::ArrowFunctionExpression(arrow) => {
                        arrow.params = build_formal_params_from_codegen(&compiled.params, ctx);
                        let directives = build_directives(&compiled.directives, ctx);
                        let body = std::mem::replace(&mut compiled.body, ctx.ast.vec());
                        arrow.body = ctx.ast.alloc_function_body(SPAN, directives, body);
                        arrow.expression = false;
                        break;
                    }
                    Expression::CallExpression(call)
                        if is_memo_or_forwardref_call(&call.callee) =>
                    {
                        replace_memo_inner_function_body(call, &mut compiled, ctx);
                        break;
                    }
                    _ => {}
                }
            }
        }
        Statement::ExportDefaultDeclaration(export_default) => {
            match &mut export_default.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function)
                | ExportDefaultDeclarationKind::FunctionExpression(function) => {
                    function.params = build_formal_params_from_codegen(&compiled.params, ctx);
                    function.body = Some(build_compiled_body(&mut compiled, ctx));
                }
                ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
                    arrow.params = build_formal_params_from_codegen(&compiled.params, ctx);
                    let directives = build_directives(&compiled.directives, ctx);
                    let body = std::mem::replace(&mut compiled.body, ctx.ast.vec());
                    arrow.body = ctx.ast.alloc_function_body(SPAN, directives, body);
                    arrow.expression = false;
                }
                ExportDefaultDeclarationKind::CallExpression(call)
                    if is_memo_or_forwardref_call(&call.callee) =>
                {
                    replace_memo_inner_function_body(call, &mut compiled, ctx);
                }
                _ => {}
            }
        }
        Statement::ExportNamedDeclaration(export_named) => {
            if let Some(declaration) = &mut export_named.declaration {
                match declaration {
                    Declaration::FunctionDeclaration(function) => {
                        function.params = build_formal_params_from_codegen(&compiled.params, ctx);
                        function.body = Some(build_compiled_body(&mut compiled, ctx));
                    }
                    Declaration::VariableDeclaration(var_decl) => {
                        for declarator in &mut var_decl.declarations {
                            let Some(init) = &mut declarator.init else {
                                continue;
                            };
                            match init {
                                Expression::FunctionExpression(function) => {
                                    function.params =
                                        build_formal_params_from_codegen(&compiled.params, ctx);
                                    function.body = Some(build_compiled_body(&mut compiled, ctx));
                                    break;
                                }
                                Expression::ArrowFunctionExpression(arrow) => {
                                    arrow.params =
                                        build_formal_params_from_codegen(&compiled.params, ctx);
                                    let directives = build_directives(&compiled.directives, ctx);
                                    let body = std::mem::replace(&mut compiled.body, ctx.ast.vec());
                                    arrow.body =
                                        ctx.ast.alloc_function_body(SPAN, directives, body);
                                    arrow.expression = false;
                                    break;
                                }
                                Expression::CallExpression(call)
                                    if is_memo_or_forwardref_call(&call.callee) =>
                                {
                                    replace_memo_inner_function_body(call, &mut compiled, ctx);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    stmt
}

/// Replace the function body within a statement in-place, targeting a specific
/// declarator when `declarator_index` is `Some`. This is used for multi-declarator
/// VariableDeclarations where each compilable declarator gets its own codegen output.
///
/// When `declarator_index` is `None`, falls back to scanning for the first
/// function-like initializer (used for non-VarDecl statement types).
fn replace_statement_function_in_place<'a>(
    stmt: &mut Statement<'a>,
    mut compiled: CodegenOutput<'a>,
    declarator_index: Option<usize>,
    ctx: &TraverseCtx<'a>,
) {
    match stmt {
        Statement::FunctionDeclaration(function) => {
            function.params = build_formal_params_from_codegen(&compiled.params, ctx);
            function.body = Some(build_compiled_body(&mut compiled, ctx));
        }
        Statement::VariableDeclaration(declaration) => {
            replace_var_decl_function(
                &mut declaration.declarations,
                &mut compiled,
                declarator_index,
                ctx,
            );
        }
        Statement::ExportDefaultDeclaration(export_default) => {
            match &mut export_default.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function)
                | ExportDefaultDeclarationKind::FunctionExpression(function) => {
                    function.params = build_formal_params_from_codegen(&compiled.params, ctx);
                    function.body = Some(build_compiled_body(&mut compiled, ctx));
                }
                ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
                    arrow.params = build_formal_params_from_codegen(&compiled.params, ctx);
                    let directives = build_directives(&compiled.directives, ctx);
                    let body = std::mem::replace(&mut compiled.body, ctx.ast.vec());
                    arrow.body = ctx.ast.alloc_function_body(SPAN, directives, body);
                    arrow.expression = false;
                }
                ExportDefaultDeclarationKind::CallExpression(call)
                    if is_memo_or_forwardref_call(&call.callee) =>
                {
                    replace_memo_inner_function_body(call, &mut compiled, ctx);
                }
                _ => {}
            }
        }
        Statement::ExportNamedDeclaration(export_named) => {
            if let Some(declaration) = &mut export_named.declaration {
                match declaration {
                    Declaration::FunctionDeclaration(function) => {
                        function.params = build_formal_params_from_codegen(&compiled.params, ctx);
                        function.body = Some(build_compiled_body(&mut compiled, ctx));
                    }
                    Declaration::VariableDeclaration(var_decl) => {
                        replace_var_decl_function(
                            &mut var_decl.declarations,
                            &mut compiled,
                            declarator_index,
                            ctx,
                        );
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// Replace the function body of a specific declarator (by index) or the first
/// function-like declarator if no index is given.
fn replace_var_decl_function<'a>(
    declarations: &mut AVec<'a, VariableDeclarator<'a>>,
    compiled: &mut CodegenOutput<'a>,
    declarator_index: Option<usize>,
    ctx: &TraverseCtx<'a>,
) {
    let iter: Box<dyn Iterator<Item = &mut VariableDeclarator<'a>>> =
        if let Some(idx) = declarator_index {
            Box::new(declarations.iter_mut().skip(idx).take(1))
        } else {
            Box::new(declarations.iter_mut())
        };
    for declarator in iter {
        let Some(init) = &mut declarator.init else {
            continue;
        };
        match init {
            Expression::FunctionExpression(function) => {
                function.params = build_formal_params_from_codegen(&compiled.params, ctx);
                function.body = Some(build_compiled_body(compiled, ctx));
                break;
            }
            Expression::ArrowFunctionExpression(arrow) => {
                arrow.params = build_formal_params_from_codegen(&compiled.params, ctx);
                let directives = build_directives(&compiled.directives, ctx);
                let body = std::mem::replace(&mut compiled.body, ctx.ast.vec());
                arrow.body = ctx.ast.alloc_function_body(SPAN, directives, body);
                arrow.expression = false;
                break;
            }
            Expression::CallExpression(call) if is_memo_or_forwardref_call(&call.callee) => {
                replace_memo_inner_function_body(call, compiled, ctx);
                break;
            }
            _ => {}
        }
    }
}

/// Build a top-level `FunctionDeclaration` statement from an outlined function.
///
/// Outlined functions are always emitted as `FunctionDeclaration` regardless of
/// the original function's type (matching the TS reference behavior in
/// `insertNewOutlinedFunctionNode`).
fn build_outlined_function_statement<'a>(
    outlined: OutlinedOutput<'a>,
    ctx: &TraverseCtx<'a>,
) -> Statement<'a> {
    let mut codegen = outlined.fn_;

    let directives = build_directives(&codegen.directives, ctx);
    let body = std::mem::replace(&mut codegen.body, ctx.ast.vec());
    let function_body = ctx.ast.alloc_function_body(SPAN, directives, body);

    // Build the function id from the codegen output id.
    let id = codegen.id.as_deref().map(|name| ctx.ast.binding_identifier(SPAN, ctx.ast.atom(name)));

    // Build formal parameters from param names.
    let params = build_formal_params_from_codegen(&codegen.params, ctx);

    let function = ctx.ast.alloc_function(
        SPAN,
        FunctionType::FunctionDeclaration,
        id,
        codegen.generator,
        codegen.is_async,
        false, // declare
        NONE,  // type_parameters
        NONE,  // this_param
        params,
        NONE, // return_type
        Some(function_body),
    );

    Statement::FunctionDeclaration(function)
}

/// Get the scope ID of the function within a statement (if the statement contains one).
///
/// Used to determine the parent scope for newly created AST nodes in the compiled output.
fn get_function_scope_id(stmt: &Statement<'_>) -> Option<ScopeId> {
    match stmt {
        Statement::FunctionDeclaration(f) => f.scope_id.get(),
        Statement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                if let Some(init) = &declarator.init {
                    match init {
                        Expression::FunctionExpression(f) => return f.scope_id.get(),
                        Expression::ArrowFunctionExpression(f) => return f.scope_id.get(),
                        Expression::CallExpression(call)
                            if is_memo_or_forwardref_call(&call.callee) =>
                        {
                            if let Some(arg) = call.arguments.first()
                                && let Some(expr) = arg.as_expression()
                            {
                                match expr {
                                    Expression::FunctionExpression(f) => {
                                        return f.scope_id.get();
                                    }
                                    Expression::ArrowFunctionExpression(f) => {
                                        return f.scope_id.get();
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            None
        }
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(f)
            | ExportDefaultDeclarationKind::FunctionExpression(f) => f.scope_id.get(),
            ExportDefaultDeclarationKind::ArrowFunctionExpression(f) => f.scope_id.get(),
            ExportDefaultDeclarationKind::CallExpression(call)
                if is_memo_or_forwardref_call(&call.callee) =>
            {
                call.arguments.first().and_then(|arg| {
                    arg.as_expression().and_then(|expr| match expr {
                        Expression::FunctionExpression(f) => f.scope_id.get(),
                        Expression::ArrowFunctionExpression(f) => f.scope_id.get(),
                        _ => None,
                    })
                })
            }
            _ => None,
        },
        Statement::ExportNamedDeclaration(export) => {
            export.declaration.as_ref().and_then(|decl| match decl {
                Declaration::FunctionDeclaration(f) => f.scope_id.get(),
                Declaration::VariableDeclaration(var_decl) => {
                    for declarator in &var_decl.declarations {
                        if let Some(init) = &declarator.init {
                            match init {
                                Expression::FunctionExpression(f) => return f.scope_id.get(),
                                Expression::ArrowFunctionExpression(f) => return f.scope_id.get(),
                                Expression::CallExpression(call)
                                    if is_memo_or_forwardref_call(&call.callee) =>
                                {
                                    if let Some(arg) = call.arguments.first()
                                        && let Some(expr) = arg.as_expression()
                                    {
                                        match expr {
                                            Expression::FunctionExpression(f) => {
                                                return f.scope_id.get();
                                            }
                                            Expression::ArrowFunctionExpression(f) => {
                                                return f.scope_id.get();
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    None
                }
                _ => None,
            })
        }
        _ => None,
    }
}

/// Recursively assign scope IDs to all scope-creating AST nodes within a statement.
///
/// The React Compiler codegen creates new AST nodes (BlockStatement, ForStatement, etc.)
/// without scope IDs. The traverse walker requires all such nodes to have valid scope IDs.
/// This function walks the AST and creates child scopes as needed.
fn assign_scope_ids_to_statement(
    stmt: &mut Statement<'_>,
    parent_scope_id: ScopeId,
    ctx: &mut TraverseCtx<'_>,
) {
    match stmt {
        Statement::FunctionDeclaration(f) => {
            // For outlined functions (which have no scope_id yet), create a new scope.
            if f.scope_id.get().is_none() {
                let scope_id =
                    ctx.create_child_scope(parent_scope_id, ScopeFlags::Function | ScopeFlags::Top);
                f.scope_id.set(Some(scope_id));
                if let Some(body) = &mut f.body {
                    assign_scope_ids_to_function_body(body, scope_id, ctx);
                }
            } else {
                // Existing function with valid scope_id: walk its body.
                let fn_scope = f.scope_id.get().unwrap_or(parent_scope_id);
                if let Some(body) = &mut f.body {
                    assign_scope_ids_to_function_body(body, fn_scope, ctx);
                }
            }
        }
        Statement::ExportDefaultDeclaration(export) => match &mut export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(f)
            | ExportDefaultDeclarationKind::FunctionExpression(f) => {
                let fn_scope = f.scope_id.get().unwrap_or(parent_scope_id);
                if let Some(body) = &mut f.body {
                    assign_scope_ids_to_function_body(body, fn_scope, ctx);
                }
            }
            ExportDefaultDeclarationKind::ArrowFunctionExpression(f) => {
                let fn_scope = f.scope_id.get().unwrap_or(parent_scope_id);
                assign_scope_ids_to_function_body(&mut f.body, fn_scope, ctx);
            }
            ExportDefaultDeclarationKind::CallExpression(call) => {
                if let Some(arg) = call.arguments.first_mut()
                    && let Some(expr) = arg.as_expression_mut()
                {
                    assign_scope_ids_to_expression(expr, parent_scope_id, ctx);
                }
            }
            _ => {}
        },
        Statement::ExportNamedDeclaration(export) => {
            if let Some(decl) = &mut export.declaration {
                assign_scope_ids_to_declaration(decl, parent_scope_id, ctx);
            }
        }
        Statement::VariableDeclaration(decl) => {
            for declarator in &mut decl.declarations {
                if let Some(init) = &mut declarator.init {
                    assign_scope_ids_to_expression(init, parent_scope_id, ctx);
                }
            }
        }
        _ => {
            assign_scope_ids_to_statement_inner(stmt, parent_scope_id, ctx);
        }
    }
}

fn assign_scope_ids_to_declaration(
    decl: &mut Declaration<'_>,
    parent_scope_id: ScopeId,
    ctx: &mut TraverseCtx<'_>,
) {
    match decl {
        Declaration::FunctionDeclaration(f) => {
            let fn_scope = f.scope_id.get().unwrap_or(parent_scope_id);
            if let Some(body) = &mut f.body {
                assign_scope_ids_to_function_body(body, fn_scope, ctx);
            }
        }
        Declaration::VariableDeclaration(var_decl) => {
            for declarator in &mut var_decl.declarations {
                if let Some(init) = &mut declarator.init {
                    assign_scope_ids_to_expression(init, parent_scope_id, ctx);
                }
            }
        }
        _ => {}
    }
}

fn assign_scope_ids_to_function_body(
    body: &mut FunctionBody<'_>,
    parent_scope_id: ScopeId,
    ctx: &mut TraverseCtx<'_>,
) {
    for stmt in &mut body.statements {
        assign_scope_ids_to_statement_inner(stmt, parent_scope_id, ctx);
    }
}

/// Inner recursive walker for statements within a function body.
fn assign_scope_ids_to_statement_inner(
    stmt: &mut Statement<'_>,
    parent_scope_id: ScopeId,
    ctx: &mut TraverseCtx<'_>,
) {
    match stmt {
        Statement::BlockStatement(block) => {
            if block.scope_id.get().is_none() {
                let scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::empty());
                block.scope_id.set(Some(scope_id));
                for s in &mut block.body {
                    assign_scope_ids_to_statement_inner(s, scope_id, ctx);
                }
            }
        }
        Statement::IfStatement(if_stmt) => {
            assign_scope_ids_to_statement_inner(&mut if_stmt.consequent, parent_scope_id, ctx);
            if let Some(alternate) = &mut if_stmt.alternate {
                assign_scope_ids_to_statement_inner(alternate, parent_scope_id, ctx);
            }
            assign_scope_ids_to_expression(&mut if_stmt.test, parent_scope_id, ctx);
        }
        Statement::ForStatement(for_stmt) => {
            if for_stmt.scope_id.get().is_none() {
                let scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::empty());
                for_stmt.scope_id.set(Some(scope_id));
                assign_scope_ids_to_statement_inner(&mut for_stmt.body, scope_id, ctx);
            }
        }
        Statement::ForInStatement(for_in) => {
            if for_in.scope_id.get().is_none() {
                let scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::empty());
                for_in.scope_id.set(Some(scope_id));
                assign_scope_ids_to_statement_inner(&mut for_in.body, scope_id, ctx);
            }
        }
        Statement::ForOfStatement(for_of) => {
            if for_of.scope_id.get().is_none() {
                let scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::empty());
                for_of.scope_id.set(Some(scope_id));
                assign_scope_ids_to_statement_inner(&mut for_of.body, scope_id, ctx);
            }
        }
        Statement::WhileStatement(while_stmt) => {
            assign_scope_ids_to_statement_inner(&mut while_stmt.body, parent_scope_id, ctx);
        }
        Statement::DoWhileStatement(do_while) => {
            assign_scope_ids_to_statement_inner(&mut do_while.body, parent_scope_id, ctx);
        }
        Statement::SwitchStatement(switch_stmt) => {
            if switch_stmt.scope_id.get().is_none() {
                let scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::empty());
                switch_stmt.scope_id.set(Some(scope_id));
                for case in &mut switch_stmt.cases {
                    for s in &mut case.consequent {
                        assign_scope_ids_to_statement_inner(s, scope_id, ctx);
                    }
                }
            }
        }
        Statement::TryStatement(try_stmt) => {
            assign_scope_ids_to_block(&mut try_stmt.block, parent_scope_id, ctx);
            if let Some(handler) = &mut try_stmt.handler
                && handler.scope_id.get().is_none()
            {
                let catch_scope_id =
                    ctx.create_child_scope(parent_scope_id, ScopeFlags::CatchClause);
                handler.scope_id.set(Some(catch_scope_id));
                assign_scope_ids_to_block(&mut handler.body, catch_scope_id, ctx);
            }
            if let Some(finalizer) = &mut try_stmt.finalizer {
                assign_scope_ids_to_block(finalizer, parent_scope_id, ctx);
            }
        }
        Statement::LabeledStatement(labeled) => {
            assign_scope_ids_to_statement_inner(&mut labeled.body, parent_scope_id, ctx);
        }
        Statement::VariableDeclaration(decl) => {
            for declarator in &mut decl.declarations {
                if let Some(init) = &mut declarator.init {
                    assign_scope_ids_to_expression(init, parent_scope_id, ctx);
                }
            }
        }
        Statement::ReturnStatement(ret) => {
            if let Some(arg) = &mut ret.argument {
                assign_scope_ids_to_expression(arg, parent_scope_id, ctx);
            }
        }
        Statement::ExpressionStatement(expr_stmt) => {
            assign_scope_ids_to_expression(&mut expr_stmt.expression, parent_scope_id, ctx);
        }
        _ => {}
    }
}

/// Assign scope ID to a BlockStatement if it doesn't have one, and recurse into its body.
fn assign_scope_ids_to_block(
    block: &mut BlockStatement<'_>,
    parent_scope_id: ScopeId,
    ctx: &mut TraverseCtx<'_>,
) {
    if block.scope_id.get().is_none() {
        let scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::empty());
        block.scope_id.set(Some(scope_id));
        for s in &mut block.body {
            assign_scope_ids_to_statement_inner(s, scope_id, ctx);
        }
    }
}

/// Recursively assign scope IDs within expressions (for arrow functions, function
/// expressions, and other scope-creating expressions).
fn assign_scope_ids_to_expression(
    expr: &mut Expression<'_>,
    parent_scope_id: ScopeId,
    ctx: &mut TraverseCtx<'_>,
) {
    match expr {
        Expression::FunctionExpression(f) => {
            if f.scope_id.get().is_none() {
                let scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::Function);
                f.scope_id.set(Some(scope_id));
                if let Some(body) = &mut f.body {
                    assign_scope_ids_to_function_body(body, scope_id, ctx);
                }
            } else {
                let fn_scope = f.scope_id.get().unwrap_or(parent_scope_id);
                if let Some(body) = &mut f.body {
                    assign_scope_ids_to_function_body(body, fn_scope, ctx);
                }
            }
        }
        Expression::ArrowFunctionExpression(f) => {
            if f.scope_id.get().is_none() {
                let scope_id = ctx
                    .create_child_scope(parent_scope_id, ScopeFlags::Function | ScopeFlags::Arrow);
                f.scope_id.set(Some(scope_id));
                assign_scope_ids_to_function_body(&mut f.body, scope_id, ctx);
            } else {
                let fn_scope = f.scope_id.get().unwrap_or(parent_scope_id);
                assign_scope_ids_to_function_body(&mut f.body, fn_scope, ctx);
            }
        }
        Expression::ConditionalExpression(cond) => {
            assign_scope_ids_to_expression(&mut cond.test, parent_scope_id, ctx);
            assign_scope_ids_to_expression(&mut cond.consequent, parent_scope_id, ctx);
            assign_scope_ids_to_expression(&mut cond.alternate, parent_scope_id, ctx);
        }
        Expression::SequenceExpression(seq) => {
            for e in &mut seq.expressions {
                assign_scope_ids_to_expression(e, parent_scope_id, ctx);
            }
        }
        Expression::CallExpression(call) => {
            assign_scope_ids_to_expression(&mut call.callee, parent_scope_id, ctx);
            for arg in &mut call.arguments {
                if let Some(e) = arg.as_expression_mut() {
                    assign_scope_ids_to_expression(e, parent_scope_id, ctx);
                }
            }
        }
        Expression::NewExpression(new_expr) => {
            assign_scope_ids_to_expression(&mut new_expr.callee, parent_scope_id, ctx);
            for arg in &mut new_expr.arguments {
                if let Some(e) = arg.as_expression_mut() {
                    assign_scope_ids_to_expression(e, parent_scope_id, ctx);
                }
            }
        }
        Expression::AssignmentExpression(assign) => {
            assign_scope_ids_to_expression(&mut assign.right, parent_scope_id, ctx);
        }
        Expression::LogicalExpression(logical) => {
            assign_scope_ids_to_expression(&mut logical.left, parent_scope_id, ctx);
            assign_scope_ids_to_expression(&mut logical.right, parent_scope_id, ctx);
        }
        Expression::BinaryExpression(binary) => {
            assign_scope_ids_to_expression(&mut binary.left, parent_scope_id, ctx);
            assign_scope_ids_to_expression(&mut binary.right, parent_scope_id, ctx);
        }
        Expression::UnaryExpression(unary) => {
            assign_scope_ids_to_expression(&mut unary.argument, parent_scope_id, ctx);
        }
        Expression::ArrayExpression(arr) => {
            for elem in &mut arr.elements {
                if let ArrayExpressionElement::SpreadElement(spread) = elem {
                    assign_scope_ids_to_expression(&mut spread.argument, parent_scope_id, ctx);
                } else if let Some(e) = elem.as_expression_mut() {
                    assign_scope_ids_to_expression(e, parent_scope_id, ctx);
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            for prop in &mut obj.properties {
                match prop {
                    ObjectPropertyKind::ObjectProperty(p) => {
                        assign_scope_ids_to_expression(&mut p.value, parent_scope_id, ctx);
                    }
                    ObjectPropertyKind::SpreadProperty(s) => {
                        assign_scope_ids_to_expression(&mut s.argument, parent_scope_id, ctx);
                    }
                }
            }
        }
        Expression::TemplateLiteral(tmpl) => {
            for e in &mut tmpl.expressions {
                assign_scope_ids_to_expression(e, parent_scope_id, ctx);
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            assign_scope_ids_to_expression(&mut tagged.tag, parent_scope_id, ctx);
            for e in &mut tagged.quasi.expressions {
                assign_scope_ids_to_expression(e, parent_scope_id, ctx);
            }
        }
        Expression::ParenthesizedExpression(paren) => {
            assign_scope_ids_to_expression(&mut paren.expression, parent_scope_id, ctx);
        }
        Expression::AwaitExpression(await_expr) => {
            assign_scope_ids_to_expression(&mut await_expr.argument, parent_scope_id, ctx);
        }
        Expression::YieldExpression(yield_expr) => {
            if let Some(arg) = &mut yield_expr.argument {
                assign_scope_ids_to_expression(arg, parent_scope_id, ctx);
            }
        }
        // Member expressions
        Expression::StaticMemberExpression(member) => {
            assign_scope_ids_to_expression(&mut member.object, parent_scope_id, ctx);
        }
        Expression::ComputedMemberExpression(member) => {
            assign_scope_ids_to_expression(&mut member.object, parent_scope_id, ctx);
            assign_scope_ids_to_expression(&mut member.expression, parent_scope_id, ctx);
        }
        Expression::PrivateFieldExpression(member) => {
            assign_scope_ids_to_expression(&mut member.object, parent_scope_id, ctx);
        }
        // JSX expressions
        Expression::JSXElement(jsx) => {
            assign_scope_ids_to_jsx_element(jsx, parent_scope_id, ctx);
        }
        Expression::JSXFragment(jsx) => {
            assign_scope_ids_to_jsx_fragment(jsx, parent_scope_id, ctx);
        }
        _ => {}
    }
}

/// Assign scope IDs within a JSX element's attributes and children.
fn assign_scope_ids_to_jsx_element(
    element: &mut JSXElement<'_>,
    parent_scope_id: ScopeId,
    ctx: &mut TraverseCtx<'_>,
) {
    // Walk attributes for expression containers (e.g., onClick={() => ...})
    for attr in &mut element.opening_element.attributes {
        if let JSXAttributeItem::Attribute(a) = attr
            && let Some(value) = &mut a.value
            && let JSXAttributeValue::ExpressionContainer(container) = value
            && let Some(e) = container.expression.as_expression_mut()
        {
            assign_scope_ids_to_expression(e, parent_scope_id, ctx);
        }
        if let JSXAttributeItem::SpreadAttribute(spread) = attr {
            assign_scope_ids_to_expression(&mut spread.argument, parent_scope_id, ctx);
        }
    }
    // Walk children
    for child in &mut element.children {
        match child {
            JSXChild::ExpressionContainer(container) => {
                if let Some(e) = container.expression.as_expression_mut() {
                    assign_scope_ids_to_expression(e, parent_scope_id, ctx);
                }
            }
            JSXChild::Element(el) => {
                assign_scope_ids_to_jsx_element(el, parent_scope_id, ctx);
            }
            JSXChild::Fragment(frag) => {
                assign_scope_ids_to_jsx_fragment(frag, parent_scope_id, ctx);
            }
            JSXChild::Spread(spread) => {
                assign_scope_ids_to_expression(&mut spread.expression, parent_scope_id, ctx);
            }
            JSXChild::Text(_) => {}
        }
    }
}

/// Assign scope IDs within a JSX fragment's children.
fn assign_scope_ids_to_jsx_fragment(
    fragment: &mut JSXFragment<'_>,
    parent_scope_id: ScopeId,
    ctx: &mut TraverseCtx<'_>,
) {
    for child in &mut fragment.children {
        match child {
            JSXChild::ExpressionContainer(container) => {
                if let Some(e) = container.expression.as_expression_mut() {
                    assign_scope_ids_to_expression(e, parent_scope_id, ctx);
                }
            }
            JSXChild::Element(el) => {
                assign_scope_ids_to_jsx_element(el, parent_scope_id, ctx);
            }
            JSXChild::Fragment(frag) => {
                assign_scope_ids_to_jsx_fragment(frag, parent_scope_id, ctx);
            }
            JSXChild::Spread(spread) => {
                assign_scope_ids_to_expression(&mut spread.expression, parent_scope_id, ctx);
            }
            JSXChild::Text(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_is_default_mode() {
        assert_eq!(parse_compilation_mode(None), CompilationMode::Infer);
        assert_eq!(parse_compilation_mode(Some("infer")), CompilationMode::Infer);
    }

    #[test]
    #[should_panic(expected = "Invalid compilationMode")]
    fn invalid_compilation_mode_panics() {
        parse_compilation_mode(Some("unknown"));
    }

    #[test]
    fn parse_valid_modes() {
        assert_eq!(parse_compilation_mode(Some("all")), CompilationMode::All);
        assert_eq!(parse_compilation_mode(Some("annotation")), CompilationMode::Annotation);
        assert_eq!(parse_compilation_mode(Some("syntax")), CompilationMode::Syntax);
        assert_eq!(parse_compilation_mode(Some("infer")), CompilationMode::Infer);
    }

    #[test]
    #[should_panic(expected = "Invalid target")]
    fn invalid_target_panics() {
        parse_target(Some("react-20"));
    }

    #[test]
    #[should_panic(expected = "Invalid panicThreshold")]
    fn invalid_panic_threshold_panics() {
        parse_panic_threshold(Some("invalid"));
    }

    #[test]
    fn parse_valid_output_modes() {
        assert_eq!(parse_output_mode(Some("client")), Some(CompilerOutputMode::Client));
        assert_eq!(parse_output_mode(Some("ssr")), Some(CompilerOutputMode::Ssr));
        assert_eq!(parse_output_mode(Some("lint")), Some(CompilerOutputMode::Lint));
        assert_eq!(parse_output_mode(None), None);
    }

    #[test]
    #[should_panic(expected = "Invalid outputMode")]
    fn invalid_output_mode_panics() {
        parse_output_mode(Some("invalid"));
    }
}
