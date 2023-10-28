use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, fixer::Fix, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(require-number-to-fixed-digits-argument): Number method .toFixed() should have an argument")]
#[diagnostic(severity(warning), help("Pass an argument to .toFixed() method."))]
struct RequireNumberToFixedDigitsArgumentDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct RequireNumberToFixedDigitsArgument;

declare_oxc_lint!(
    /// ### What it does
    /// Enforce using the digits argument with Number.toFixed()
    ///
    /// ### Why is this bad?
    /// It's better to make it clear what the value of the digits argument is when calling Number.toFixed(),
    /// instead of relying on the default value of 0.
    ///
    /// ### Example
    /// ```javascript
    /// // Pass
    /// number.toFixed(0);
    /// number.toFixed(2);
    ///
    /// // Fail:
    /// number.toFixed();
    /// ```
    RequireNumberToFixedDigitsArgument,
    correctness
);

impl Rule for RequireNumberToFixedDigitsArgument {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(expr) = node.kind() else {
            return;
        };

        if !expr.arguments.is_empty() || expr.optional {
            return;
        }

        if let Some(member) = expr.callee.get_member_expr() {
            if let Expression::NewExpression(_) = member.object() {
                return;
            }

            if member.optional() || member.is_computed() {
                return;
            }

            if let Some(property_name) = member.static_property_name() {
                if property_name == "toFixed" {
                    let parenthesis_span = Span { start: member.span().end, end: expr.span.end };

                    ctx.diagnostic_with_fix(
                        RequireNumberToFixedDigitsArgumentDiagnostic(parenthesis_span),
                        || {
                            let modified_code = {
                                let mut formatter = ctx.formatter();

                                let mut parenthesis_span_without_right_one = parenthesis_span;
                                parenthesis_span_without_right_one.end -= 1;

                                let span_source_code = parenthesis_span_without_right_one
                                    .source_text(ctx.source_text());

                                formatter.print_str(span_source_code.as_bytes());
                                formatter.print_str(b"0)");

                                formatter.into_code()
                            };

                            Fix::new(modified_code, parenthesis_span)
                        },
                    );
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "number.toFixed(0)",
        "number.toFixed(...[])",
        "number.toFixed(2)",
        "number.toFixed(1,2,3)",
        "number[toFixed]()",
        "number[\"toFixed\"]()",
        "number?.toFixed()",
        "number.toFixed?.()",
        "number.notToFixed();",
        "new BigNumber(1).toFixed()",
        "new Number(1).toFixed()",
    ];

    let fail = vec![
        "const string = number.toFixed();",
        "const string = number.toFixed( /* comment */ );",
        "Number(1).toFixed()",
        "const bigNumber = new BigNumber(1); const string = bigNumber.toFixed();",
    ];

    let fix = vec![
        ("const string = number.toFixed();", "const string = number.toFixed(0);", None),
        (
            "const string = number.toFixed( /* comment */ );",
            "const string = number.toFixed( /* comment */ 0);",
            None,
        ),
        ("Number(1).toFixed()", "Number(1).toFixed(0)", None),
        (
            "const bigNumber = new BigNumber(1); const string = bigNumber.toFixed();",
            "const bigNumber = new BigNumber(1); const string = bigNumber.toFixed(0);",
            None,
        ),
    ];

    Tester::new_without_config(RequireNumberToFixedDigitsArgument::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
