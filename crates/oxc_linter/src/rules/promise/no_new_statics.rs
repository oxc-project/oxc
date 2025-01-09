use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::PROMISE_STATIC_METHODS, AstNode};

fn static_promise_diagnostic(static_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not use `new` on `Promise.{static_name}`"))
        .with_help(format!(
            "`Promise.{static_name}` is not a constructor. Call it as a function instead."
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNewStatics;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows calling new on static `Promise` methods.
    ///
    /// ### Why is this bad?
    ///
    /// Calling a static `Promise` method with `new` is invalid and will result
    /// in a `TypeError` at runtime.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const x = new Promise.resolve(value);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const x = Promise.resolve(value);
    /// ```
    NoNewStatics,
    promise,
    correctness,
    fix
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

        if PROMISE_STATIC_METHODS.contains(prop_name) {
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
    Tester::new(NoNewStatics::NAME, NoNewStatics::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
