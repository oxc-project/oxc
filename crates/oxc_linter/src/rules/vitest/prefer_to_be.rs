use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{PossibleJestNode, is_equality_matcher, parse_expect_and_typeof_vitest_fn_call},
};

fn use_to_be(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBe()` when comparing primitive values")
        .with_help("Replace `toEqual()` with `toBe()` for primitive comparison")
        .with_label(span)
}

fn use_to_be_null(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeNull()` when checking for null values")
        .with_help("Replace with `toBeNull()` for more explicit null checking")
        .with_label(span)
}

fn use_to_be_undefined(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeUndefined()` when checking for undefined values")
        .with_help("Replace with `toBeUndefined()` for more explicit undefined checking")
        .with_label(span)
}

fn use_to_be_nan(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeNaN()` when checking for NaN values")
        .with_help("Replace with `toBeNaN()` for more explicit NaN checking")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferToBe;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces using more specific Vitest matchers instead of generic
    /// equality matchers when comparing with specific values.
    ///
    /// ### Why is this bad?
    ///
    /// Using specific matchers like `toBeNull()`, `toBeUndefined()`, or `toBeNaN()`
    /// makes test assertions more explicit and easier to understand. Additionally,
    /// using `toBe()` for primitive values is more performant than `toEqual()`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// expect(value).toEqual("string");
    /// expect(value).toBe(null);
    /// expect(value).toEqual(undefined);
    /// expect(value).toBe(NaN);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// expect(value).toBe("string");
    /// expect(value).toBeNull();
    /// expect(value).toBeUndefined();
    /// expect(value).toBeNaN();
    /// expect(obj).toStrictEqual({ key: "value" });
    /// ```
    PreferToBe,
    vitest,
    style,
    fix
);

impl Rule for PreferToBe {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(vitest_fn_call) =
            parse_expect_and_typeof_vitest_fn_call(call_expr, jest_node, ctx)
        else {
            return;
        };

        let Some(matcher) = vitest_fn_call.matcher() else {
            return;
        };

        if !is_equality_matcher(matcher) || vitest_fn_call.args.is_empty() {
            return;
        }

        let Some(arg_expr) = vitest_fn_call.args.first().and_then(Argument::as_expression) else {
            return;
        };

        // Unwrap TypeScript type assertions to get the actual value
        let inner_expr = arg_expr.get_inner_expression();

        // Create span from matcher to end of call
        let span = Span::new(matcher.span.start, call_expr.span.end);

        // Check if using computed member expression (e.g., ["toBe"])
        let is_computed = matches!(matcher.parent, Some(Expression::ComputedMemberExpression(_)));

