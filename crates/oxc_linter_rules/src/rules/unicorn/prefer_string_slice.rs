use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_string_slice_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer String#slice() over String#{method_name}()"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferStringSlice;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer [`String#slice()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/slice) over [`String#substr()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substr) and [`String#substring()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substring).
    ///
    /// ### Why is this bad?
    ///
    /// [`String#substr()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substr) and [`String#substring()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substring) are the two lesser known legacy ways to slice a string. It's better to use [`String#slice()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/slice) as it's a more popular option with clearer behavior that has a consistent [`Array` counterpart](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/slice).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// "foo".substr(1, 2)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// "foo".slice(1, 2)
    /// ```
    PreferStringSlice,
    unicorn,
    pedantic,
    conditional_fix,
    version = "0.0.18",
);

impl Rule for PreferStringSlice {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        if let MemberExpression::StaticMemberExpression(v) = member_expr {
            if !matches!(v.property.name.as_str(), "substr" | "substring") {
                return;
            }
            let method_name = v.property.name.as_str();
            let has_spread_arguments = call_expr.arguments.iter().any(Argument::is_spread);
            let has_unsafe_arguments = match method_name {
                "substr" => has_spread_arguments || call_expr.arguments.len() >= 2,
                "substring" => {
                    has_spread_arguments || !is_safe_substring_arguments(&call_expr.arguments)
                }
                _ => unreachable!(),
            };

            if has_unsafe_arguments {
                ctx.diagnostic(prefer_string_slice_diagnostic(v.property.span, method_name));
                return;
            }

            ctx.diagnostic_with_fix(
                prefer_string_slice_diagnostic(v.property.span, method_name),
                |fixer| fixer.replace(v.property.span, "slice"),
            );
        }
    }
}

