use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

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

pub fn run_on_jest_node<'a, 'c>(jest_node: &PossibleJestNode<'a, 'c>, ctx: &'c LintContext<'a>) {
    PreferToBe::run(jest_node, ctx);
}

struct PreferToBe;

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
        not_modifier: Option<&&KnownMemberExpressionProperty>,
        is_cmp_mem_expr: bool,
        span_start: u32,
    ) -> (u32, String) {
        if let Some(&not_modifier) = not_modifier {
            let not_is_computed =
                matches!(not_modifier.parent, Some(Expression::ComputedMemberExpression(_)));

            if not_is_computed {
                // ["not"]["toBe"](value) -> ["not"]["toBeMatcher"]()
                let start = not_modifier.span.start - 1; // Include opening bracket of ["not"]
                let suggestion = if is_cmp_mem_expr {
                    format!("[\"not\"][\"{matcher_name}\"]()")
                } else {
                    format!("[\"not\"].{matcher_name}()")
                };
                (start, suggestion)
            } else if is_cmp_mem_expr {
                // .not["toBe"](value) -> .not["toBeMatcher"]()
                (not_modifier.span.start, format!("not[\"{matcher_name}\"]()"))
            } else {
                // .not.toBe(value) -> .not.toBeMatcher()
                (not_modifier.span.start, format!("not.{matcher_name}()"))
            }
        } else {
            // No "not" modifier
            let start = if is_cmp_mem_expr { span_start - 1 } else { span_start };
            let suggestion = if is_cmp_mem_expr {
                format!("[\"{matcher_name}\"]()")
            } else {
                format!("{matcher_name}()")
            };
            (start, suggestion)
        }
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
            let replacement_span = if let Some(not_modifier) = maybe_not_modifier {
                Span::new(not_modifier.span.start, end)
            } else {
                Span::new(span.start, end)
            };
            let source_text = ctx.source_range(replacement_span);
            let new_matcher =
                if is_cmp_mem_expr { "[\"toBeUndefined\"]()" } else { "toBeUndefined()" };

            ctx.diagnostic_with_fix(
                use_to_be_undefined(source_text, new_matcher, replacement_span),
                |fixer| fixer.replace(replacement_span, new_matcher),
            );
        } else if kind == &PreferToBeKind::Defined {
            let start = if is_cmp_mem_expr {
                modifiers.first().unwrap().span.end
            } else {
                maybe_not_modifier.unwrap().span.start
            };
            let replacement_span = Span::new(start, end);
            let source_text = ctx.source_range(replacement_span);
            let new_matcher = if is_cmp_mem_expr { "[\"toBeDefined\"]()" } else { "toBeDefined()" };

            ctx.diagnostic_with_fix(
                use_to_be_defined(source_text, new_matcher, replacement_span),
                |fixer| fixer.replace(replacement_span, new_matcher),
            );
        } else if kind == &PreferToBeKind::Null {
            let (source_start, suggestion) = Self::build_suggestion_with_not_modifier(
                "toBeNull",
                maybe_not_modifier,
                is_cmp_mem_expr,
                span.start,
            );

            let replacement_span = Span::new(source_start, end);
            let source_text = ctx.source_range(replacement_span);

            ctx.diagnostic_with_fix(
                use_to_be_null(source_text, &suggestion, replacement_span),
                |fixer| fixer.replace(replacement_span, suggestion),
            );
        } else if kind == &PreferToBeKind::NaN {
            let (source_start, suggestion) = Self::build_suggestion_with_not_modifier(
                "toBeNaN",
                maybe_not_modifier,
                is_cmp_mem_expr,
                span.start,
            );

            let replacement_span = Span::new(source_start, end);
            let source_text = ctx.source_range(replacement_span);

            ctx.diagnostic_with_fix(
                use_to_be_na_n(source_text, &suggestion, replacement_span),
                |fixer| fixer.replace(replacement_span, suggestion),
            );
        } else {
            let source_text = ctx.source_range(span);
            let new_matcher = if is_cmp_mem_expr { "\"toBe\"" } else { "toBe" };

            ctx.diagnostic_with_fix(use_to_be(source_text, new_matcher, span), |fixer| {
                fixer.replace(span, new_matcher)
            });
        }
    }
}
