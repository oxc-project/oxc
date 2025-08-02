use oxc_ast::{
    AstKind,
    ast::{Declaration, ExportDefaultDeclaration, ExportNamedDeclaration, Expression, Program},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn export_all_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This rule can't verify that `export *` only exports components.")
        .with_label(span)
}

fn named_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Fast refresh only works when a file only exports components. Use a new file to share constants or functions between components.")
        .with_label(span)
}

fn anonymous_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Fast refresh can't handle anonymous components. Add a name to your export.",
    )
    .with_label(span)
}

fn local_components_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Fast refresh only works when a file only exports components. Move your component(s) to a separate file.")
        .with_label(span)
}

fn no_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Fast refresh only works when a file has exports. Move your component(s) to a separate file.")
        .with_label(span)
}

fn react_context_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Fast refresh only works when a file only exports components. Move your React context(s) to a separate file.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct OnlyExportComponents;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates that your components can safely be updated with Fast Refresh.
    ///
    /// ### Why is this bad?
    ///
    /// "Fast Refresh", also known as "hot reloading", is a feature in many modern bundlers.
    /// If you update some React component(s) on disk, then the bundler will know to update
    /// only the impacted parts of your page -- without a full page reload.
    ///
    /// This rule enforces that your components are structured in a way that integrations
    /// such as react-refresh expect.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// export const foo = () => {};
    /// export const Bar = () => <></>;
    /// ```
    ///
    /// ```jsx
    /// export default function () {}
    /// export default compose()(MainComponent)
    /// ```
    ///
    /// ```jsx
    /// export * from "./foo";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// export default function Foo() {
    ///   return <></>;
    /// }
    /// ```
    ///
    /// ```jsx
    /// const foo = () => {};
    /// export const Bar = () => <></>;
    /// ```
    OnlyExportComponents,
    react,
    correctness
);

impl Rule for OnlyExportComponents {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Program(program) = node.kind() else {
            return;
        };

        // Check if this is a test file - skip if so
        let filename = ctx.file_path().file_name().unwrap_or_default().to_string_lossy();
        if filename.contains(".test.")
            || filename.contains(".spec.")
            || filename.contains(".cy.")
            || filename.contains(".stories.")
        {
            return;
        }

        self.check_program(program, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        let source_type = ctx.source_type();
        // Only run on JSX/TSX files (matching the original plugin's behavior)
        source_type.is_jsx()
    }
}

impl OnlyExportComponents {
    fn check_program<'a>(&self, program: &Program<'a>, ctx: &LintContext<'a>) {
        let mut has_exports = false;
        let mut has_react_export = false;
        let mut local_components = Vec::new();
        let mut non_component_exports = Vec::new();
        let mut react_context_exports = Vec::new();

        // Check if React is imported - simplified for now
        // TODO: Add proper React import detection if needed for checkJS option

        // Analyze all statements
        for stmt in &program.body {
            match stmt {
                // Handle export all declarations
                oxc_ast::ast::Statement::ExportAllDeclaration(export_all) => {
                    if export_all.export_kind.is_type() {
                        continue;
                    }
                    has_exports = true;
                    ctx.diagnostic(export_all_diagnostic(export_all.span));
                }

                // Handle export default declarations
                oxc_ast::ast::Statement::ExportDefaultDeclaration(export_default) => {
                    has_exports = true;
                    self.handle_export_default_declaration(
                        export_default,
                        &mut has_react_export,
                        ctx,
                    );
                }

                // Handle export named declarations
                oxc_ast::ast::Statement::ExportNamedDeclaration(export_named) => {
                    if export_named.export_kind.is_type() {
                        continue;
                    }
                    has_exports = true;
                    self.handle_export_named_declaration(
                        export_named,
                        &mut has_react_export,
                        &mut non_component_exports,
                        &mut react_context_exports,
                        ctx,
                    );
                }

                // Check for local components (not exported)
                oxc_ast::ast::Statement::VariableDeclaration(var_decl) => {
                    self.check_variable_declaration_for_local_components(
                        var_decl,
                        &mut local_components,
                    );
                }

                oxc_ast::ast::Statement::FunctionDeclaration(func_decl) => {
                    if let Some(id) = &func_decl.id {
                        if self.is_react_component_name(&id.name) {
                            local_components.push(id.span);
                        }
                    }
                }

                _ => {}
            }
        }

