use oxc_allocator::{Box as ABox, TakeIn, Vec as AVec};
use oxc_ast::NONE;
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_react_compiler::{
    compiler_error::{CompilerError, CompilerErrorEntry, SourceLocation},
    entrypoint::{
        imports::validate_restricted_imports,
        options::{CompilationMode, CompilerReactTarget, DynamicGatingOptions, PanicThreshold},
        pipeline::{resolve_output_mode, run_codegen, run_pipeline},
        program::{
            ErrorAction, find_directive_disabling_memoization, handle_compilation_error,
            should_compile_function,
        },
        suppression::{
            DEFAULT_ESLINT_SUPPRESSION_RULES, SuppressionRange, find_program_suppressions,
        },
    },
    hir::{
        NonLocalBinding, ReactFunctionType,
        build_hir::{LowerableFunction, collect_import_bindings, lower},
        environment::{CompilerOutputMode, Environment, EnvironmentConfig, ExternalFunction},
    },
    reactive_scopes::codegen_reactive_function::{CodegenOutput, OutlinedOutput},
};
use oxc_semantic::{ScopeFlags, ScopeId, SymbolFlags};
use oxc_span::{Atom, SPAN, Span};
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
    /// Array of filename regex patterns to filter which files get compiled.
    /// When set, only files whose path matches at least one pattern will be compiled.
    ///
    /// NOTE: Filtering logic is not yet implemented because the filename is not
    /// directly available in `enter_program`. This field is wired through so that
    /// the option can be passed from the NAPI layer.
    pub sources: Option<Vec<String>>,
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
    target: CompilerReactTarget,
    environment_config: EnvironmentConfig,
    output_mode: CompilerOutputMode,
    ignore_use_no_forget: bool,
    custom_opt_out_directives: Option<Vec<String>>,
    has_module_scope_opt_out: bool,
    _gating: Option<ExternalFunction>,
    _dynamic_gating: Option<DynamicGatingOptions>,
    outer_bindings: FxHashMap<String, NonLocalBinding>,
    suppressions: Vec<SuppressionRange>,
}