        match inner_expr {
            // Handle null: toBe(null) or toEqual(null) -> toBeNull()
            Expression::NullLiteral(_) => {
                let replacement = if is_computed { r#"["toBeNull"]()"# } else { "toBeNull()" };
                ctx.diagnostic_with_fix(use_to_be_null(span), |fixer| {
                    fixer.replace(span, replacement)
                });
            }

            // Handle undefined and NaN identifiers
            Expression::Identifier(id) if id.name == "undefined" || id.name == "NaN" => {
                if id.name == "undefined" {
                    let replacement =
                        if is_computed { r#"["toBeUndefined"]()"# } else { "toBeUndefined()" };
                    ctx.diagnostic_with_fix(use_to_be_undefined(span), |fixer| {
                        fixer.replace(span, replacement)
                    });
                } else {
                    let replacement = if is_computed { r#"["toBeNaN"]()"# } else { "toBeNaN()" };
                    ctx.diagnostic_with_fix(use_to_be_nan(span), |fixer| {
                        fixer.replace(span, replacement)
                    });
                }
            }

            // Handle primitive literals: suggest toBe instead of toEqual
            Expression::StringLiteral(_) | Expression::BooleanLiteral(_) => {
                // Only suggest toBe if currently using toEqual or toStrictEqual
                let matcher_name = matcher.name();
                if let Some(name) = matcher_name.as_deref() {
                    if matches!(name, "toEqual" | "toStrictEqual") {
                        // Only replace the matcher name, not the entire expression
                        let matcher_replacement = if is_computed { r#"["toBe"]"# } else { "toBe" };
                        ctx.diagnostic_with_fix(use_to_be(matcher.span), |fixer| {
                            fixer.replace(matcher.span, matcher_replacement)
                        });
                    }
                }
            }

            // Handle numeric literals separately to check for floats
            Expression::NumericLiteral(num) => {
                let matcher_name = matcher.name();
                if let Some(name) = matcher_name.as_deref() {
                    if matches!(name, "toEqual" | "toStrictEqual") {
                        // Check if this is a float by examining the source text
                        let num_span = num.span;
                        let has_decimal = ctx.source_range(num_span).contains('.');
                        if !has_decimal {
                            // Only suggest toBe for integer literals
                            let matcher_replacement =
                                if is_computed { r#"["toBe"]"# } else { "toBe" };
                            ctx.diagnostic_with_fix(use_to_be(matcher.span), |fixer| {
                                fixer.replace(matcher.span, matcher_replacement)
                            });
                        }
                    }
                }
            }

            // For RegExp, objects, arrays, etc. - don't suggest changes
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "expect(null).toBeNull();",
        "expect(null).not.toBeNull();",
        "expect(null).toBe(-1);",
        "expect(null).toBe(1);",
        "expect(obj).toStrictEqual([ x, 1 ]);",
        "expect(obj).toStrictEqual({ x: 1 });",
        "expect(obj).not.toStrictEqual({ x: 1 });",
        "expect(value).toMatchSnapshot();",
        "expect(catchError()).toStrictEqual({ message: 'oh noes!' })",
        r#"expect("something");"#,
        "expect(token).toStrictEqual(/[abc]+/g);",
        "expect(token).toStrictEqual(new RegExp('[abc]+', 'g'));",
        "expect(0.1 + 0.2).toEqual(0.3);",
        "expect(NaN).toBeNaN();",
        "expect(true).not.toBeNaN();",
        "expect({}).toEqual({});",
        "expect(something).toBe()",
        "expect(something).toBe(somethingElse)",
        "expect(something).toEqual(somethingElse)",
        "expect(something).not.toBe(somethingElse)",
        "expect(something).not.toEqual(somethingElse)",
        "expect(undefined).toBe",
        r#"expect("something");"#,
        "expect(null).toBeNull();",
        "expect(null).not.toBeNull();",
        "expect(null).toBe(1);",
        "expect(obj).toStrictEqual([ x, 1 ]);",
        "expect(obj).toStrictEqual({ x: 1 });",
        "expect(obj).not.toStrictEqual({ x: 1 });",
    ];

    let fail = vec![
        r#"expect(value).toEqual("my string");"#,
        r#"expect("a string").not.toEqual(null);"#,
        r#"expect("a string").not.toStrictEqual(null);"#,
        "expect(NaN).toBe(NaN);",
        r#"expect("a string").not.toBe(NaN);"#,
        r#"expect("a string").not.toStrictEqual(NaN);"#,
        "expect(null).toBe(null);",
        "expect(null).toEqual(null);",
        r#"expect("a string").not.toEqual(null as number);"#,
        "expect(undefined).toBe(undefined as unknown as string as any);",
        r#"expect("a string").toEqual(undefined as number);"#,
    ];

    let fix = vec![
        (r#"expect(value).toEqual("my string");"#, r#"expect(value).toBe("my string");"#, None),
        (r#"expect("a string").not.toEqual(null);"#, r#"expect("a string").not.toBeNull();"#, None),
        (
            r#"expect("a string").not.toStrictEqual(null);"#,
            r#"expect("a string").not.toBeNull();"#,
            None,
        ),
        ("expect(NaN).toBe(NaN);", "expect(NaN).toBeNaN();", None),
        (r#"expect("a string").not.toBe(NaN);"#, r#"expect("a string").not.toBeNaN();"#, None),
        (
            r#"expect("a string").not.toStrictEqual(NaN);"#,
            r#"expect("a string").not.toBeNaN();"#,
            None,
        ),
        ("expect(null).toBe(null);", "expect(null).toBeNull();", None),
        ("expect(null).toEqual(null);", "expect(null).toBeNull();", None),
        (
            r#"expect("a string").not.toEqual(null as number);"#,
            r#"expect("a string").not.toBeNull();"#,
            None,
        ),
        (
            "expect(undefined).toBe(undefined as unknown as string as any);",
            "expect(undefined).toBeUndefined();",
            None,
        ),
        (
            r#"expect("a string").toEqual(undefined as number);"#,
            r#"expect("a string").toBeUndefined();"#,
            None,
        ),
    ];
    Tester::new(PreferToBe::NAME, PreferToBe::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
