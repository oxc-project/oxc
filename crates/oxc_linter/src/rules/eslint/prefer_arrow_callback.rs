use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_arrow_callback_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected function expression.")
        .with_help("Use an arrow function instead of a function expression.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferArrowCallback;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires using arrow functions for callbacks.
    ///
    /// ### Why is this bad?
    ///
    /// Arrow functions can be a more concise syntax for function expressions.
    /// They also don't bind their own `this`, making them more predictable
    /// when used as callbacks.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// foo(function(a) { return a; });
    /// foo(function() { return this.a; }.bind(this));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// foo((a) => a);
    /// foo(() => this.a);
    /// ```
    PreferArrowCallback,
    eslint,
    style,
    pending
);

impl Rule for PreferArrowCallback {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Function(func) = node.kind() else {
            return;
        };

        // Only target function expressions, not declarations
        if func.is_declaration() {
            return;
        }

        // Skip named function expressions (they may use their own name recursively)
        if func.id.is_some() {
            return;
        }

        // Skip generator functions
        if func.generator {
            return;
        }

        // Check if used as a callback (argument in a call expression)
        let parent = ctx.nodes().parent_node(node.id());
        let is_callback =
            matches!(parent.kind(), AstKind::CallExpression(_) | AstKind::NewExpression(_));

        if !is_callback {
            return;
        }

        ctx.diagnostic(prefer_arrow_callback_diagnostic(Span::new(
            func.span.start,
            func.params.span.start,
        )));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo((a) => a);",
        "foo(function bar() { return bar; });", // named, may be recursive
        "foo(function*() { yield 1; });",       // generator
        "var x = function() { return 1; };",    // not a callback
    ];

    let fail = vec![
        "foo(function(a) { return a; });",
        "foo(function() { return 1; });",
        "[1, 2].map(function(x) { return x * 2; });",
        "setTimeout(function() { console.log('hi'); }, 100);",
    ];

    Tester::new(PreferArrowCallback::NAME, PreferArrowCallback::PLUGIN, pass, fail)
        .test_and_snapshot();
}
