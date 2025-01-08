use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_caller_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not use `arguments.{method_name}`"))
        .with_help("'caller', 'callee', and 'arguments' properties may not be accessed on strict mode functions or the arguments objects for calls to them.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoCaller;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of `arguments.caller` or `arguments.callee`.
    ///
    /// ### Why is this bad?
    ///
    /// The use of `arguments.caller` and `arguments.callee` make several code
    /// optimizations impossible.  They have been deprecated in future versions
    /// of JavaScript and their use is forbidden in ECMAScript 5 while in strict
    /// mode.
    ///
    /// ```js
    /// function foo() {
    /// var callee = arguments.callee;
    /// }
    /// ```
    ///
    /// This rule is aimed at discouraging the use of deprecated and sub-optimal
    /// code by disallowing the use of `arguments.caller` and `arguments.callee`. As
    /// such, it will warn when `arguments.caller` and `arguments.callee` are used.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function foo(n) {
    ///     if (n <= 0) {
    ///         return;
    ///     }
    ///
    ///     arguments.callee(n - 1);
    /// }
    ///
    /// [1,2,3,4,5].map(function(n) {
    ///    return !(n > 1) ? 1 : arguments.callee(n - 1) * n;
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function foo(n) {
    ///     if (n <= 0) {
    ///         return;
    ///     }
    ///
    ///     foo(n - 1);
    /// }
    ///
    /// [1,2,3,4,5].map(function factorial(n) {
    ///     return !(n > 1) ? 1 : factorial(n - 1) * n;
    /// });
    /// ```
    NoCaller,
    eslint,
    correctness
);

impl Rule for NoCaller {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::MemberExpression(MemberExpression::StaticMemberExpression(expr)) =
            node.kind()
        {
            if (expr.property.name == "callee" || expr.property.name == "caller")
                && expr.object.is_specific_id("arguments")
            {
                ctx.diagnostic(no_caller_diagnostic(expr.property.span, &expr.property.name));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var x = arguments.length", None),
        ("var x = arguments", None),
        ("var x = arguments[0]", None),
        ("var x = arguments[caller]", None),
    ];

    let fail = vec![("var x = arguments.callee", None), ("var x = arguments.caller", None)];

    Tester::new(NoCaller::NAME, NoCaller::PLUGIN, pass, fail).test_and_snapshot();
}
