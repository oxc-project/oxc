use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-caller): Disallow the use of arguments.caller or arguments.callee")]
#[diagnostic(
    severity(warning),
    help(
        "'caller', 'callee', and 'arguments' properties may not be accessed on strict mode functions or the arguments objects for calls to them"
    )
)]
struct NoCallerDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoCaller;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of arguments.caller or arguments.callee
    ///
    /// ### Why is this bad?
    ///
    /// The use of arguments.caller and arguments.callee make several code optimizations impossible.
    /// They have been deprecated in future versions of JavaScript and their use is forbidden in ECMAScript 5 while in strict mode.
    ///
    /// ### Example
    /// ```javascript
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
    NoCaller,
    correctness
);

impl Rule for NoCaller {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MemberExpression(member_expr) = node.kind() else {return};
        if let MemberExpression::StaticMemberExpression(expr) = member_expr {
            if let Some(reference) = expr.object.get_identifier_reference() {
                if reference.name != "arguments" {
                    return;
                }

                if expr.property.name == "callee" || expr.property.name == "caller" {
                    ctx.diagnostic(NoCallerDiagnostic(expr.property.span));
                }
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

    Tester::new(NoCaller::NAME, pass, fail).test_and_snapshot();
}
