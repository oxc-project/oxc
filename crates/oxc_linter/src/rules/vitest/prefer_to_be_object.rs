use oxc_ast::{
    AstKind,
    ast::{Argument, BinaryOperator, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        KnownMemberExpressionProperty, ParsedExpectFnCall, ParsedJestFnCallNew, PossibleJestNode,
        parse_jest_fn_call,
    },
};

fn prefer_to_be_object(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `toBeObject()` for object assertions")
        .with_help("Consider using `toBeObject()` to test if a value is an object.")
        .with_label(span)
}

fn is_parsed_instance_of_matcher_call(
    parsed_expect_call: &ParsedExpectFnCall,
    matcher: &KnownMemberExpressionProperty,
) -> bool {
    parsed_expect_call.args.len() == 1
        && matches!(
            parsed_expect_call.args.first(),
            Some(Argument::Identifier(id)) if matcher.name().as_deref() == Some("toBeInstanceOf") && id.name == "Object"
        )
}

#[derive(Debug, Default, Clone)]
pub struct PreferToBeObject;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces using `toBeObject()` to check if a value is of type `Object`.
    ///
    /// ### Why is this bad?
    ///
    /// Using other methods such as `toBeInstanceOf(Object)` or `instanceof Object` can
    /// be less clear and potentially misleading. Enforcing the use of `toBeObject()`
    /// provides more explicit and readable code, making your intentions clear and
    /// improving the overall maintainability and readability of your tests.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// expectTypeOf({}).toBeInstanceOf(Object);
    /// expectTypeOf({} instanceof Object).toBeTruthy();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// expectTypeOf({}).toBeObject();
    /// expectTypeOf({}).toBeObject();
    /// ```
    PreferToBeObject,
    vitest,
    style,
    fix
);

impl Rule for PreferToBeObject {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl PreferToBeObject {
    fn run<'a>(possible_vitest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_vitest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(ParsedJestFnCallNew::ExpectTypeOf(parsed_expect_call)) =
            parse_jest_fn_call(call_expr, possible_vitest_node, ctx)
        else {
            return;
        };

        let Some(matcher) = parsed_expect_call.matcher() else {
            return;
        };

        if is_parsed_instance_of_matcher_call(&parsed_expect_call, matcher) {
            ctx.diagnostic_with_fix(prefer_to_be_object(matcher.span), |fixer| {
                fixer.replace(Span::new(matcher.span.start, call_expr.span.end), "toBeObject()")
            });
            return;
        }

        if matches!(matcher.name().as_deref(), Some("toBeTruthy" | "toBeFalsy")) {
            let Some(Expression::CallExpression(parent_call_expr)) = parsed_expect_call.head.parent
            else {
                return;
            };

            let Some(arg) = parent_call_expr.arguments.first() else {
                return;
            };

            let expr = match &arg {
                Argument::ParenthesizedExpression(paren_expr) => &paren_expr.expression,
                _ => arg.to_expression(),
            };

            let Expression::BinaryExpression(binary_expr) = expr else {
                return;
            };

            if binary_expr.operator != BinaryOperator::Instanceof {
                return;
            }

            let Expression::Identifier(id) = &binary_expr.right else {
                return;
            };

            if id.name == "Object" {
                ctx.diagnostic_with_fix(prefer_to_be_object(matcher.span), |fixer| {
                    let code = {
                        let not_modifier = parsed_expect_call
                            .modifiers()
                            .iter()
                            .any(|node| node.name().as_deref() == Some("not"));
                        let is_not_modifier =
                            (matcher.name().as_deref() == Some("toBeFalsy")) != not_modifier;

                        let left = fixer.source_range(Span::new(
                            call_expr.span.start,
                            binary_expr.left.span().end,
                        ));
                        let right = fixer.source_range(Span::new(
                            binary_expr.span.end,
                            parent_call_expr.span.end,
                        ));

                        let mut code = String::with_capacity(
                            left.len() + right.len() + if is_not_modifier { 4 } else { 0 } + 13,
                        );

                        code.push_str(left);
                        code.push_str(right);

                        if is_not_modifier {
                            code.push_str(".not");
                        }

                        code.push_str(".toBeObject()");
                        code
                    };
                    fixer.replace(call_expr.span, code)
                });
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "expectTypeOf.hasAssertions",
        "expectTypeOf.hasAssertions()",
        "expectTypeOf",
        "expectTypeOf().not",
        "expectTypeOf().toBe",
        "expectTypeOf().toBe(true)",
        "expectTypeOf({}).toBe(true)",
        "expectTypeOf({}).toBeObject()",
        "expectTypeOf({}).not.toBeObject()",
        "expectTypeOf([] instanceof Array).not.toBeObject()",
        "expectTypeOf({}).not.toBeInstanceOf(Array)",
    ];

    let fail = vec![
        // "expectTypeOf(({} instanceof Object)).toBeTruthy();",
        "expectTypeOf({} instanceof Object).toBeTruthy();",
        "expectTypeOf({} instanceof Object).not.toBeTruthy();",
        "expectTypeOf({} instanceof Object).toBeFalsy();",
        "expectTypeOf({} instanceof Object).not.toBeFalsy();",
        "expectTypeOf({}).toBeInstanceOf(Object);",
        "expectTypeOf({}).not.toBeInstanceOf(Object);",
        "expectTypeOf(requestValues()).resolves.toBeInstanceOf(Object);",
        "expectTypeOf(queryApi()).resolves.not.toBeInstanceOf(Object);",
    ];

    let fix = vec![
        (
            "expectTypeOf(({} instanceof Object)).toBeTruthy();",
            "expectTypeOf(({})).toBeObject();",
            None,
        ),
        (
            "expectTypeOf({} instanceof Object).toBeTruthy();",
            "expectTypeOf({}).toBeObject();",
            None,
        ),
        (
            "expectTypeOf({} instanceof Object).not.toBeTruthy();",
            "expectTypeOf({}).not.toBeObject();",
            None,
        ),
        (
            "expectTypeOf({} instanceof Object).toBeFalsy();",
            "expectTypeOf({}).not.toBeObject();",
            None,
        ),
        (
            "expectTypeOf({} instanceof Object).not.toBeFalsy();",
            "expectTypeOf({}).toBeObject();",
            None,
        ),
        ("expectTypeOf({}).toBeInstanceOf(Object);", "expectTypeOf({}).toBeObject();", None),
        (
            "expectTypeOf({}).not.toBeInstanceOf(Object);",
            "expectTypeOf({}).not.toBeObject();",
            None,
        ),
        (
            "expectTypeOf(requestValues()).resolves.toBeInstanceOf(Object);",
            "expectTypeOf(requestValues()).resolves.toBeObject();",
            None,
        ),
        (
            "expectTypeOf(queryApi()).resolves.not.toBeInstanceOf(Object);",
            "expectTypeOf(queryApi()).resolves.not.toBeObject();",
            None,
        ),
    ];
    Tester::new(PreferToBeObject::NAME, PreferToBeObject::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
