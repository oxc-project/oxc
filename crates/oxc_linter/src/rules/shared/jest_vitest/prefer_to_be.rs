use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    utils::{
        KnownMemberExpressionProperty, ParsedExpectFnCall, PossibleJestNode, is_equality_matcher,
        parse_expect_jest_fn_call,
    },
};

fn use_to_be(source_text: &str, suggestion: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBe` when expecting primitive literals.")
        .with_help(format!("Replace `{source_text}` with `{suggestion}`."))
        .with_label(span)
}

fn use_to_be_undefined(source_text: &str, suggestion: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeUndefined` instead.")
        .with_help(format!("Replace `{source_text}` with `{suggestion}`."))
        .with_label(span)
}

fn use_to_be_defined(source_text: &str, suggestion: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeDefined` instead.")
        .with_help(format!("Replace `{source_text}` with `{suggestion}`."))
        .with_label(span)
}

fn use_to_be_null(source_text: &str, suggestion: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeNull` instead.")
        .with_help(format!("Replace `{source_text}` with `{suggestion}`."))
        .with_label(span)
}

fn use_to_be_na_n(source_text: &str, suggestion: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeNaN` instead.")
        .with_help(format!("Replace `{source_text}` with `{suggestion}`."))
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Recommends using `toBe` matcher for primitive literals and specific
matchers for `null`, `undefined`, and `NaN`.

### Why is this bad?

When asserting against primitive literals such as numbers and strings,
the equality matchers all operate the same, but read slightly
differently in code.

This rule recommends using the `toBe` matcher in these situations, as
it forms the most grammatically natural sentence. For `null`,
`undefined`, and `NaN` this rule recommends using their specific `toBe`
matchers, as they give better error messages as well.

### Examples

Examples of **incorrect** code for this rule:
```javascript
expect(value).not.toEqual(5);
expect(getMessage()).toStrictEqual('hello world');
expect(loadMessage()).resolves.toEqual('hello world');
```

Examples of **correct** code for this rule:
```javascript
expect(value).not.toBe(5);
expect(getMessage()).toBe('hello world');
expect(loadMessage()).resolves.toBe('hello world');
expect(didError).not.toBe(true);
expect(catchError()).toStrictEqual({ message: 'oh noes!' });
```
";

#[derive(Clone, Debug, PartialEq)]
enum PreferToBeKind {
    Defined,
    NaN,
    Null,
    ToBe,
    Undefined,
}

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
    PreferToBe::run(node, ctx);
}

struct PreferToBe;

impl PreferToBe {
    fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(possible_jest_node) = PossibleJestNode::from_call_expression(node, call_expr, ctx)
        else {
            return;
        };
        let Some(jest_expect_fn_call) =
            parse_expect_jest_fn_call(call_expr, &possible_jest_node, ctx)
        else {
            return;
        };
        let Some(matcher) = jest_expect_fn_call.matcher() else {
            return;
        };

        let has_not_modifier =
            jest_expect_fn_call.modifiers().any(|modifier| modifier.is_name_equal("not"));

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

        if !is_equality_matcher(matcher) || jest_expect_fn_call.args.is_empty() {
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

        if Self::should_use_tobe(first_matcher_arg)
            && !matcher.is_name_equal("toBe")
            && !Self::should_skip_float(first_matcher_arg, ctx)
        {
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

        if let Expression::UnaryExpression(unary_expr) = expr
            && unary_expr.operator.as_str() == "-"
        {
            expr = &unary_expr.argument;
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

    fn should_skip_float(expr: &Expression, ctx: &LintContext) -> bool {
        // Check if this is a float literal by examining the source text
        if let Expression::NumericLiteral(num) = expr {
            let source = ctx.source_range(num.span);
            return source.contains('.');
        }
        false
    }

    /// Helper function to build suggestion for matchers that keep the "not" modifier (null, NaN).
    /// Returns (source_start, suggestion_string).
    fn build_suggestion_with_not_modifier(
        matcher_name: &str,
        not_modifier: Option<&KnownMemberExpressionProperty>,
        is_cmp_mem_expr: bool,
        matcher: &KnownMemberExpressionProperty,
        call_expr: &CallExpression,
        ctx: &LintContext,
    ) -> Option<(u32, String)> {
        let first_member = not_modifier.unwrap_or(matcher);
        let start = if let Some(Expression::ComputedMemberExpression(member)) = first_member.parent
        {
            let object_end = member.object.span().end;
            object_end + ctx.find_next_token_within(object_end, first_member.span.start, "[")?
        } else {
            first_member.span.start
        };

        let (not_prefix, member_operator) = if let Some(not_modifier) = not_modifier {
            let prefix =
                if matches!(not_modifier.parent, Some(Expression::ComputedMemberExpression(_))) {
                    r#"["not"]"#
                } else {
                    "not"
                };
            let operator = if matcher.parent?.as_member_expression()?.optional() {
                "?."
            } else if is_cmp_mem_expr {
                ""
            } else {
                "."
            };
            (prefix, operator)
        } else {
            ("", "")
        };
        let call_operator = if call_expr.optional { "?." } else { "" };
        let suggestion = if is_cmp_mem_expr {
            format!(r#"{not_prefix}{member_operator}["{matcher_name}"]{call_operator}()"#)
        } else {
            format!("{not_prefix}{member_operator}{matcher_name}{call_operator}()")
        };
        Some((start, suggestion))
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

        let maybe_not_modifier =
            jest_expect_fn_call.modifiers().find(|modifier| modifier.is_name_equal("not"));

        if kind == &PreferToBeKind::Undefined {
            let replacement_span = if let Some(not_modifier) = maybe_not_modifier {
                Span::new(not_modifier.span.start, end)
            } else {
                Span::new(span.start, end)
            };
            let source_text = ctx.source_range(replacement_span);
            let new_matcher =
                if is_cmp_mem_expr { "[\"toBeUndefined\"]()" } else { "toBeUndefined()" };

            ctx.diagnostic_with_suggestion(
                use_to_be_undefined(source_text, new_matcher, replacement_span),
                |fixer| fixer.replace(replacement_span, new_matcher),
            );
        } else if kind == &PreferToBeKind::Defined {
            let start = if is_cmp_mem_expr {
                jest_expect_fn_call.modifiers().next().unwrap().span.end
            } else {
                maybe_not_modifier.unwrap().span.start
            };
            let replacement_span = Span::new(start, end);
            let source_text = ctx.source_range(replacement_span);
            let new_matcher = if is_cmp_mem_expr { "[\"toBeDefined\"]()" } else { "toBeDefined()" };

            ctx.diagnostic_with_suggestion(
                use_to_be_defined(source_text, new_matcher, replacement_span),
                |fixer| fixer.replace(replacement_span, new_matcher),
            );
        } else if kind == &PreferToBeKind::Null {
            let Some((source_start, suggestion)) = Self::build_suggestion_with_not_modifier(
                "toBeNull",
                maybe_not_modifier,
                is_cmp_mem_expr,
                matcher,
                call_expr,
                ctx,
            ) else {
                return;
            };

            let replacement_span = Span::new(source_start, end);
            let source_text = ctx.source_range(replacement_span);

            ctx.diagnostic_with_suggestion(
                use_to_be_null(source_text, &suggestion, replacement_span),
                |fixer| fixer.replace(replacement_span, suggestion),
            );
        } else if kind == &PreferToBeKind::NaN {
            let Some((source_start, suggestion)) = Self::build_suggestion_with_not_modifier(
                "toBeNaN",
                maybe_not_modifier,
                is_cmp_mem_expr,
                matcher,
                call_expr,
                ctx,
            ) else {
                return;
            };

            let replacement_span = Span::new(source_start, end);
            let source_text = ctx.source_range(replacement_span);

            ctx.diagnostic_with_suggestion(
                use_to_be_na_n(source_text, &suggestion, replacement_span),
                |fixer| fixer.replace(replacement_span, suggestion),
            );
        } else {
            let source_text = ctx.source_range(span);
            let new_matcher = if is_cmp_mem_expr { "\"toBe\"" } else { "toBe" };

            ctx.diagnostic_with_suggestion(use_to_be(source_text, new_matcher, span), |fixer| {
                fixer.replace(span, new_matcher)
            });
        }
    }
}

#[test]
fn test_computed_matcher_trivia() {
    use crate::{fixer::FixKind, tester::Tester};

    let fix: Vec<_> = vec![
        ("expect(value)[\u{a0}\"toEqual\"](null);", "expect(value)[\"toBeNull\"]();"),
        ("expect(value)[\u{2003}\"toEqual\"](NaN);", "expect(value)[\"toBeNaN\"]();"),
        ("expect(value)[ \"toEqual\" ](null);", "expect(value)[\"toBeNull\"]();"),
        (
            "expect(value) /* [ */ [ /* [ ] */ \"toEqual\" /* ] */ ](NaN);",
            "expect(value) /* [ */ [\"toBeNaN\"]();",
        ),
        ("expect(value)[ // [\n \"toEqual\"\n](null);", "expect(value)[\"toBeNull\"]();"),
        ("expect(value)[\u{a0}\"not\"].toEqual(null);", "expect(value)[\"not\"].toBeNull();"),
        (
            "expect(value)[\u{2003}\"not\"][\u{a0}\"toEqual\"](NaN);",
            "expect(value)[\"not\"][\"toBeNaN\"]();",
        ),
        ("expect(value).not[\u{a0}\"toEqual\"](null);", "expect(value).not[\"toBeNull\"]();"),
        (
            "expect(value).resolves /* [ */ [ /* [ ] */ \"not\" ].toEqual(null);",
            "expect(value).resolves /* [ */ [\"not\"].toBeNull();",
        ),
        (
            "expect(value).rejects[\u{a0}`not`][\u{2003}`toStrictEqual`](NaN);",
            "expect(value).rejects[\"not\"][\"toBeNaN\"]();",
        ),
        ("expect(value)[\u{a0}`toEqual`](null);", "expect(value)[\"toBeNull\"]();"),
        ("expect(value)?.[\u{a0}\"toEqual\"](null);", "expect(value)?.[\"toBeNull\"]();"),
        ("expect(value)[\u{a0}\"toEqual\"]?.(null);", "expect(value)[\"toBeNull\"]?.();"),
        ("expect(value)[\u{a0}\"not\"]?.toEqual(null);", "expect(value)[\"not\"]?.toBeNull();"),
        (
            "expect(value)[\u{a0}\"not\"]?.[\"toEqual\"]?.(NaN);",
            "expect(value)[\"not\"]?.[\"toBeNaN\"]?.();",
        ),
        ("expect(value).not?.[\u{a0}\"toEqual\"](null);", "expect(value).not?.[\"toBeNull\"]();"),
        ("expect(value)?.[\u{a0}\"not\"].toEqual?.(NaN);", "expect(value)?.[\"not\"].toBeNaN?.();"),
    ]
    .into_iter()
    .map(|(source, expected)| (source, expected, None, FixKind::Suggestion))
    .collect();

    for plugin in ["jest", "vitest"] {
        Tester::new(
            "prefer-to-be",
            plugin,
            fix.iter().map(|(_, fixed, _, _)| *fixed).collect(),
            fix.iter().map(|(source, _, _, _)| *source).collect(),
        )
        .with_jest_plugin(plugin == "jest")
        .with_vitest_plugin(plugin == "vitest")
        .expect_fix(fix.clone())
        .test();
    }
}