/// Result of compiling a single function.
struct CompileResult<'a> {
    /// Index of the statement in `program.body` that was compiled.
    index: usize,
    /// The codegen output for the compiled function.
    output: CodegenOutput<'a>,
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
        Self {
            options,
            panic_threshold,
            target,
            environment_config,
            output_mode,
            ignore_use_no_forget,
            custom_opt_out_directives,
            has_module_scope_opt_out: false,
            _gating: gating,
            _dynamic_gating: dynamic_gating,
            outer_bindings: FxHashMap::default(),
            suppressions: Vec::new(),
        }
    }

    pub fn enter_program<'a>(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.options.enabled {
            return;
        }

        let runtime_module = get_runtime_module(&self.target);

        // Check for already-compiled marker import: `import { c } from "<runtime_module>"`.
        // If found, skip compilation entirely to prevent double-compilation.
        // Port of `hasMemoCacheFunctionImport` from Program.ts.
        for stmt in &program.body {
            if let Statement::ImportDeclaration(import) = stmt
                && import.source.value.as_str() == runtime_module
            {
                return;
            }
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
        // Port of findProgramSuppressions call in Program.ts.
        let rule_names: Vec<String> =
            self.options.eslint_suppression_rules.clone().unwrap_or_else(|| {
                DEFAULT_ESLINT_SUPPRESSION_RULES.iter().map(|s| (*s).to_string()).collect()
            });
        self.suppressions = find_program_suppressions(
            &program.comments,
            program.source_text,
            Some(&rule_names),
            self.options.flow_suppressions.unwrap_or(true),
        );

        self.outer_bindings = collect_import_bindings(&program.body);

        // Pre-generate the cache function UID before compiling any functions.
        // This ensures the same name (e.g. "_c" or "_c2") is used in both the
        // import binding and the codegen body references.
        let cache_binding = ctx.generate_uid_in_root_scope("c", SymbolFlags::Import);
        let cache_identifier_name = cache_binding.name.to_string();

        // Phase 1: Compile all candidate functions, collecting results by statement index.
        let mut compiled_results: Vec<CompileResult<'a>> = Vec::new();
        for (index, statement) in program.body.iter().enumerate() {
            if let Some(output) = self.compile_statement(statement, &cache_identifier_name, ctx) {
                compiled_results.push(CompileResult { index, output });
            }
        }

        // Track whether any compiled function needs memo import (updated across phases).
        let mut needs_memo_import = compiled_results.iter().any(|r| r.output.memo_slots_used > 0);

        let has_top_level_results = !compiled_results.is_empty();

        // Phase 2 (conditional): Rebuild program.body, replacing compiled functions
        // and inserting outlined functions after the replaced statement.
        // Track which indices in new_body correspond to replaced/outlined statements
        // that need scope IDs assigned.
        let mut replaced_indices: Vec<usize> = Vec::new();
        // Track which indices in new_body correspond to compiled top-level statements
        // (used by nested discovery to skip already-compiled functions).
        let mut compiled_stmt_indices: FxHashSet<usize> = FxHashSet::default();

        let mut new_body = if has_top_level_results {
            let mut result_map: FxHashMap<usize, CodegenOutput<'a>> = FxHashMap::default();
            for result in compiled_results {
                result_map.insert(result.index, result.output);
            }

            let old_body = program.body.take_in(ctx.ast);
            let mut new_body = ctx.ast.vec_with_capacity(old_body.len());

            for (i, stmt) in old_body.into_iter().enumerate() {
                if let Some(mut compiled) = result_map.remove(&i) {
                    // Extract outlined functions before consuming compiled for replacement.
                    let outlined_fns = std::mem::take(&mut compiled.outlined);
                    let replaced = replace_statement_function(stmt, compiled, ctx);
                    let new_idx = new_body.len();
                    replaced_indices.push(new_idx);
                    compiled_stmt_indices.insert(new_idx);
                    new_body.push(replaced);
                    // Process outlined functions using a queue, matching TS Program.ts
                    // lines 426-454. Outlined functions with fn_type are requeued for
                    // full compilation; their own outlined outputs are then processed.
                    let mut outlined_queue: Vec<OutlinedOutput<'a>> = outlined_fns;
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
        // - In `all` mode, only top-level functions are compiled (skip nested discovery)
        // - Skip class bodies (ClassDeclaration/ClassExpression)
        // - Skip walking into already-compiled function bodies (fn.skip() in TS)
        // - `alreadyCompiled` set prevents double-compilation
        let compilation_mode = parse_compilation_mode(self.options.compilation_mode.as_deref());
        let mut compiled_any_nested = false;
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
        }

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
                Atom::from(runtime_module),
                Atom::from("c"),
                cache_binding,
                false,
            );
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
    /// output on success.
    fn compile_statement<'a>(
        &self,
        statement: &Statement<'a>,
        cache_identifier_name: &str,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<CodegenOutput<'a>> {
        match statement {
            Statement::FunctionDeclaration(function) => {
                let directives = function_directives(function);
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
            }
            Statement::VariableDeclaration(declaration) => {
                // For variable declarations, we only compile the first function-like
                // initializer we find. Multiple declarations in one statement with
                // separate compiled functions would be unusual.
                for declarator in &declaration.declarations {
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
                            let lowerable_function = LowerableFunction::Function(function);
                            let result = self.compile_function(
                                &lowerable_function,
                                function_name,
                                &directives,
                                function.span,
                                false,
                                cache_identifier_name,
                                ctx,
                            );
                            if result.is_some() {
                                return result;
                            }
                        }
                        Expression::ArrowFunctionExpression(arrow) => {
                            let directives = arrow_directives(arrow);
                            let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                            let result = self.compile_function(
                                &lowerable_function,
                                binding_name,
                                &directives,
                                arrow.span,
                                false,
                                cache_identifier_name,
                                ctx,
                            );
                            if result.is_some() {
                                return result;
                            }
                        }
                        Expression::CallExpression(call)
                            if is_memo_or_forwardref_call(&call.callee) =>
                        {
                            if let Some(result) = self.compile_memo_or_forwardref_arg(
                                call,
                                binding_name,
                                cache_identifier_name,
                                ctx,
                            ) {
                                return Some(result);
                            }
                        }
                        _ => {}
                    }
                }
                None
            }
            Statement::ExportDefaultDeclaration(export_default) => {
                match &export_default.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(function)
                    | ExportDefaultDeclarationKind::FunctionExpression(function) => {
                        let directives = function_directives(function);
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
                    }
                    ExportDefaultDeclarationKind::CallExpression(call)
                        if is_memo_or_forwardref_call(&call.callee) =>
                    {
                        self.compile_memo_or_forwardref_arg(call, None, cache_identifier_name, ctx)
                    }
                    _ => None,
                }
            }
            Statement::ExportNamedDeclaration(export_named) => {
                let Some(declaration) = &export_named.declaration else {
                    return None;
                };
                match declaration {
                    Declaration::FunctionDeclaration(function) => {
                        let directives = function_directives(function);
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
                    }
                    Declaration::VariableDeclaration(var_decl) => {
                        for declarator in &var_decl.declarations {
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
                                    let lowerable_function = LowerableFunction::Function(function);
                                    let result = self.compile_function(
                                        &lowerable_function,
                                        function_name,
                                        &directives,
                                        function.span,
                                        false,
                                        cache_identifier_name,
                                        ctx,
                                    );
                                    if result.is_some() {
                                        return result;
                                    }
                                }
                                Expression::ArrowFunctionExpression(arrow) => {
                                    let directives = arrow_directives(arrow);
                                    let lowerable_function =
                                        LowerableFunction::ArrowFunction(arrow);
                                    let result = self.compile_function(
                                        &lowerable_function,
                                        binding_name,
                                        &directives,
                                        arrow.span,
                                        false,
                                        cache_identifier_name,
                                        ctx,
                                    );
                                    if result.is_some() {
                                        return result;
                                    }
                                }
                                Expression::CallExpression(call)
                                    if is_memo_or_forwardref_call(&call.callee) =>
                                {
                                    if let Some(result) = self.compile_memo_or_forwardref_arg(
                                        call,
                                        binding_name,
                                        cache_identifier_name,
                                        ctx,
                                    ) {
                                        return Some(result);
                                    }
                                }
                                _ => {}
                            }
                        }
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
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
            Environment::new(fn_type, self.output_mode, self.environment_config.clone());

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

        let codegen_output =
            match run_codegen(pipeline_output, &environment, ctx.ast, cache_identifier_name) {
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
            Environment::new(fn_type, self.output_mode, self.environment_config.clone());

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

        let codegen_output =
            match run_codegen(pipeline_output, &environment, ctx.ast, cache_identifier_name) {
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
            // Variable declarations: check initializers for function expressions.
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
            // If statements: recurse into consequent and alternate.
            Statement::IfStatement(if_stmt) => {
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
            // For/while/do-while: recurse into body.
            Statement::ForStatement(for_stmt) => {
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
            }
            // Switch: recurse into case consequents.
            Statement::SwitchStatement(switch_stmt) => {
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
            // Assignment expressions: recurse into right-hand side.
            Expression::AssignmentExpression(assign) => {
                self.walk_expression_for_nested_functions(
                    &mut assign.right,
                    None,
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
                            self.walk_expression_for_nested_functions(
                                &mut p.value,
                                None,
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
        _ => CompilationMode::Infer,
    }
}

fn parse_output_mode(mode: Option<&str>) -> Option<CompilerOutputMode> {
    match mode {
        Some("client") => Some(CompilerOutputMode::Client),
        Some("ssr") => Some(CompilerOutputMode::Ssr),
        Some("lint") => Some(CompilerOutputMode::Lint),
        _ => None,
    }
}

fn parse_panic_threshold(threshold: Option<&str>) -> PanicThreshold {
    match threshold {
        Some("all_errors") => PanicThreshold::AllErrors,
        Some("critical_errors") => PanicThreshold::CriticalErrors,
        _ => PanicThreshold::None,
    }
}

/// Parse a target string into a `CompilerReactTarget`.
///
/// Maps string values to targets:
/// - "react-17" / "17" -> React17
/// - "react-18" / "18" -> React18
/// - "react-19" / "19" (default) -> React19
fn parse_target(target: Option<&str>) -> CompilerReactTarget {
    match target {
        Some("react-17" | "17") => CompilerReactTarget::React17,
        Some("react-18" | "18") => CompilerReactTarget::React18,
        _ => CompilerReactTarget::React19,
    }
}

/// Get the runtime module name for the given target.
///
/// Port of `getReactCompilerRuntimeModule` from Program.ts.
fn get_runtime_module(target: &CompilerReactTarget) -> &'static str {
    match target {
        CompilerReactTarget::React17 | CompilerReactTarget::React18 => "react-compiler-runtime",
        CompilerReactTarget::React19 => "react/compiler-runtime",
        CompilerReactTarget::MetaInternal { .. } => "react",
    }
}

/// Filter suppression ranges to those that affect a given function span.
///
/// Port of `filterSuppressionsThatAffectFunction` from Suppression.ts.
///
/// A suppression affects a function if:
/// 1. The suppression is within the function's body; or
/// 2. The suppression wraps the function
fn filter_suppressions_that_affect_function(
    suppressions: &[SuppressionRange],
    fn_span: Span,
) -> Vec<&SuppressionRange> {
    let fn_start = fn_span.start;
    let fn_end = fn_span.end;

    suppressions
        .iter()
        .filter(|s| {
            let disable_start = s.start;

            // The suppression is within the function
            let within = disable_start > fn_start
                && match s.end {
                    None => true,
                    Some(enable_end) => enable_end < fn_end,
                };

            // The suppression wraps the function
            let wraps = disable_start < fn_start
                && match s.end {
                    None => true,
                    Some(enable_end) => enable_end > fn_end,
                };

            within || wraps
        })
        .collect()
}

/// Convert suppression ranges that affect a function into a CompilerError.
///
/// Port of `suppressionsToCompilerError` from Suppression.ts.
fn suppressions_to_compiler_error(suppressions: &[&SuppressionRange]) -> CompilerError {
    use oxc_react_compiler::compiler_error::{
        CompilerErrorDetail, CompilerErrorDetailOptions, ErrorCategory,
    };
    use oxc_react_compiler::entrypoint::suppression::SuppressionSource;

    let mut error = CompilerError::new();
    for suppression in suppressions {
        let reason = match suppression.source {
            SuppressionSource::Eslint => {
                "React Compiler has skipped optimizing this component because one or more React ESLint rules were disabled. React Compiler only works when it can safely apply React rules of hooks and other React rules."
            }
            SuppressionSource::Flow => {
                "React Compiler has skipped optimizing this component because a Flow suppression was found."
            }
        };
        error.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
            category: ErrorCategory::Todo,
            reason: reason.to_string(),
            description: None,
            loc: Some(SourceLocation::Source(Span::new(
                suppression.start,
                suppression.end.unwrap_or(suppression.start),
            ))),
            suggestions: None,
        }));
    }
    error
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
        assert_eq!(parse_compilation_mode(Some("unknown")), CompilationMode::Infer);
    }

    #[test]
    fn parse_valid_modes() {
        assert_eq!(parse_compilation_mode(Some("all")), CompilationMode::All);
        assert_eq!(parse_compilation_mode(Some("annotation")), CompilationMode::Annotation);
        assert_eq!(parse_compilation_mode(Some("syntax")), CompilationMode::Syntax);
        assert_eq!(parse_compilation_mode(Some("infer")), CompilationMode::Infer);
    }
}
