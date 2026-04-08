use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn detect_eval_with_expression_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eval() called with a non-literal argument")
        .with_help("Avoid calling eval() with dynamic expressions. This can lead to code injection vulnerabilities.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectEvalWithExpression;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects calls to `eval()` where the argument is not a string literal.
    ///
    /// ### Why is this bad?
    ///
    /// Calling `eval()` with dynamic expressions allows arbitrary code execution
    /// and is a common code injection vector. If the argument comes from user
    /// input or an untrusted source, an attacker can execute arbitrary JavaScript.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// eval(userInput);
    /// eval(a + b);
    /// eval(getCode());
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// eval("var x = 1");
    /// ```
    DetectEvalWithExpression,
    oxc,
    suspicious,
    none
);

impl Rule for DetectEvalWithExpression {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Expression::Identifier(callee) = &call_expr.callee else {
            return;
        };

        if callee.name != "eval" {
            return;
        }

        let Some(arg) = call_expr.arguments.first().and_then(|a| a.as_expression()) else {
            return;
        };

        match arg {
            Expression::StringLiteral(_) => return,
            Expression::TemplateLiteral(tpl) if tpl.expressions.is_empty() => return,
            _ => {}
        }

        ctx.diagnostic(detect_eval_with_expression_diagnostic(call_expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![r#"eval("var x = 1")"#, "console.log('hello')", "let x = 1;"];

    let fail = vec!["eval(userInput)", "eval(a + b)", "eval(getCode())"];

    Tester::new(DetectEvalWithExpression::NAME, DetectEvalWithExpression::PLUGIN, pass, fail)
        .test_and_snapshot();
}
