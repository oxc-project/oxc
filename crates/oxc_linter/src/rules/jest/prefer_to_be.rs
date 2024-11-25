use oxc_ast::{
    ast::{Argument, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        is_equality_matcher, parse_expect_jest_fn_call, KnownMemberExpressionProperty,
        ParsedExpectFnCall, PossibleJestNode,
    },
};

fn use_to_be(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBe` when expecting primitive literals.").with_label(span)
}

fn use_to_be_undefined(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeUndefined` instead.").with_label(span)
}

fn use_to_be_defined(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeDefined` instead.").with_label(span)
}

fn use_to_be_null(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeNull` instead.").with_label(span)
}

fn use_to_be_na_n(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeNaN` instead.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferToBe;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When asserting against primitive literals such as numbers and strings, the
    /// equality matchers all operate the same, but read slightly differently in code.
    ///
    /// This rule recommends using the `toBe` matcher in these situations, as it forms
    /// the most grammatically natural sentence. For `null`, `undefined`, and `NaN` this
    /// rule recommends using their specific `toBe` matchers, as they give better error
    /// messages as well.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // valid
    /// expect(value).not.toBe(5);
    /// expect(getMessage()).toBe('hello world');
    /// expect(loadMessage()).resolves.toBe('hello world');
    /// expect(didError).not.toBe(true);
    /// expect(catchError()).toStrictEqual({ message: 'oh noes!' });
    ///
    /// // invalid
    /// expect(value).not.toEqual(5);
    /// expect(getMessage()).toStrictEqual('hello world');
    /// expect(loadMessage()).resolves.toEqual('hello world');
    /// ```
    PreferToBe,
    style,
    fix
);

#[derive(Clone, Debug, PartialEq)]
enum PreferToBeKind {
    Defined,
    NaN,
    Null,
    ToBe,
    Undefined,
}

impl Rule for PreferToBe {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl PreferToBe {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_expect_fn_call) =
            parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let Some(matcher) = jest_expect_fn_call.matcher() else {
            return;
        };

        let has_not_modifier =
            jest_expect_fn_call.modifiers().iter().any(|modifier| modifier.is_name_equal("not"));

        if has_not_modifier {
            if matcher.is_name_equal("toBeUndefined") {
                Self::check_and_fix(
                    &PreferToBeKind::Defined,
                    call_expr,
                    matcher,
                    &jest_expect_fn_call,
                    ctx,
                );
                return;
            } else if matcher.is_name_equal("toBeDefined") {
                Self::check_and_fix(
                    &PreferToBeKind::Undefined,
                    call_expr,
                    matcher,
                    &jest_expect_fn_call,
                    ctx,
                );
                return;
            }
        }

        if !is_equality_matcher(matcher) || jest_expect_fn_call.args.len() == 0 {
            return;
        }

        let Some(arg_expr) = jest_expect_fn_call.args.first().and_then(Argument::as_expression)
        else {
            return;
        };
        let first_matcher_arg = arg_expr.get_inner_expression();

        if first_matcher_arg.is_undefined() {
            let kind =
                if has_not_modifier { PreferToBeKind::Defined } else { PreferToBeKind::Undefined };
            Self::check_and_fix(&kind, call_expr, matcher, &jest_expect_fn_call, ctx);
            return;
        }

        if first_matcher_arg.is_nan() {
            Self::check_and_fix(
                &PreferToBeKind::NaN,
                call_expr,
                matcher,
                &jest_expect_fn_call,
                ctx,
            );
            return;
        }

        if first_matcher_arg.is_null() {
            Self::check_and_fix(
                &PreferToBeKind::Null,
                call_expr,
                matcher,
                &jest_expect_fn_call,
                ctx,
            );
            return;
        }

