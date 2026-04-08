use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn detect_non_literal_regexp_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("new RegExp() called with a non-literal argument")
        .with_help("Avoid constructing regular expressions from dynamic values. This can lead to ReDoS or injection attacks.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectNonLiteralRegexp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects `new RegExp()` calls where the pattern argument is not a string literal.
    ///
    /// ### Why is this bad?
    ///
    /// Constructing regular expressions from untrusted input can lead to Regular
    /// Expression Denial of Service (ReDoS) attacks, where a crafted pattern
    /// causes catastrophic backtracking.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// new RegExp(userInput);
    /// new RegExp(pattern, "gi");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// new RegExp("^[a-z]+$");
    /// /^[a-z]+$/;
    /// ```
    DetectNonLiteralRegexp,
    oxc,
    suspicious,
    none
);

impl Rule for DetectNonLiteralRegexp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        let Expression::Identifier(callee) = &new_expr.callee else {
            return;
        };

        if callee.name != "RegExp" {
            return;
        }

        let Some(arg) = new_expr.arguments.first().and_then(|a| a.as_expression()) else {
            return;
        };

        match arg {
            Expression::StringLiteral(_) => return,
            Expression::TemplateLiteral(tpl) if tpl.expressions.is_empty() => return,
            _ => {}
        }

        ctx.diagnostic(detect_non_literal_regexp_diagnostic(new_expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"new RegExp("^[a-z]+$")"#,
        r#"new RegExp("test", "gi")"#,
        "new RegExp(`static`)",
        "/^[a-z]+$/",
        "new Map()",
    ];

    let fail = vec![
        "new RegExp(userInput)",
        "new RegExp(pattern, 'gi')",
        "new RegExp(getPattern())",
        "new RegExp(`dynamic-${name}`)",
    ];

    Tester::new(DetectNonLiteralRegexp::NAME, DetectNonLiteralRegexp::PLUGIN, pass, fail)
        .test_and_snapshot();
}
