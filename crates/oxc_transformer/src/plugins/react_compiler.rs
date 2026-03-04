use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_react_compiler::{
    compiler_error::{CompilerError, CompilerErrorEntry, SourceLocation},
    entrypoint::{
        options::CompilationMode, pipeline::run_pipeline, program::should_compile_function,
    },
    hir::{
        NonLocalBinding,
        build_hir::{LowerableFunction, collect_import_bindings, lower},
        environment::{CompilerOutputMode, Environment, EnvironmentConfig},
    },
};
use oxc_span::Span;
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
/// This currently runs React Compiler analysis and validation in the transformer
/// pipeline and surfaces diagnostics through transformer errors.
pub struct ReactCompiler {
    options: ReactCompilerOptions,
    outer_bindings: FxHashMap<String, NonLocalBinding>,
}

impl ReactCompiler {
    pub fn new(options: ReactCompilerOptions) -> Self {
        Self { options, outer_bindings: FxHashMap::default() }
    }

    pub fn enter_program<'a>(&mut self, program: &Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.options.enabled {
            return;
        }

        self.outer_bindings = collect_import_bindings(&program.body);
        for statement in &program.body {
            self.compile_statement(statement, ctx);
        }
    }

    fn compile_statement<'a>(&self, statement: &Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match statement {
            Statement::FunctionDeclaration(function) => {
                let directives = function_directives(function);
                let lowerable_function = LowerableFunction::Function(function);
                self.compile_function(
                    &lowerable_function,
                    function.id.as_ref().map(|id| id.name.as_str()),
                    &directives,
                    function.span,
                    ctx,
                );
            }
            Statement::VariableDeclaration(declaration) => {
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
                            self.compile_function(
                                &lowerable_function,
                                function_name,
                                &directives,
                                function.span,
                                ctx,
                            );
                        }
                        Expression::ArrowFunctionExpression(arrow) => {
                            let directives = arrow_directives(arrow);
                            let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                            self.compile_function(
                                &lowerable_function,
                                binding_name,
                                &directives,
                                arrow.span,
                                ctx,
                            );
                        }
                        _ => {}
                    }
                }
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
                            ctx,
                        );
                    }
                    ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
                        let directives = arrow_directives(arrow);
                        let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                        self.compile_function(
                            &lowerable_function,
                            None,
                            &directives,
                            arrow.span,
                            ctx,
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn compile_function(
        &self,
        function: &LowerableFunction<'_>,
        name: Option<&str>,
        directives: &[String],
        fallback_span: Span,
        ctx: &mut TraverseCtx<'_>,
    ) {
        let Some(fn_type) = should_compile_function(
            name,
            directives,
            parse_compilation_mode(self.options.compilation_mode.as_deref()),
            false,
        ) else {
            return;
        };

        let environment =
            Environment::new(fn_type, CompilerOutputMode::Client, EnvironmentConfig::default());

        let mut hir_function =
            match lower(&environment, fn_type, function, self.outer_bindings.clone()) {
                Ok(hir_function) => hir_function,
                Err(error) => {
                    Self::report_compiler_error(&error, fallback_span, ctx);
                    return;
                }
            };

        if let Err(error) = run_pipeline(&mut hir_function, &environment) {
            Self::report_compiler_error(&error, fallback_span, ctx);
        }

        for diagnostic in hir_function.env.take_diagnostics() {
            Self::report_compiler_error(&diagnostic, fallback_span, ctx);
        }
    }

    fn report_compiler_error(
        error: &CompilerError,
        fallback_span: Span,
        ctx: &mut TraverseCtx<'_>,
    ) {
        for entry in &error.details {
            let span = compiler_error_entry_span(entry).unwrap_or(fallback_span);
            ctx.state
                .error(OxcDiagnostic::warn(format!("React Compiler: {entry}")).with_label(span));
        }
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
