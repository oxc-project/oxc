use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_this_in_static_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `this` in a static method.")
        .with_help("In static methods, `this` refers to the class constructor, not an instance. Use the class name directly instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoThisInStatic;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of `this` in static class methods.
    ///
    /// ### Why is this bad?
    ///
    /// In a static method, `this` refers to the class constructor itself,
    /// not to an instance. This can be confusing, especially for developers
    /// coming from other languages. Using the class name directly makes the
    /// intent clearer.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class Foo {
    ///   static bar() {
    ///     return this.baz();
    ///   }
    ///   static baz() { return 1; }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class Foo {
    ///   static bar() {
    ///     return Foo.baz();
    ///   }
    ///   static baz() { return 1; }
    /// }
    /// ```
    NoThisInStatic,
    typescript,
    suspicious,
    pending
);

impl Rule for NoThisInStatic {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ThisExpression(this_expr) = node.kind() else {
            return;
        };

        // Walk up ancestors to find if we're directly in a static method.
        // Count function boundaries: the first Function is the method's own body,
        // a second Function means a nested function with its own `this`.
        let mut function_count = 0u32;
        for ancestor in ctx.nodes().ancestors(node.id()) {
            match ancestor.kind() {
                AstKind::MethodDefinition(method) => {
                    if method.r#static && function_count <= 1 {
                        ctx.diagnostic(no_this_in_static_diagnostic(this_expr.span));
                    }
                    return;
                }
                AstKind::PropertyDefinition(prop) => {
                    if prop.r#static && function_count == 0 {
                        ctx.diagnostic(no_this_in_static_diagnostic(this_expr.span));
                    }
                    return;
                }
                AstKind::Function(_) => {
                    function_count += 1;
                    if function_count > 1 {
                        return; // Nested function has its own `this`
                    }
                }
                AstKind::ArrowFunctionExpression(_) => {
                    // Arrow functions inherit `this`, keep looking
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class Foo { bar() { return this.baz; } }",
        "class Foo { static bar() { return Foo.baz; } }",
        "class Foo { static bar() { return 1; } }",
    ];

    let fail = vec![
        "class Foo { static bar() { return this.baz; } }",
        "class Foo { static bar() { this.baz(); } }",
        "class Foo { static bar = () => { return this.baz; }; }",
    ];

    Tester::new(NoThisInStatic::NAME, NoThisInStatic::PLUGIN, pass, fail).test_and_snapshot();
}
