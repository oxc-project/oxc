use oxc_allocator::{Box as ABox, TakeIn, Vec as AVec};
use oxc_ast::NONE;
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_react_compiler::{
    compiler_error::{CompilerError, CompilerErrorEntry, SourceLocation},
    entrypoint::{
        options::{CompilationMode, PanicThreshold},
        pipeline::{run_codegen, run_pipeline},
        program::{ErrorAction, handle_compilation_error, should_compile_function},
    },
    hir::{
        NonLocalBinding,
        build_hir::{LowerableFunction, collect_import_bindings, lower},
        environment::{CompilerOutputMode, Environment, EnvironmentConfig},
    },
    reactive_scopes::codegen_reactive_function::{CodegenOutput, OutlinedOutput},
};
use oxc_semantic::SymbolFlags;
use oxc_span::{Atom, SPAN};
use rustc_hash::FxHashMap;
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
}

/// React Compiler transformer plugin.
///
/// Runs the React Compiler analysis and codegen pipeline in the transformer,
/// replacing compiled functions in the output AST and injecting the
/// `react/compiler-runtime` import when memoization is used.
pub struct ReactCompiler {
    options: ReactCompilerOptions,
    panic_threshold: PanicThreshold,
    outer_bindings: FxHashMap<String, NonLocalBinding>,
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
        Self { options, panic_threshold, outer_bindings: FxHashMap::default() }
    }

    pub fn enter_program<'a>(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.options.enabled {
            return;
        }

        // Check for already-compiled marker import: `import { c } from "react/compiler-runtime"`.
        // If found, skip compilation entirely to prevent double-compilation.
        for stmt in program.body.iter() {
            if let Statement::ImportDeclaration(import) = stmt {
                if import.source.value == "react/compiler-runtime" {
                    return;
                }
            }
        }

        // Check for module-level opt-out directives
        for directive in &program.directives {
            let value = directive.directive.as_str();
            if value == "use no memo" || value == "use no forget" {
                return;
            }
        }

        self.outer_bindings = collect_import_bindings(&program.body);

        // Phase 1: Compile all candidate functions, collecting results by statement index.
        let mut compiled_results: Vec<CompileResult<'a>> = Vec::new();
        for (index, statement) in program.body.iter().enumerate() {
            if let Some(output) = self.compile_statement(statement, ctx) {
                compiled_results.push(CompileResult { index, output });
            }
        }

        if compiled_results.is_empty() {
            return;
        }

        // Phase 2: Inject `import { c as _c } from "react/compiler-runtime"` if any
        // compiled function uses memo slots.
        let needs_memo_import = compiled_results.iter().any(|r| r.output.memo_slots_used > 0);
        if needs_memo_import {
            let binding = ctx.generate_uid_in_root_scope("c", SymbolFlags::Import);
            ctx.state.module_imports.add_named_import(
                Atom::from("react/compiler-runtime"),
                Atom::from("c"),
                binding,
                false,
            );
        }

        // Phase 3: Rebuild program.body, replacing compiled functions and inserting
        // outlined functions after the replaced statement.
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
                new_body.push(replaced);
                // Insert outlined functions as top-level FunctionDeclarations after
                // the replaced statement.
                for outlined in outlined_fns {
                    new_body.push(build_outlined_function_statement(outlined, ctx));
                }
            } else {
                new_body.push(stmt);
            }
        }

        program.body = new_body;
    }

    /// Try to compile the function(s) within a statement, returning the codegen
    /// output on success.
    fn compile_statement<'a>(
        &self,
        statement: &Statement<'a>,
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
                            ctx,
                        )
                    }
                    ExportDefaultDeclarationKind::CallExpression(call)
                        if is_memo_or_forwardref_call(&call.callee) =>
                    {
                        self.compile_memo_or_forwardref_arg(call, None, ctx)
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
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<CodegenOutput<'a>> {
        let fn_type = should_compile_function(
            name,
            directives,
            parse_compilation_mode(self.options.compilation_mode.as_deref()),
            is_memo_or_forwardref_arg,
        )?;

        let environment =
            Environment::new(fn_type, CompilerOutputMode::Client, EnvironmentConfig::default());

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
                    Self::report_compiler_error(&diagnostic, fallback_span, self.panic_threshold, ctx);
                }
                return None;
            }
        };

        // Report accumulated non-fatal diagnostics from the pipeline.
        for diagnostic in hir_function.env.take_diagnostics() {
            Self::report_compiler_error(&diagnostic, fallback_span, self.panic_threshold, ctx);
        }

        match run_codegen(pipeline_output, &environment, ctx.ast) {
            Ok(codegen_output) => Some(codegen_output),
            Err(error) => {
                Self::report_compiler_error(&error, fallback_span, self.panic_threshold, ctx);
                None
            }
        }
    }

    /// Try to compile the inner function of a memo/forwardRef call expression.
    fn compile_memo_or_forwardref_arg<'a>(
        &self,
        call: &CallExpression<'a>,
        binding_name: Option<&str>,
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
                    ctx,
                )
            }
            _ => None,
        }
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

