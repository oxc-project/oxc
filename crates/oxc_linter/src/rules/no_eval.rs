use oxc_ast::{AstKind, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-eval): Disallow the use of `eval()`")]
#[diagnostic(severity(error))]
struct NoEvalDiagnostic(&'static str, #[label("{0} can be harmful.")] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoEval {
    // TODO: supports allowIndirect option: https://eslint.org/docs/latest/rules/no-eval#options
    // allow_indirect: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow the use of `eval()`
    ///
    /// ### Why is this bad?
    /// JavaScript’s eval() function is potentially dangerous and is often misused. Using eval() on untrusted code can open a program up to several different injection attacks. The use of eval() in most contexts can be substituted for a better, alternative approach to a problem.
    ///
    /// ### Example
    /// ```javascript
    /// var obj = { x: "foo" },
    /// key = "x",
    /// value = eval("obj." + key);
    /// ```
    NoEval,
    nursery,
);

impl Rule for NoEval {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.get().kind() {
            AstKind::CallExpression(call_expr) => {
                match &call_expr.callee {
                    oxc_ast::ast::Expression::Identifier(id) if id.name == "eval" => {
                        ctx.diagnostic(NoEvalDiagnostic("eval", id.span))
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {

    use crate::tester::Tester;

    let pass = vec![
        ("Eval(foo)", None),
        ("setTimeout('foo')", None),
        ("setInterval('foo')", None),
        ("window.setTimeout('foo')", None),
        ("window.setInterval('foo')", None),
    ];

    let fail = vec![
        ("eval(foo)", None),
        // ("try { foo() } catch (ex) {throw ex} finally {}", None),
        // ("try { foo() } catch (ex) {}", None),
        // ("if (foo) {}", None),
        // ("while (foo) {}", None),
        // ("for (;foo;) {}", None),
        // ("switch(foo) {}", None),
        // ("switch (foo) { /* empty */ }", None),
        // ("try {} catch (ex) {}", Some(json!([ { "allowEmptyCatch": true }]))),
        // ("try { foo(); } catch (ex) {} finally {}", Some(json!([ { "allowEmptyCatch": true }]))),
        // ("try {} catch (ex) {} finally {}", Some(json!([ { "allowEmptyCatch": true }]))),
        // ("try { foo(); } catch (ex) {} finally {}", None),
    ];

    Tester::new(NoEval::NAME, pass, fail).test_and_snapshot();
}
