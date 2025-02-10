use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

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
    /// Prefers usage of `String.prototype.codePointAt` over `String.prototype.charCodeAt`.
    /// Prefers usage of `String.fromCodePoint` over `String.fromCharCode`.
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
    fix
);

impl Rule for PreferCodePoint {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(memb_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        if memb_expr.is_computed() || memb_expr.optional() || call_expr.optional {
            return;
        }

        let (current, replacement, span) = match memb_expr.static_property_info() {
            Some((span, "charCodeAt")) => ("charCodeAt", "codePointAt", span),
            Some((span, "fromCharCode")) => ("fromCharCode", "fromCodePoint", span),
            _ => return,
        };

        ctx.diagnostic_with_fix(
            prefer_code_point_diagnostic(span, replacement, current),
            |fixer| fixer.replace(span, replacement),
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#""🦄".codePointAt(0)"#,
        r"foo.charCodeAt",
        r"new foo.charCodeAt",
        r"charCodeAt(0)",
        r"foo.charCodeAt?.(0)",
        r"foo?.charCodeAt(0)",
        r"foo[charCodeAt](0)",
        r#"foo["charCodeAt"](0)"#,
        r"foo.notCharCodeAt(0)",
        r"String.fromCodePoint(0x1f984)",
        r"String.fromCodePoint",
        r"new String.fromCodePoint",
        r"fromCodePoint(foo)",
        r"String.fromCodePoint?.(foo)",
        r"String?.fromCodePoint(foo)",
        r"window.String.fromCodePoint(foo)",
        r"String[fromCodePoint](foo)",
        r#"String["fromCodePoint"](foo)"#,
        r"String.notFromCodePoint(foo)",
        r"NotString.fromCodePoint(foo)",
    ];

    let fail = vec![
        r"string.charCodeAt(index)",
        r"(( (( string )).charCodeAt( ((index)), )))",
        r"String.fromCharCode( code )",
        r"(( (( String )).fromCharCode( ((code)), ) ))",
    ];

    let fix = vec![
        (r"string.charCodeAt(index)", r"string.codePointAt(index)"),
        (
            r"(( (( String )).fromCharCode( ((code)), ) ))",
            r"(( (( String )).fromCodePoint( ((code)), ) ))",
        ),
        (r#""🦄".charCodeAt(0)"#, r#""🦄".codePointAt(0)"#),
        (r"String.fromCharCode(0x1f984);", r"String.fromCodePoint(0x1f984);"),
    ];

    Tester::new(PreferCodePoint::NAME, PreferCodePoint::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
