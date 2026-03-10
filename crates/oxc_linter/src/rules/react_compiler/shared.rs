//! Shared compilation helpers for the React Compiler lint rules.
//!
//! These functions walk statements, find compilable functions, and run the
//! React Compiler pipeline, collecting diagnostics into a `Vec<CachedDiagnostic>`
//! rather than reporting them directly via `LintContext`.

use oxc_ast::ast::*;
use oxc_react_compiler::{
    compiler_error::{CompilerError, CompilerErrorEntry, ErrorSeverity, SourceLocation},
    entrypoint::{
        pipeline::run_pipeline,
        program::{find_directive_disabling_memoization, should_compile_function},
    },
    hir::{
        NonLocalBinding,
        build_hir::{LowerableFunction, lower},
        environment::{CompilerOutputMode, Environment},
    },
};
use oxc_span::Span;
use rustc_hash::FxHashMap;

use super::cache::CachedDiagnostic;
use super::react_compiler_rule::ReactCompilerConfig;

pub fn walk_statements<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
    outer_bindings: &FxHashMap<String, NonLocalBinding>,
    config: &ReactCompilerConfig,
    diagnostics: &mut Vec<CachedDiagnostic>,
) {
    for statement in statements {
        lint_statement(statement, outer_bindings, config, diagnostics);
        walk_nested_statement(statement, outer_bindings, config, diagnostics);
    }
}

fn walk_nested_statement<'a>(
    statement: &'a Statement<'a>,
    outer_bindings: &FxHashMap<String, NonLocalBinding>,
    config: &ReactCompilerConfig,
    diagnostics: &mut Vec<CachedDiagnostic>,
) {
    match statement {
        Statement::BlockStatement(block) => {
            walk_statements(&block.body, outer_bindings, config, diagnostics);
        }
        Statement::IfStatement(s) => {
            walk_nested_statement(&s.consequent, outer_bindings, config, diagnostics);
            if let Some(alt) = &s.alternate {
                walk_nested_statement(alt, outer_bindings, config, diagnostics);
            }
        }
        Statement::ForStatement(s) => {
            lint_statement(&s.body, outer_bindings, config, diagnostics);
            walk_nested_statement(&s.body, outer_bindings, config, diagnostics);
        }
        Statement::ForInStatement(s) => {
            lint_statement(&s.body, outer_bindings, config, diagnostics);
            walk_nested_statement(&s.body, outer_bindings, config, diagnostics);
        }
        Statement::ForOfStatement(s) => {
            lint_statement(&s.body, outer_bindings, config, diagnostics);
            walk_nested_statement(&s.body, outer_bindings, config, diagnostics);
        }
        Statement::WhileStatement(s) => {
            lint_statement(&s.body, outer_bindings, config, diagnostics);
            walk_nested_statement(&s.body, outer_bindings, config, diagnostics);
        }
        Statement::DoWhileStatement(s) => {
            lint_statement(&s.body, outer_bindings, config, diagnostics);
            walk_nested_statement(&s.body, outer_bindings, config, diagnostics);
        }
        Statement::TryStatement(s) => {
            walk_statements(&s.block.body, outer_bindings, config, diagnostics);
            if let Some(handler) = &s.handler {
                walk_statements(&handler.body.body, outer_bindings, config, diagnostics);
            }
            if let Some(finalizer) = &s.finalizer {
                walk_statements(&finalizer.body, outer_bindings, config, diagnostics);
            }
        }
        Statement::SwitchStatement(s) => {
            for case in &s.cases {
                walk_statements(&case.consequent, outer_bindings, config, diagnostics);
            }
        }
        Statement::LabeledStatement(s) => {
            lint_statement(&s.body, outer_bindings, config, diagnostics);
            walk_nested_statement(&s.body, outer_bindings, config, diagnostics);
        }
        _ => {}
    }
}

