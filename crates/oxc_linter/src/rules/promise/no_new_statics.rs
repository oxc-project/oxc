use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn static_promise_diagnostic(x0: &str, span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Disallow calling `new` on a `Promise.{x0}`")).with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoNewStatics;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow calling new on a Promise static method.
    ///
    /// ### Why is this bad?
    ///
    /// Calling a Promise static method with new is invalid, resulting in a TypeError at runtime.
    ///
    /// ### Example
    /// ```javascript
    /// new Promise.resolve(value);
    /// ```
    NoNewStatics,
    correctness
);

impl Rule for NoNewStatics {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = &new_expr.callee.get_member_expr() else {
            return;
        };

        let Expression::Identifier(ident) = &member_expr.object() else {
            return;
        };

        if ident.name != "Promise" || !ctx.semantic().is_reference_to_global_variable(ident) {
            return;
        }

        let Some(prop_name) = member_expr.static_property_name() else {
            return;
        };

        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
        if matches!(
            prop_name,
            "resolve" | "reject" | "all" | "allSettled" | "race" | "any" | "withResolvers"
        ) {
            ctx.diagnostic_with_fix(
                static_promise_diagnostic(
                    prop_name,
                    Span::new(new_expr.span.start, ident.span.start - 1),
                ),
                |fixer| fixer.delete_range(Span::new(new_expr.span.start, ident.span.start)),
            );
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Promise.resolve()",
        "Promise.reject()",
        "Promise.all()",
        "Promise.race()",
        "new Promise(function (resolve, reject) {})",
        "new SomeClass()",
        "SomeClass.resolve()",
        "new SomeClass.resolve()",
    ];

    let fail = vec![
        "new Promise.resolve()",
        "new Promise.reject()",
        "new Promise.all()",
        "new Promise.allSettled()",
        "new Promise.any()",
        "new Promise.race()",
        "function foo() {
			  var a = getA()
			  return new Promise.resolve(a)
			}",
    ];

    let fix = vec![
        ("new Promise.resolve()", "Promise.resolve()", None),
        ("new Promise.reject()", "Promise.reject()", None),
        ("new Promise.all()", "Promise.all()", None),
        ("new Promise.allSettled()", "Promise.allSettled()", None),
        ("new Promise.any()", "Promise.any()", None),
        ("new Promise.race()", "Promise.race()", None),
    ];
    Tester::new(NoNewStatics::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
