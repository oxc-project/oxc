use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_code_point_diagnostic(span: Span, good_method: &str, bad_method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `{good_method}` over `{bad_method}`"))
        .with_help(format!("Unicode is better supported in `{good_method}` than `{bad_method}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferCodePoint;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer usage of `String#codePointAt` over `String#charCodeAt`.
    /// Prefer usage of `String.fromCodePoint` over `String.fromCharCode`.
    ///
    /// ### Why is this bad?
    ///
    /// Unicode is better supported in [`String#codePointAt()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/codePointAt) and [`String.fromCodePoint()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCodePoint).
    ///
    /// [Difference between `String.fromCodePoint()` and `String.fromCharCode()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCodePoint#compared_to_fromcharcode)
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// '🦄'.charCodeAt(0);
    /// String.fromCharCode(0x1f984);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// '🦄'.codePointAt(0);
    /// String.fromCodePoint(0x1f984);
    /// ```
    PreferCodePoint,
    unicorn,
    pedantic,
    fix_dangerous,
    version = "0.0.16",
    short_description = "Prefer `String#codePointAt` over `String#charCodeAt` and `String.fromCodePoint` over `String.fromCharCode`.",
);

impl Rule for PreferCodePoint {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StaticMemberExpression(member_expr) = node.kind() else {
            return;
        };

        let (span, property_name) = member_expr.static_property_info();
        let (current, replacement) = match property_name {
            "fromCharCode" => {
                if !member_expr.object.is_specific_id("String") {
                    return;
                }
                ("fromCharCode", "fromCodePoint")
            }
            "charCodeAt" => {
                let AstKind::CallExpression(call_expr) = ctx.nodes().parent_kind(node.id()) else {
                    return;
                };
                if call_expr.optional
                    || call_expr.callee.as_member_expression().and_then(|callee| {
                        callee.static_property_info().map(|(_, property_name)| property_name)
                    }) != Some("charCodeAt")
                {
                    return;
                }
                ("charCodeAt", "codePointAt")
            }
            _ => return,
        };

        ctx.diagnostic_with_dangerous_fix(
            prefer_code_point_diagnostic(span, replacement, current),
            |fixer| fixer.replace(span, replacement),
        );
    }
}

#[test]
fn test() {
    use crate::fixer::FixKind;
    use crate::tester::Tester;

    let pass = vec![
        r#""🦄".codePointAt(0)"#,
        "foo.charCodeAt",
        "new foo.charCodeAt",
        "charCodeAt(0)",
        "foo.charCodeAt?.(0)",
        "foo[charCodeAt](0)",
        r#"foo["charCodeAt"](0)"#,
        "foo.notCharCodeAt(0)",
        "String.fromCodePoint(0x1f984)",
        "String.fromCodePoint",
        "NotString.fromCharCode(foo)",
        "new String.fromCodePoint",
        "fromCodePoint(foo)",
        "String.fromCodePoint?.(foo)",
        "String?.fromCodePoint(foo)",
        "window.String.fromCodePoint(foo)",
        "String[fromCodePoint](foo)",
        r#"String["fromCodePoint"](foo)"#,
        "String.notFromCodePoint(foo)",
        "NotString.fromCodePoint(foo)",
    ];

    let fail = vec![
        "string.charCodeAt(index)",
        "string?.charCodeAt(index)",
        "(( (( string )).charCodeAt( ((index)), )))",
        "String.fromCharCode( code )",
        "(( (( String )).fromCharCode( ((code)), ) ))",
        "String.fromCharCode.bind(String)",
        "const x = String.fromCharCode",
    ];

    let fix = vec![
        ("string.charCodeAt(index)", "string.codePointAt(index)", None, FixKind::DangerousFix),
        (
            "(( (( String )).fromCharCode( ((code)), ) ))",
            "(( (( String )).fromCodePoint( ((code)), ) ))",
            None,
            FixKind::DangerousFix,
        ),
        (
            "String.fromCharCode.bind(String)",
            "String.fromCodePoint.bind(String)",
            None,
            FixKind::DangerousFix,
        ),
        (
            "const x = String.fromCharCode",
            "const x = String.fromCodePoint",
            None,
            FixKind::DangerousFix,
        ),
        (r#""🦄".charCodeAt(0)"#, r#""🦄".codePointAt(0)"#, None, FixKind::DangerousFix),
        (
            "String.fromCharCode(0x1f984);",
            "String.fromCodePoint(0x1f984);",
            None,
            FixKind::DangerousFix,
        ),
        // `--fix` alone must not rewrite this. The result feeds a bitwise operator, so
        // the rename changes the value for astral input. Only `--fix-dangerously` applies it.
        (
            "hash ^= string.charCodeAt(index)",
            "hash ^= string.charCodeAt(index)",
            None,
            FixKind::SafeFix,
        ),
    ];

    Tester::new(PreferCodePoint::NAME, PreferCodePoint::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
