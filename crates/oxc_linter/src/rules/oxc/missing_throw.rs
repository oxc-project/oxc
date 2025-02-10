use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn missing_throw_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing throw")
        .with_help("The `throw` keyword seems to be missing in front of this 'new' expression")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct MissingThrow;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks whether the `throw` keyword is missing in front of a `new` expression.
    ///
    /// ### Why is this bad?
    ///
    /// The `throw` keyword is required in front of a `new` expression to throw an error. Omitting it is usually a mistake.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function foo() { throw Error() }
    /// const foo = () => { new Error() }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function foo() { throw new Error() }
    /// const foo = () => { throw new Error() }
    /// ```
    MissingThrow,
    oxc,
    correctness,
    suggestion
);

impl Rule for MissingThrow {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };
        if new_expr.callee.is_specific_id("Error") && Self::has_missing_throw(node, ctx) {
            ctx.diagnostic_with_suggestion(missing_throw_diagnostic(new_expr.span), |fixer| {
                fixer.insert_text_before(node, "throw ")
            });
        }
    }
}

impl MissingThrow {
    fn has_missing_throw<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        let mut node_ancestors = ctx.nodes().ancestor_ids(node.id()).skip(1);

        let Some(node_id) = node_ancestors.next() else {
            return false;
        };

        if matches!(ctx.nodes().kind(node_id), AstKind::ExpressionStatement(_)) {
            for node_id in node_ancestors {
                match ctx.nodes().kind(node_id) {
                    // ignore arrow `const foo = () => new Error()`
                    AstKind::ArrowFunctionExpression(arrow_expr) if arrow_expr.expression => {
                        return false;
                    }
                    AstKind::ArrayExpression(_) | AstKind::Function(_) => break,
                    _ => {}
                }
            }
            return true;
        }

        false
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Note: lone `Error()` should be caught by no-effect-call
    let pass = vec![
        ("function foo() { throw new Error() }", None),
        ("const foo = () => new Error()", None),
        ("[new Error()]", None),
    ];

    let fail =
        vec![("function foo() { new Error() }", None), ("const foo = () => { new Error() }", None)];

    let fix = vec![
        ("function foo() { new Error() }", "function foo() { throw new Error() }"),
        ("const foo = () => { new Error() }", "const foo = () => { throw new Error() }"),
    ];

    Tester::new(MissingThrow::NAME, MissingThrow::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