        if Self::should_use_tobe(first_matcher_arg) && !matcher.is_name_equal("toBe") {
            Self::check_and_fix(
                &PreferToBeKind::ToBe,
                call_expr,
                matcher,
                &jest_expect_fn_call,
                ctx,
            );
        }
    }

    fn should_use_tobe(first_matcher_arg: &Expression) -> bool {
        let mut expr = first_matcher_arg;

        if let Expression::UnaryExpression(unary_expr) = expr {
            if unary_expr.operator.as_str() == "-" {
                expr = &unary_expr.argument;
            }
        }

        if matches!(expr, Expression::RegExpLiteral(_)) {
            return false;
        }

        matches!(
            expr,
            Expression::BigIntLiteral(_)
                | Expression::BooleanLiteral(_)
                | Expression::NumericLiteral(_)
                | Expression::NullLiteral(_)
                | Expression::TemplateLiteral(_)
                | Expression::StringLiteral(_)
        )
    }

    fn check_and_fix(
        kind: &PreferToBeKind,
        call_expr: &CallExpression,
        matcher: &KnownMemberExpressionProperty,
        jest_expect_fn_call: &ParsedExpectFnCall,
        ctx: &LintContext,
    ) {
        let span = matcher.span;
        let end = call_expr.span.end;

        let is_cmp_mem_expr = match matcher.parent {
            Some(Expression::ComputedMemberExpression(_)) => true,
            Some(Expression::StaticMemberExpression(_) | Expression::PrivateFieldExpression(_)) => {
                false
            }
            _ => return,
        };

        let modifiers = jest_expect_fn_call.modifiers();
        let maybe_not_modifier = modifiers.iter().find(|modifier| modifier.is_name_equal("not"));

        if kind == &PreferToBeKind::Undefined {
            ctx.diagnostic_with_fix(use_to_be_undefined(span), |fixer| {
                let new_matcher =
                    if is_cmp_mem_expr { "[\"toBeUndefined\"]()" } else { "toBeUndefined()" };
                let span = if let Some(not_modifier) = maybe_not_modifier {
                    Span::new(not_modifier.span.start, end)
                } else {
                    Span::new(span.start, end)
                };
                fixer.replace(span, new_matcher)
            });
        } else if kind == &PreferToBeKind::Defined {
            ctx.diagnostic_with_fix(use_to_be_defined(span), |fixer| {
                let (new_matcher, start) = if is_cmp_mem_expr {
                    ("[\"toBeDefined\"]()", modifiers.first().unwrap().span.end)
                } else {
                    ("toBeDefined()", maybe_not_modifier.unwrap().span.start)
                };

                fixer.replace(Span::new(start, end), new_matcher)
            });
        } else if kind == &PreferToBeKind::Null {
            ctx.diagnostic_with_fix(use_to_be_null(span), |fixer| {
                let new_matcher = if is_cmp_mem_expr { "\"toBeNull\"]()" } else { "toBeNull()" };
                fixer.replace(Span::new(span.start, end), new_matcher)
            });
        } else if kind == &PreferToBeKind::NaN {
            ctx.diagnostic_with_fix(use_to_be_na_n(span), |fixer| {
                let new_matcher = if is_cmp_mem_expr { "\"toBeNaN\"]()" } else { "toBeNaN()" };
                fixer.replace(Span::new(span.start, end), new_matcher)
            });
        } else {
            ctx.diagnostic_with_fix(use_to_be(span), |fixer| {
                let new_matcher = if is_cmp_mem_expr { "\"toBe\"" } else { "toBe" };
                fixer.replace(span, new_matcher)
            });
        }
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect(null).toBeNull();", None),
        ("expect(null).not.toBeNull();", None),
        ("expect(null).toBe(1);", None),
        ("expect(null).toBe(-1);", None),
        ("expect(null).toBe(...1);", None),
        ("expect(obj).toStrictEqual([ x, 1 ]);", None),
        ("expect(obj).toStrictEqual({ x: 1 });", None),
        ("expect(obj).not.toStrictEqual({ x: 1 });", None),
        ("expect(value).toMatchSnapshot();", None),
        ("expect(catchError()).toStrictEqual({ message: 'oh noes!' })", None),
        ("expect(\"something\");", None),
        ("expect(token).toStrictEqual(/[abc]+/g);", None),
        ("expect(token).toStrictEqual(new RegExp('[abc]+', 'g'));", None),
        ("expect(value).toEqual(dedent`my string`);", None),
        // null
        ("expect(null).toBeNull();", None),
        ("expect(null).not.toBeNull();", None),
        ("expect(null).toBe(1);", None),
        ("expect(obj).toStrictEqual([ x, 1 ]);", None),
        ("expect(obj).toStrictEqual({ x: 1 });", None),
        ("expect(obj).not.toStrictEqual({ x: 1 });", None),
        ("expect(value).toMatchSnapshot();", None),
        ("expect(catchError()).toStrictEqual({ message: 'oh noes!' })", None),
        ("expect(\"something\");", None),
        ("expect(null).not.toEqual();", None),
        ("expect(null).toBe();", None),
        ("expect(null).toMatchSnapshot();", None),
        ("expect(\"a string\").toMatchSnapshot(null);", None),
        ("expect(\"a string\").not.toMatchSnapshot();", None),
        ("expect(null).toBe", None),
        // undefined
        ("expect(undefined).toBeUndefined();", None),
        ("expect(true).toBeDefined();", None),
        ("expect({}).toEqual({});", None),
        ("expect(something).toBe()", None),
        ("expect(something).toBe(somethingElse)", None),
        ("expect(something).toEqual(somethingElse)", None),
        ("expect(something).not.toBe(somethingElse)", None),
        ("expect(something).not.toEqual(somethingElse)", None),
        ("expect(undefined).toBe", None),
        ("expect(\"something\");", None),
        // NaN
        ("expect(NaN).toBeNaN();", None),
        ("expect(true).not.toBeNaN();", None),
        ("expect({}).toEqual({});", None),
        ("expect(something).toBe()", None),
        ("expect(something).toBe(somethingElse)", None),
        ("expect(something).toEqual(somethingElse)", None),
        ("expect(something).not.toBe(somethingElse)", None),
        ("expect(something).not.toEqual(somethingElse)", None),
        ("expect(undefined).toBe", None),
        ("expect(\"something\");", None),
        // undefined vs defined
        ("expect(NaN).toBeNaN();", None),
        ("expect(true).not.toBeNaN();", None),
        ("expect({}).toEqual({});", None),
        ("expect(something).toBe()", None),
        ("expect(something).toBe(somethingElse)", None),
        ("expect(something).toEqual(somethingElse)", None),
        ("expect(something).not.toBe(somethingElse)", None),
        ("expect(something).not.toEqual(somethingElse)", None),
        ("expect(undefined).toBe", None),
        ("expect(\"something\");", None),
        // typescript edition
        (
            "(expect('Model must be bound to an array if the multiple property is true') as any).toHaveBeenTipped()",
            None,
        ),
    ];

    let fail = vec![
        ("expect(value).toEqual(\"my string\");", None),
        ("expect(value).toStrictEqual(\"my string\");", None),
        ("expect(value).toStrictEqual(1);", None),
        ("expect(value).toStrictEqual(1,);", None),
        ("expect(value).toStrictEqual(-1);", None),
        ("expect(value).toEqual(`my string`);", None),
        ("expect(value)[\"toEqual\"](`my string`);", None),
        ("expect(value).toStrictEqual(`my ${string}`);", None),
        ("expect(loadMessage()).resolves.toStrictEqual(\"hello world\");", None),
        ("expect(loadMessage()).resolves[\"toStrictEqual\"](\"hello world\");", None),
        ("expect(loadMessage())[\"resolves\"].toStrictEqual(\"hello world\");", None),
        ("expect(loadMessage()).resolves.toStrictEqual(false);", None),
        // null
        ("expect(null).toBe(null);", None),
        ("expect(null).toEqual(null);", None),
        ("expect(null).toEqual(null,);", None),
        ("expect(null).toStrictEqual(null);", None),
        ("expect(\"a string\").not.toBe(null);", None),
        ("expect(\"a string\").not[\"toBe\"](null);", None),
        ("expect(\"a string\")[\"not\"][\"toBe\"](null);", None),
        ("expect(\"a string\").not.toEqual(null);", None),
        ("expect(\"a string\").not.toStrictEqual(null);", None),
        // undefined
        ("expect(undefined).toBe(undefined);", None),
        ("expect(undefined).toEqual(undefined);", None),
        ("expect(undefined).toStrictEqual(undefined);", None),
        ("expect(\"a string\").not.toBe(undefined);", None),
        ("expect(\"a string\").rejects.not.toBe(undefined);", None),
        ("expect(\"a string\").rejects.not[\"toBe\"](undefined);", None),
        ("expect(\"a string\").not.toEqual(undefined);", None),
        ("expect(\"a string\").not.toStrictEqual(undefined);", None),
        // NaN
        ("expect(NaN).toBe(NaN);", None),
        ("expect(NaN).toEqual(NaN);", None),
        ("expect(NaN).toStrictEqual(NaN);", None),
        ("expect(\"a string\").not.toBe(NaN);", None),
        ("expect(\"a string\").rejects.not.toBe(NaN);", None),
        ("expect(\"a string\")[\"rejects\"].not.toBe(NaN);", None),
        ("expect(\"a string\").not.toEqual(NaN);", None),
        ("expect(\"a string\").not.toStrictEqual(NaN);", None),
        // undefined vs defined
        ("expect(undefined).not.toBeDefined();", None),
        ("expect(undefined).resolves.not.toBeDefined();", None),
        ("expect(undefined).resolves.toBe(undefined);", None),
        ("expect(\"a string\").not.toBeUndefined();", None),
        ("expect(\"a string\").rejects.not.toBeUndefined();", None),
        // typescript edition
        ("expect(null).toEqual(1 as unknown as string as unknown as any);", None),
        ("expect(null).toEqual(-1 as unknown as string as unknown as any);", None),
        ("expect(\"a string\").not.toStrictEqual(\"string\" as number);", None),
        ("expect(null).toBe(null as unknown as string as unknown as any);", None),
        ("expect(\"a string\").not.toEqual(null as number);", None),
        ("expect(undefined).toBe(undefined as unknown as string as any);", None),
        ("expect(\"a string\").toEqual(undefined as number);", None),
    ];

    let fix = vec![
        ("expect(value).toEqual(\"my string\");", "expect(value).toBe(\"my string\");", None),
        ("expect(value).toStrictEqual(\"my string\");", "expect(value).toBe(\"my string\");", None),
        ("expect(value).toStrictEqual(1);", "expect(value).toBe(1);", None),
        ("expect(value).toStrictEqual(1,);", "expect(value).toBe(1,);", None),
        ("expect(value).toStrictEqual(-1);", "expect(value).toBe(-1);", None),
        ("expect(value).toEqual(`my string`);", "expect(value).toBe(`my string`);", None),
        ("expect(value)[\"toEqual\"](`my string`);", "expect(value)[\"toBe\"](`my string`);", None),
        (
            "expect(value).toStrictEqual(`my ${string}`);",
            "expect(value).toBe(`my ${string}`);",
            None,
        ),
        (
            "expect(loadMessage()).resolves.toStrictEqual(\"hello world\");",
            "expect(loadMessage()).resolves.toBe(\"hello world\");",
            None,
        ),
        (
            "expect(loadMessage()).resolves[\"toStrictEqual\"](\"hello world\");",
            "expect(loadMessage()).resolves[\"toBe\"](\"hello world\");",
            None,
        ),
        (
            "expect(loadMessage())[\"resolves\"].toStrictEqual(\"hello world\");",
            "expect(loadMessage())[\"resolves\"].toBe(\"hello world\");",
            None,
        ),
        (
            "expect(loadMessage()).resolves.toStrictEqual(false);",
            "expect(loadMessage()).resolves.toBe(false);",
            None,
        ),
        // null
        ("expect(null).toBe(null);", "expect(null).toBeNull();", None),
        ("expect(null).toEqual(null);", "expect(null).toBeNull();", None),
        ("expect(null).toEqual(null,);", "expect(null).toBeNull();", None),
        ("expect(null).toStrictEqual(null);", "expect(null).toBeNull();", None),
        ("expect(\"a string\").not.toBe(null);", "expect(\"a string\").not.toBeNull();", None),
        (
            "expect(\"a string\").not[\"toBe\"](null);",
            "expect(\"a string\").not[\"toBeNull\"]();",
            None,
        ),
        (
            "expect(\"a string\")[\"not\"][\"toBe\"](null);",
            "expect(\"a string\")[\"not\"][\"toBeNull\"]();",
            None,
        ),
        ("expect(\"a string\").not.toEqual(null);", "expect(\"a string\").not.toBeNull();", None),
        (
            "expect(\"a string\").not.toStrictEqual(null);",
            "expect(\"a string\").not.toBeNull();",
            None,
        ),
        // undefined
        ("expect(undefined).toBe(undefined);", "expect(undefined).toBeUndefined();", None),
        ("expect(undefined).toEqual(undefined);", "expect(undefined).toBeUndefined();", None),
        ("expect(undefined).toStrictEqual(undefined);", "expect(undefined).toBeUndefined();", None),
        ("expect(\"a string\").not.toBe(undefined);", "expect(\"a string\").toBeDefined();", None),
        (
            "expect(\"a string\").rejects.not.toBe(undefined);",
            "expect(\"a string\").rejects.toBeDefined();",
            None,
        ),
        (
            "expect(\"a string\").rejects.not[\"toBe\"](undefined);",
            "expect(\"a string\").rejects[\"toBeDefined\"]();",
            None,
        ),
        (
            "expect(\"a string\").not.toEqual(undefined);",
            "expect(\"a string\").toBeDefined();",
            None,
        ),
        (
            "expect(\"a string\").not.toStrictEqual(undefined);",
            "expect(\"a string\").toBeDefined();",
            None,
        ),
        // NaN
        ("expect(NaN).toBe(NaN);", "expect(NaN).toBeNaN();", None),
        ("expect(NaN).toEqual(NaN);", "expect(NaN).toBeNaN();", None),
        ("expect(NaN).toStrictEqual(NaN);", "expect(NaN).toBeNaN();", None),
        ("expect(\"a string\").not.toBe(NaN);", "expect(\"a string\").not.toBeNaN();", None),
        (
            "expect(\"a string\").rejects.not.toBe(NaN);",
            "expect(\"a string\").rejects.not.toBeNaN();",
            None,
        ),
        (
            "expect(\"a string\")[\"rejects\"].not.toBe(NaN);",
            "expect(\"a string\")[\"rejects\"].not.toBeNaN();",
            None,
        ),
        ("expect(\"a string\").not.toEqual(NaN);", "expect(\"a string\").not.toBeNaN();", None),
        (
            "expect(\"a string\").not.toStrictEqual(NaN);",
            "expect(\"a string\").not.toBeNaN();",
            None,
        ),
        // undefined vs defined
        ("expect(undefined).not.toBeDefined();", "expect(undefined).toBeUndefined();", None),
        (
            "expect(undefined).resolves.not.toBeDefined();",
            "expect(undefined).resolves.toBeUndefined();",
            None,
        ),
        (
            "expect(undefined).resolves.toBe(undefined);",
            "expect(undefined).resolves.toBeUndefined();",
            None,
        ),
        ("expect(\"a string\").not.toBeUndefined();", "expect(\"a string\").toBeDefined();", None),
        (
            "expect(\"a string\").rejects.not.toBeUndefined();",
            "expect(\"a string\").rejects.toBeDefined();",
            None,
        ),
        // typescript edition
        (
            "expect(null).toEqual(1 as unknown as string as unknown as any);",
            "expect(null).toBe(1 as unknown as string as unknown as any);",
            None,
        ),
        (
            "expect(null).toEqual(-1 as unknown as string as unknown as any);",
            "expect(null).toBe(-1 as unknown as string as unknown as any);",
            None,
        ),
        (
            "expect(\"a string\").not.toStrictEqual(\"string\" as number);",
            "expect(\"a string\").not.toBe(\"string\" as number);",
            None,
        ),
        (
            "expect(null).toBe(null as unknown as string as unknown as any);",
            "expect(null).toBeNull();",
            None,
        ),
        (
            "expect(\"a string\").not.toEqual(null as number);",
            "expect(\"a string\").not.toBeNull();",
            None,
        ),
        (
            "expect(undefined).toBe(undefined as unknown as string as any);",
            "expect(undefined).toBeUndefined();",
            None,
        ),
        (
            "expect(\"a string\").toEqual(undefined as number);",
            "expect(\"a string\").toBeUndefined();",
            None,
        ),
    ];

    Tester::new(PreferToBe::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
