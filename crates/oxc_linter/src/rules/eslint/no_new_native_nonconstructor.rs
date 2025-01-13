use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_new_native_nonconstructor_diagnostic(fn_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{fn_name}` cannot be called as a constructor.")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNewNativeNonconstructor;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `new` operators with global non-constructor functions (`Symbol`, `BigInt`)
    ///
    /// ### Why is this bad?
    ///
    /// Both `new Symbol` and `new BigInt` throw a type error because they are
    /// functions and not classes.  It is easy to make this mistake by assuming
    /// the uppercase letters indicate classes.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // throws a TypeError
    /// let foo = new Symbol("foo");
    ///
    /// // throws a TypeError
    /// let result = new BigInt(9007199254740991);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// let foo = Symbol("foo");
    ///
    /// let result = BigInt(9007199254740991);
    /// ```
    NoNewNativeNonconstructor,
    eslint,
    correctness,
);

impl Rule for NoNewNativeNonconstructor {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(expr) = node.kind() else {
            return;
        };
        let Expression::Identifier(ident) = &expr.callee else {
            return;
        };
        if matches!(ident.name.as_str(), "Symbol" | "BigInt")
            && ctx.semantic().is_reference_to_global_variable(ident)
        {
            let start = expr.span.start;
            let end = start + 3;
            ctx.diagnostic(no_new_native_nonconstructor_diagnostic(
                ident.name.as_str(),
                Span::new(start, end),
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var foo = Symbol('foo');",
        "function bar(Symbol) { var baz = new Symbol('baz');}",
        "function Symbol() {} new Symbol();",
        "new foo(Symbol);",
        "new foo(bar, Symbol);",
        "var foo = BigInt(9007199254740991);",
        "function bar(BigInt) { var baz = new BigInt(9007199254740991);}",
        "function BigInt() {} new BigInt();",
        "new foo(BigInt);",
        "new foo(bar, BigInt);",
    ];

    let fail = vec![
        "var foo = new Symbol('foo');",
        "function bar() { return function Symbol() {}; } var baz = new Symbol('baz');",
        "var foo = new BigInt(9007199254740991);",
        "function bar() { return function BigInt() {}; } var baz = new BigInt(9007199254740991);",
    ];

    Tester::new(NoNewNativeNonconstructor::NAME, NoNewNativeNonconstructor::PLUGIN, pass, fail)
        .test_and_snapshot();
}