fn lint_statement<'a>(
    statement: &'a Statement<'a>,
    outer_bindings: &FxHashMap<String, NonLocalBinding>,
    config: &ReactCompilerConfig,
    diagnostics: &mut Vec<CachedDiagnostic>,
) {
    match statement {
        Statement::FunctionDeclaration(function) => {
            let directives = function_directives(function);
            let lowerable_function = LowerableFunction::Function(function);
            lint_function(
                &lowerable_function,
                function.id.as_ref().map(|id| id.name.as_str()),
                &directives,
                function.span,
                outer_bindings,
                config,
                false,
                diagnostics,
            );
        }
        Statement::VariableDeclaration(declaration) => {
            lint_variable_declaration(declaration, outer_bindings, config, diagnostics);
        }
        Statement::ExportDefaultDeclaration(export_default) => match &export_default.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(function)
            | ExportDefaultDeclarationKind::FunctionExpression(function) => {
                let directives = function_directives(function);
                let lowerable_function = LowerableFunction::Function(function);
                lint_function(
                    &lowerable_function,
                    function.id.as_ref().map(|id| id.name.as_str()),
                    &directives,
                    function.span,
                    outer_bindings,
                    config,
                    false,
                    diagnostics,
                );
            }
            ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
                let directives = arrow_directives(arrow);
                let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                lint_function(
                    &lowerable_function,
                    None,
                    &directives,
                    arrow.span,
                    outer_bindings,
                    config,
                    false,
                    diagnostics,
                );
            }
            ExportDefaultDeclarationKind::CallExpression(call) => {
                lint_memo_or_forwardref_call(call, None, outer_bindings, config, diagnostics);
            }
            _ => {}
        },
        Statement::ExportNamedDeclaration(export_named) => {
            if let Some(declaration) = &export_named.declaration {
                match declaration {
                    Declaration::FunctionDeclaration(function) => {
                        let directives = function_directives(function);
                        let lowerable_function = LowerableFunction::Function(function);
                        lint_function(
                            &lowerable_function,
                            function.id.as_ref().map(|id| id.name.as_str()),
                            &directives,
                            function.span,
                            outer_bindings,
                            config,
                            false,
                            diagnostics,
                        );
                    }
                    Declaration::VariableDeclaration(declaration) => {
                        lint_variable_declaration(declaration, outer_bindings, config, diagnostics);
                    }
                    _ => {}
                }
            }
        }
        // Port of getFunctionName context 2 (Program.ts:1189-1195):
        // AssignmentExpression — infer name from LHS identifier.
        Statement::ExpressionStatement(expr_stmt) => {
            if let Expression::AssignmentExpression(assign) = &expr_stmt.expression {
                let assign_name = match &assign.left {
                    AssignmentTarget::AssignmentTargetIdentifier(id) => Some(id.name.as_str()),
                    _ => None,
                };
                match &assign.right {
                    Expression::FunctionExpression(function) => {
                        let directives = function_directives(function);
                        let function_name =
                            function.id.as_ref().map(|id| id.name.as_str()).or(assign_name);
                        let lowerable_function = LowerableFunction::Function(function);
                        lint_function(
                            &lowerable_function,
                            function_name,
                            &directives,
                            function.span,
                            outer_bindings,
                            config,
                            false,
                            diagnostics,
                        );
                    }
                    Expression::ArrowFunctionExpression(arrow) => {
                        let directives = arrow_directives(arrow);
                        let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                        lint_function(
                            &lowerable_function,
                            assign_name,
                            &directives,
                            arrow.span,
                            outer_bindings,
                            config,
                            false,
                            diagnostics,
                        );
                    }
                    Expression::CallExpression(call) => {
                        lint_memo_or_forwardref_call(
                            call,
                            assign_name,
                            outer_bindings,
                            config,
                            diagnostics,
                        );
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn lint_variable_declaration<'a>(
    declaration: &'a VariableDeclaration<'a>,
    outer_bindings: &FxHashMap<String, NonLocalBinding>,
    config: &ReactCompilerConfig,
    diagnostics: &mut Vec<CachedDiagnostic>,
) {
    for declarator in &declaration.declarations {
        let binding_name = match &declarator.id {
            BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
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
                lint_function(
                    &lowerable_function,
                    function_name,
                    &directives,
                    function.span,
                    outer_bindings,
                    config,
                    false,
                    diagnostics,
                );
            }
            Expression::ArrowFunctionExpression(arrow) => {
                let directives = arrow_directives(arrow);
                let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                lint_function(
                    &lowerable_function,
                    binding_name,
                    &directives,
                    arrow.span,
                    outer_bindings,
                    config,
                    false,
                    diagnostics,
                );
            }
            Expression::CallExpression(call) => {
                lint_memo_or_forwardref_call(
                    call,
                    binding_name,
                    outer_bindings,
                    config,
                    diagnostics,
                );
            }
            _ => {}
        }
    }
}

/// Returns `true` if the callee expression is `memo`, `React.memo`,
/// `forwardRef`, or `React.forwardRef`.
fn is_memo_or_forwardref_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(ident) => {
            matches!(ident.name.as_str(), "memo" | "forwardRef")
        }
        Expression::StaticMemberExpression(member) => {
            if let Expression::Identifier(obj) = &member.object {
                obj.name.as_str() == "React"
                    && matches!(member.property.name.as_str(), "memo" | "forwardRef")
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Handles a `CallExpression` that might be `memo(fn)` or `forwardRef(fn)`.
/// Extracts the first argument and lints it with `is_memo_or_forwardref_arg = true`.
fn lint_memo_or_forwardref_call<'a>(
    call: &'a CallExpression<'a>,
    binding_name: Option<&str>,
    outer_bindings: &FxHashMap<String, NonLocalBinding>,
    config: &ReactCompilerConfig,
    diagnostics: &mut Vec<CachedDiagnostic>,
) {
    if !is_memo_or_forwardref_callee(&call.callee) {
        return;
    }
    let Some(first_arg) = call.arguments.first() else {
        return;
    };
    let Some(first_arg_expr) = first_arg.as_expression() else {
        return;
    };
    match first_arg_expr {
        Expression::FunctionExpression(function) => {
            let directives = function_directives(function);
            let function_name = function.id.as_ref().map(|id| id.name.as_str()).or(binding_name);
            let lowerable_function = LowerableFunction::Function(function);
            lint_function(
                &lowerable_function,
                function_name,
                &directives,
                function.span,
                outer_bindings,
                config,
                true,
                diagnostics,
            );
        }
        Expression::ArrowFunctionExpression(arrow) => {
            let directives = arrow_directives(arrow);
            let lowerable_function = LowerableFunction::ArrowFunction(arrow);
            lint_function(
                &lowerable_function,
                binding_name,
                &directives,
                arrow.span,
                outer_bindings,
                config,
                true,
                diagnostics,
            );
        }
        _ => {}
    }
}

fn lint_function(
    function: &LowerableFunction<'_>,
    name: Option<&str>,
    directives: &[String],
    fallback_span: Span,
    outer_bindings: &FxHashMap<String, NonLocalBinding>,
    config: &ReactCompilerConfig,
    is_memo_or_forwardref_arg: bool,
    diagnostics: &mut Vec<CachedDiagnostic>,
) {
    let Some(fn_type) = should_compile_function(
        function,
        name,
        directives,
        config.compilation_mode.into(),
        is_memo_or_forwardref_arg,
        false, // linter does not support dynamic gating
    ) else {
        return;
    };

    if !config.ignore_use_no_forget
        && find_directive_disabling_memoization(
            directives,
            config.custom_opt_out_directives.as_deref(),
        )
        .is_some()
    {
        return;
    }

    let mut env_config = config.environment.to_environment_config();
    if config.enable_reanimated_check.0 {
        env_config.enable_custom_type_definition_for_reanimated = true;
    }
    let environment = match Environment::new(fn_type, CompilerOutputMode::Lint, env_config) {
        Ok(env) => env,
        Err(error) => {
            collect_compiler_error(&error, fallback_span, diagnostics);
            return;
        }
    };

    let mut hir_function = match lower(&environment, fn_type, function, outer_bindings.clone()) {
        Ok(hir_function) => hir_function,
        Err(error) => {
            collect_compiler_error(&error, fallback_span, diagnostics);
            return;
        }
    };

    match run_pipeline(&mut hir_function, &environment) {
        Ok(output) => {
            if let Some(recorded) = output.recorded_errors {
                collect_compiler_error(&recorded, fallback_span, diagnostics);
            }
        }
        Err(error) => {
            collect_compiler_error(&error, fallback_span, diagnostics);
        }
    }

    for diagnostic in hir_function.env.take_diagnostics() {
        collect_compiler_error(&diagnostic, fallback_span, diagnostics);
    }
}

/// Collects compiler errors into a `Vec<CachedDiagnostic>` instead of
/// reporting them directly via `LintContext`.
fn collect_compiler_error(
    error: &CompilerError,
    fallback_span: Span,
    diagnostics: &mut Vec<CachedDiagnostic>,
) {
    for entry in &error.details {
        let severity = entry.severity();
        if matches!(severity, ErrorSeverity::Hint | ErrorSeverity::Off) {
            continue;
        }
        let span = compiler_error_entry_span(entry).unwrap_or(fallback_span);
        let message = entry.to_string();
        let category = entry.category();
        diagnostics.push(CachedDiagnostic { category, severity, message, span });
    }
}

pub(super) fn function_directives(function: &Function<'_>) -> Vec<String> {
    function.body.as_ref().map_or_else(Vec::new, |body| {
        body.directives.iter().map(|directive| directive.directive.to_string()).collect()
    })
}

pub(super) fn arrow_directives(function: &ArrowFunctionExpression<'_>) -> Vec<String> {
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
