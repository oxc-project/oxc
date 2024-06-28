use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn prefer_string_trim_start_end_diagnostic(span0: Span, x1: &str, x2: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "eslint-plugin-unicorn(prefer-string-trim-start-end): Prefer `{x1}` over `{x2}`"
    ))
    .with_help(format!("Replace with `{x1}`"))
    .with_label(span0)
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
    /// ### Example
    /// ```javascript
    /// // Bad
    /// str.trimLeft();
    /// str.trimRight();
    ///
    /// // Good
    /// str.trimStart();
    /// str.trimEnd();
    /// ```
    PreferStringTrimStartEnd,
    style
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

        ctx.diagnostic(prefer_string_trim_start_end_diagnostic(
            span,
            name.as_str(),
            get_replacement(name.as_str()),
        ));
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
        r"foo.trimStart()",
        r"foo.trimStart?.()",
        r"foo.trimEnd()",
        r"new foo.trimLeft();",
        r"trimLeft();",
        r"foo['trimLeft']();",
        r"foo[trimLeft]();",
        r"foo.bar();",
        r"foo.trimLeft(extra);",
        r"foo.trimLeft(...argumentsArray)",
        r"foo.bar(trimLeft)",
        r"foo.bar(foo.trimLeft)",
        r"trimLeft.foo()",
        r"foo.trimLeft.bar()",
    ];

    let fail = vec![
        r"foo.trimLeft()",
        r"foo.trimRight()",
        r"trimLeft.trimRight()",
        r"foo.trimLeft.trimRight()",
        r#""foo".trimLeft()"#,
        r"foo?.trimLeft()",
    ];

    Tester::new(PreferStringTrimStartEnd::NAME, pass, fail).test_and_snapshot();
}
