use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_nested_components_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Component definition inside another component")
        .with_help("Move the component definition outside of the parent component. Nested component definitions cause the component to be recreated on every render, destroying its state.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNestedComponents;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents defining React components inside other React components.
    ///
    /// ### Why is this bad?
    ///
    /// When a component is defined inside another component's render, a new
    /// component type is created on every render. This causes React to unmount
    /// and remount the nested component, destroying all its state. It also
    /// prevents React from optimizing re-renders.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function ParentComponent() {
    ///     function ChildComponent() {
    ///         return <div />;
    ///     }
    ///     return <ChildComponent />;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function ChildComponent() {
    ///     return <div />;
    /// }
    /// function ParentComponent() {
    ///     return <ChildComponent />;
    /// }
    /// ```
    NoNestedComponents,
    oxc,
    correctness,
    none
);

impl Rule for NoNestedComponents {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Look for function declarations or arrow functions that return JSX
        let (fn_name_span, has_jsx) = match node.kind() {
            AstKind::Function(func) => {
                let Some(id) = &func.id else {
                    return;
                };
                // Check if name starts with uppercase (React component convention)
                if !id.name.starts_with(|c: char| c.is_ascii_uppercase()) {
                    return;
                }
                (id.span, true)
            }
            AstKind::VariableDeclarator(decl) => {
                // const MyComponent = () => <div />
                // const MyComponent = function() { return <div /> }
                let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &decl.id else {
                    return;
                };
                if !id.name.starts_with(|c: char| c.is_ascii_uppercase()) {
                    return;
                }
                let Some(init) = &decl.init else {
                    return;
                };
                let is_fn = matches!(
                    init,
                    oxc_ast::ast::Expression::ArrowFunctionExpression(_)
                        | oxc_ast::ast::Expression::FunctionExpression(_)
                );
                if !is_fn {
                    return;
                }
                (id.span, true)
            }
            _ => return,
        };

        if !has_jsx {
            return;
        }

        // Check if we're inside another function component
        for ancestor_id in ctx.nodes().ancestor_ids(node.id()).skip(1) {
            let ancestor = ctx.nodes().get_node(ancestor_id);
            match ancestor.kind() {
                AstKind::Function(func) => {
                    if let Some(id) = &func.id
                        && id.name.starts_with(|c: char| c.is_ascii_uppercase())
                    {
                        ctx.diagnostic(no_nested_components_diagnostic(fn_name_span));
                        return;
                    }
                }
                AstKind::VariableDeclarator(decl) => {
                    if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &decl.id
                        && id.name.starts_with(|c: char| c.is_ascii_uppercase())
                        && decl.init.as_ref().is_some_and(|init| {
                            matches!(
                                init,
                                oxc_ast::ast::Expression::ArrowFunctionExpression(_)
                                    | oxc_ast::ast::Expression::FunctionExpression(_)
                            )
                        })
                    {
                        ctx.diagnostic(no_nested_components_diagnostic(fn_name_span));
                        return;
                    }
                }
                AstKind::Program(_) => return,
                _ => {}
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Top-level components
        "function ChildComponent() { return <div />; }\nfunction ParentComponent() { return <ChildComponent />; }",
        // Non-component functions (lowercase)
        "function ParentComponent() { function helper() { return 1; } return <div />; }",
    ];

    let fail = vec![
        // Nested function component
        "function ParentComponent() { function ChildComponent() { return <div />; } return <ChildComponent />; }",
        // Nested arrow component
        "function ParentComponent() { const ChildComponent = () => <div />; return <ChildComponent />; }",
    ];

    Tester::new(NoNestedComponents::NAME, NoNestedComponents::PLUGIN, pass, fail)
        .change_rule_path_extension("tsx")
        .test_and_snapshot();
}