        // Apply the main logic based on what was found
        if has_exports {
            if has_react_export {
                // If we have React exports, report any non-component exports
                for span in non_component_exports {
                    ctx.diagnostic(named_export_diagnostic(span));
                }
                for span in react_context_exports {
                    ctx.diagnostic(react_context_diagnostic(span));
                }
            } else if !local_components.is_empty() {
                // We have exports but no React exports, and local components exist
                for span in local_components {
                    ctx.diagnostic(local_components_diagnostic(span));
                }
            }
        } else if !local_components.is_empty() {
            // No exports but local components exist
            for span in local_components {
                ctx.diagnostic(no_export_diagnostic(span));
            }
        }
    }

    fn handle_export_default_declaration<'a>(
        &self,
        export_default: &ExportDefaultDeclaration<'a>,
        has_react_export: &mut bool,
        ctx: &LintContext<'a>,
    ) {
        match &export_default.declaration {
            oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                if func.id.is_some() {
                    *has_react_export = true;
                } else {
                    ctx.diagnostic(anonymous_export_diagnostic(func.span));
                }
            }
            oxc_ast::ast::ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                if class.id.is_some() {
                    *has_react_export = true;
                } else {
                    ctx.diagnostic(anonymous_export_diagnostic(class.span));
                }
            }
            oxc_ast::ast::ExportDefaultDeclarationKind::ArrowFunctionExpression(expr) => {
                ctx.diagnostic(anonymous_export_diagnostic(expr.span));
            }
            oxc_ast::ast::ExportDefaultDeclarationKind::Identifier(ident) => {
                if self.is_react_component_name(&ident.name) {
                    *has_react_export = true;
                }
            }
            _ => {
                // For other expressions, we assume they might be components
                // This is a simplification - the original rule has more complex logic
                *has_react_export = true;
            }
        }
    }

    fn handle_export_named_declaration<'a>(
        &self,
        export_named: &ExportNamedDeclaration<'a>,
        has_react_export: &mut bool,
        non_component_exports: &mut Vec<Span>,
        react_context_exports: &mut Vec<Span>,
        _ctx: &LintContext<'a>,
    ) {
        // Handle declaration exports (export const/function/class)
        if let Some(declaration) = &export_named.declaration {
            self.handle_export_declaration(
                declaration,
                has_react_export,
                non_component_exports,
                react_context_exports,
            );
        }

        // Handle named exports (export { ... })
        for specifier in &export_named.specifiers {
            let exported_name = self.extract_module_export_name(&specifier.exported);

            if self.is_react_component_name(exported_name) {
                *has_react_export = true;
            } else {
                non_component_exports.push(specifier.local.span());
            }
        }
    }

    fn handle_export_declaration<'a>(
        &self,
        declaration: &Declaration<'a>,
        has_react_export: &mut bool,
        non_component_exports: &mut Vec<Span>,
        react_context_exports: &mut Vec<Span>,
    ) {
        match declaration {
            Declaration::VariableDeclaration(var_decl) => {
                for declarator in &var_decl.declarations {
                    if let Some(id) = declarator.id.get_binding_identifier() {
                        let name = &id.name;

                        if self.is_react_component_name(name)
                            && self.can_be_react_function_component(&declarator.init)
                        {
                            *has_react_export = true;
                        } else if self.is_react_context_creation(&declarator.init) {
                            react_context_exports.push(id.span);
                        } else {
                            non_component_exports.push(id.span);
                        }
                    }
                }
            }
            Declaration::FunctionDeclaration(func_decl) => {
                if let Some(id) = &func_decl.id {
                    if self.is_react_component_name(&id.name) {
                        *has_react_export = true;
                    } else {
                        non_component_exports.push(id.span);
                    }
                }
            }
            Declaration::ClassDeclaration(class_decl) => {
                if let Some(id) = &class_decl.id {
                    if self.is_react_component_name(&id.name) {
                        *has_react_export = true;
                    } else {
                        non_component_exports.push(id.span);
                    }
                }
            }
            _ => {}
        }
    }

    fn check_variable_declaration_for_local_components<'a>(
        &self,
        var_decl: &oxc_ast::ast::VariableDeclaration<'a>,
        local_components: &mut Vec<Span>,
    ) {
        for declarator in &var_decl.declarations {
            if let Some(id) = declarator.id.get_binding_identifier() {
                if self.is_react_component_name(&id.name)
                    && self.can_be_react_function_component(&declarator.init)
                {
                    local_components.push(id.span);
                }
            }
        }
    }

    fn extract_module_export_name<'a>(
        &self,
        export_name: &'a oxc_ast::ast::ModuleExportName<'a>,
    ) -> &'a str {
        match export_name {
            oxc_ast::ast::ModuleExportName::IdentifierName(ident) => &ident.name,
            oxc_ast::ast::ModuleExportName::IdentifierReference(ident) => &ident.name,
            oxc_ast::ast::ModuleExportName::StringLiteral(lit) => &lit.value,
        }
    }

    fn is_react_component_name(&self, name: &str) -> bool {
        // React component names must start with uppercase letter
        name.chars().next().map_or(false, |c| c.is_ascii_uppercase())
            && name.chars().all(|c| c.is_ascii_alphanumeric())
    }

    fn can_be_react_function_component(&self, init: &Option<Expression>) -> bool {
        match init {
            Some(Expression::ArrowFunctionExpression(_)) => true,
            Some(Expression::CallExpression(call)) => {
                // Check for HOCs like memo, forwardRef
                if let Expression::Identifier(callee) = &call.callee {
                    matches!(callee.name.as_str(), "memo" | "forwardRef")
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn is_react_context_creation(&self, init: &Option<Expression>) -> bool {
        match init {
            Some(Expression::CallExpression(call)) => {
                match &call.callee {
                    // createContext()
                    Expression::Identifier(ident) => ident.name == "createContext",
                    // React.createContext()
                    Expression::StaticMemberExpression(member) => {
                        member.property.name == "createContext"
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"export default function Foo() { return <></> }",
        r"const foo = () => {}; export const Bar = () => <></>;",
        r"import { App } from './App'; createRoot(document.getElementById('root')).render(<App />);",
        r"export default function Component() { return <div />; }",
        r"export { Component as default };",
        r"export const Component = () => <div />;",
        r"export function Component() { return <div />; }",
    ];

    let fail = vec![
        r"export const foo = () => {}; export const Bar = () => <></>;",
        r"export default function () {}",
        r"export * from './foo';",
        r"const Tab = () => {}; export const tabs = [<Tab />, <Tab />];",
        r"const App = () => {}; createRoot(document.getElementById('root')).render(<App />);",
        r"export const CONSTANT = 3; export const Foo = () => <></>;",
        r"export const createContext = () => {}; export const Foo = () => <></>;",
    ];

    Tester::new(OnlyExportComponents::NAME, OnlyExportComponents::PLUGIN, pass, fail)
        .test_and_snapshot();
}