fn is_safe_substring_arguments(arguments: &[Argument<'_>]) -> bool {
    match arguments {
        [] => true,
        [start] => get_non_negative_integer_argument(start).is_some(),
        [start, end] => {
            let Some(start_value) = get_non_negative_integer_argument(start) else {
                return false;
            };
            let Some(end_value) = get_non_negative_integer_argument(end) else {
                return false;
            };
            start_value <= end_value
        }
        _ => false,
    }
}

fn get_non_negative_integer_argument(argument: &Argument<'_>) -> Option<f64> {
    if !argument.is_expression() {
        return None;
    }

    let Expression::NumericLiteral(number) = argument.to_expression().get_inner_expression() else {
        return None;
    };

    if number.value >= 0.0 && number.value.fract() == 0.0 { Some(number.value) } else { None }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const substr = foo.substr",
        "const substring = foo.substring",
        "foo.slice()",
        "foo.slice(0)",
        "foo.slice(1, 2)",
        "foo?.slice(1, 2)",
        "foo?.slice?.(1, 2)",
        "foo?.bar.baz.slice(1, 2)",
        "foo.slice(-3, -2)",
    ];

    let fail = vec![
        "foo.substr()",
        "foo?.substr()",
        "foo.bar?.substring()",
        "foo?.[0]?.substring()",
        "foo.bar.substr?.()",
        "foo.bar?.substring?.()",
        "foo.bar?.baz?.substr()",
        "foo.bar?.baz.substring()",
        "foo.bar.baz?.substr()",
        r#""foo".substr()"#,
        r#""foo".substr(1)"#,
        r#""foo".substr(1, 2)"#,
        r#""foo".substr(11, 8)"#,
        r#""foo".substr(bar.length, Math.min(baz, 100))"#,
        r#""foo".substr(1, length)"#,
        r#""foo".substr(1, "abc".length)"#,
        r#""foo".substr("1", 2)"#,
        r#""foo".substr(0, -1)"#,
        r#""foo".substr(0, "foo".length)"#,
        r#"const length = 123;
            "foo".substr(1, length - 4)"#,
        r#""foo".substr(1, length)"#,
        "const uri = 'foo';
            ((uri || '')).substr(1)",
        "foo.substr(start)",
        r#""foo".substr(1)"#,
        "foo.substr(start, length)",
        r#""foo".substr(1, 2)"#,
        "foo.substr(1, 2, 3)",
        r#""Sample".substr(0, "Sample".lastIndexOf("/"))"#,
        "foo.substring()",
        r#""foo".substring()"#,
        r#""foo".substring(1)"#,
        r#""foo".substring(1, 2)"#,
        r#""foo".substring(2, 1)"#,
        r#""foo".substring(-1, -5)"#,
        r#""foo".substring(-1, 2)"#,
        r#""foo".substring(length)"#,
        r#""foobar".substring("foo".length)"#,
        r#""foo".substring(0, length)"#,
        r#""foo".substring(length, 0)"#,
        "foo.substring(start)",
        r#""foo".substring(1)"#,
        "foo.substring(start, end)",
        r#""foo".substring(1, 3)"#,
        "foo.substring(1, 2, 3)",
        "function foo() {
                return (bar as string).substr(3);
            }",
        "function foo() {
                return ((bar as string)).substring(3);
            }",
        "/* 1 */ (( /* 2 */ 0 /* 3 */, /* 4 */ foo /* 5 */ )) /* 6 */
                . /* 7 */ substring /* 8 */ (
                    /* 9 */ (( /* 10 */ bar /* 11 */ )) /* 12 */,
                    /* 13 */ (( /* 14 */ 0 /* 15 */ )) /* 16 */,
                    /* 17 */
                )
            /* 18 */",
        "foo.substr(0, ...bar)",
        "foo.substr(...bar)",
        "foo.substr(0, (100, 1))",
        "foo.substr(0, 1, extraArgument)",
        "foo.substr((0, bar.length), (0, baz.length))",
        "foo.substring((10, 1), 0)",
        "foo.substring(0, (10, 1))",
        "foo.substring(0, await 1)",
        "foo.substring((10, bar))",
        r#"const string = "::";
            const output = string.substr(-2, 2);"#,
    ];

    let fix = vec![
        ("foo.substr()", "foo.slice()"),
        ("foo?.substr()", "foo?.slice()"),
        ("foo.bar?.substring()", "foo.bar?.slice()"),
        ("foo?.[0]?.substring()", "foo?.[0]?.slice()"),
        ("foo.bar.substr?.()", "foo.bar.slice?.()"),
        ("foo.bar?.substring?.()", "foo.bar?.slice?.()"),
        ("foo.bar?.baz?.substr()", "foo.bar?.baz?.slice()"),
        ("foo.bar?.baz.substring()", "foo.bar?.baz.slice()"),
        ("foo.bar.baz?.substr()", "foo.bar.baz?.slice()"),
        (r#""foo".substr()"#, r#""foo".slice()"#),
        // TODO: Get this passing.
        // (
        //     r#"const length = 123;
        //     "foo".substr(0, length)"#,
        //     r#"const length = 123;
        //     "foo".slice(0, Math.max(0, length))"#,
        // ),
        // (r#""foo".substr(0, -1)"#, r#""foo".slice(0, 0)"#),
        (
            "const uri = 'foo';
            ((uri || '')).substr(1)",
            "const uri = 'foo';
            ((uri || '')).slice(1)",
        ),
        ("foo.substr(start)", "foo.slice(start)"),
        (r#""foo".substr(1)"#, r#""foo".slice(1)"#),
        // TODO: Get this passing.
        // (r#""foo".substr(1, 2)"#, r#""foo".slice(1, 3)"#),
        // (
        //     r#""Sample".substr(0, "Sample".lastIndexOf("/"))"#,
        //     r#""Sample".slice(0, Math.max(0, "Sample".lastIndexOf("/")))"#,
        // ),
        ("foo.substring()", "foo.slice()"),
        (r#""foo".substring()"#, r#""foo".slice()"#),
        (r#""foo".substring(1)"#, r#""foo".slice(1)"#),
        (r#""foo".substring(1, 2)"#, r#""foo".slice(1, 2)"#),
        // TODO: Get this passing.
        // (r#""foo".substring(2, 1)"#, r#""foo".slice(1, 2)"#),
        // (r#""foo".substring(-1, -5)"#, r#""foo".slice(0, 0)"#),
        // (r#""foo".substring(-1, 2)"#, r#""foo".slice(0, 2)"#),
        // (r#""foo".substring(length)"#, r#""foo".slice(Math.max(0, length))"#),
        // TODO: Get this passing.
        // (r#""foo".substring(0, length)"#, r#""foo".slice(0, Math.max(0, length))"#),
        // (r#""foo".substring(length, 0)"#, r#""foo".slice(0, Math.max(0, length))"#),
        // ("foo.substring(start)", "foo.slice(Math.max(0, start))"),
        (r#""foo".substring(1, 3)"#, r#""foo".slice(1, 3)"#),
        (
            "function foo() {
                return (bar as string).substr(3);
            }",
            "function foo() {
                return (bar as string).slice(3);
            }",
        ),
        (
            "function foo() {
                return ((bar as string)).substring(3);
            }",
            "function foo() {
                return ((bar as string)).slice(3);
            }",
        ),
    ];

    Tester::new(PreferStringSlice::NAME, PreferStringSlice::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
