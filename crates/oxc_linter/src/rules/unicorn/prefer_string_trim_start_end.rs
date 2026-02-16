use oxc_ast::{AstKind, ast::MemberExpression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_string_trim_start_end_diagnostic(
    span: Span,
    good_trim: &str,
    bad_trim: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `{good_trim}` over `{bad_trim}`"))
        .with_help(format!("Replace with `{good_trim}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferStringTrimStartEnd;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// [`String#trimLeft()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trimLeft) and [`String#trimRight()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trimRight) are aliases of [`String#trimStart()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trimStart) and [`String#trimEnd()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trimEnd). This is to ensure consistency and use [direction](https://developer.mozilla.org/en-US/docs/Learn/CSS/Building_blocks/Handling_different_text_directions)-independent wording.
    ///
    /// ### Why is this bad?
    ///
    /// The `trimLeft` and `trimRight` names are confusing and inconsistent with the rest of the language.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// str.trimLeft();
    /// str.trimRight();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// str.trimStart();
    /// str.trimEnd();
    /// ```
    PreferStringTrimStartEnd,
    unicorn,
    style,
    fix
);

impl Rule for PreferStringTrimStartEnd {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let (span, name) = match member_expr {
            MemberExpression::StaticMemberExpression(v) => {
                if !matches!(v.property.name.as_str(), "trimLeft" | "trimRight") {
                    return;
                }
                (v.property.span, &v.property.name)
            }
            _ => return,
        };

        if !call_expr.arguments.is_empty() {
            return;
        }

        ctx.diagnostic_with_fix(
            prefer_string_trim_start_end_diagnostic(
                span,
                get_replacement(name.as_str()),
                name.as_str(),
            ),
            |fixer| fixer.replace(span, get_replacement(name.as_str())),
        );
    }
}

fn get_replacement(name: &str) -> &'static str {
    match name {
        "trimLeft" => "trimStart",
        "trimRight" => "trimEnd",
        _ => unreachable!(),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo.trimStart()",
        "foo.trimStart?.()",
        "foo.trimEnd()",
        "new foo.trimLeft();",
        "trimLeft();",
        "foo['trimLeft']();",
        "foo[trimLeft]();",
        "foo.bar();",
        "foo.trimLeft(extra);",
        "foo.trimLeft(...argumentsArray)",
        "foo.bar(trimLeft)",
        "foo.bar(foo.trimLeft)",
        "trimLeft.foo()",
        "foo.trimLeft.bar()",
    ];

    let fail = vec![
        "foo.trimLeft()",
        "foo.trimRight()",
        "trimLeft.trimRight()",
        "foo.trimLeft.trimRight()",
        r#""foo".trimLeft()"#,
        "foo?.trimLeft()",
    ];

    let fix = vec![
        ("foo.trimLeft()", "foo.trimStart()"),
        ("foo.trimRight()", "foo.trimEnd()"),
        ("trimLeft.trimRight()", "trimLeft.trimEnd()"),
        ("foo.trimLeft.trimRight()", "foo.trimLeft.trimEnd()"),
        (r#""foo".trimLeft()"#, r#""foo".trimStart()"#),
        ("foo?.trimLeft()", "foo?.trimStart()"),
    ];

    Tester::new(PreferStringTrimStartEnd::NAME, PreferStringTrimStartEnd::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