fn parse_panic_threshold(threshold: Option<&str>) -> PanicThreshold {
    match threshold {
        Some("all_errors") => PanicThreshold::AllErrors,
        Some("critical_errors") => PanicThreshold::CriticalErrors,
        _ => PanicThreshold::None,
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

/// Replace the inner function body of a memo/forwardRef call expression with compiled output.
fn replace_memo_inner_function_body<'a>(
    call: &mut CallExpression<'a>,
    compiled: &mut CodegenOutput<'a>,
    ctx: &TraverseCtx<'a>,
) {
    if let Some(arg) = call.arguments.first_mut() {
        if let Some(expr) = arg.as_expression_mut() {
            match expr {
                Expression::FunctionExpression(function) => {
                    function.body = Some(build_compiled_body(compiled, ctx));
                }
                Expression::ArrowFunctionExpression(arrow) => {
                    let directives = build_directives(&compiled.directives, ctx);
                    let body = std::mem::replace(&mut compiled.body, ctx.ast.vec());
                    arrow.body = ctx.ast.alloc_function_body(SPAN, directives, body);
                    arrow.expression = false;
                }
                _ => {}
            }
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
            function.body = Some(build_compiled_body(&mut compiled, ctx));
        }
        Statement::VariableDeclaration(declaration) => {
            for declarator in &mut declaration.declarations {
                let Some(init) = &mut declarator.init else {
                    continue;
                };
                match init {
                    Expression::FunctionExpression(function) => {
                        function.body = Some(build_compiled_body(&mut compiled, ctx));
                        break;
                    }
                    Expression::ArrowFunctionExpression(arrow) => {
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
                    function.body = Some(build_compiled_body(&mut compiled, ctx));
                }
                ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
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
                        function.body = Some(build_compiled_body(&mut compiled, ctx));
                    }
                    Declaration::VariableDeclaration(var_decl) => {
                        for declarator in &mut var_decl.declarations {
                            let Some(init) = &mut declarator.init else {
                                continue;
                            };
                            match init {
                                Expression::FunctionExpression(function) => {
                                    function.body = Some(build_compiled_body(&mut compiled, ctx));
                                    break;
                                }
                                Expression::ArrowFunctionExpression(arrow) => {
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
    let id =
        codegen.id.as_deref().map(|name| ctx.ast.binding_identifier(SPAN, ctx.ast.atom(name)));

    // Build formal parameters from param names.
    let params = {
        let mut items = ctx.ast.vec_with_capacity(codegen.params.len());
        for param_name in &codegen.params {
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
        ctx.ast.alloc_formal_parameters(SPAN, FormalParameterKind::FormalParameter, items, NONE)
    };

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
