use oxc_ast::{
    AstKind,
    ast::{ArrayExpressionElement, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};
use oxc_str::static_ident;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_array_to_reversed_diagnostic(span: Span, replacement: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `{replacement}` instead of copying and reversing."))
        .with_help(
            "Copying and then calling `reverse()` allocates an extra array. `toReversed()` returns a reversed copy in one step.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferArrayToReversed;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `Array#toReversed()` over copying an array and then calling `reverse()`.
    ///
    /// ### Why is this bad?
    ///
    /// Copying an array (with `[...array]`, `array.slice()`, `array.slice(0)` or
    /// `array.concat()`) and then reversing the copy creates an extra array.
    /// `toReversed()` returns a reversed copy in one step and is easier to read.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const a = [...array].reverse();
    /// const b = array.slice().reverse();
    /// const c = array.concat().reverse();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const a = array.toReversed();
    /// ```
    ///
    /// ### Notes
    ///
    /// Without type information, the rule cannot know that the copied value is an array.
    /// Sets, Maps, strings and other iterables can be spread with `[...value]` but have no
    /// `toReversed()` method. The rule therefore only applies an automatic fix when the value
    /// is clearly an array: an array literal, `Array.from()`, `Array.of()`, `Object.keys()`,
    /// `Object.values()` or `Object.entries()`. In all other cases it offers a suggestion.
    ///
    /// `toReversed()` requires ES2023 (Node.js 20+ and modern browsers).
    ///
    /// `unicorn/no-array-reverse` also reports `[...array].reverse()`, but its suggestion
    /// (`[...array].toReversed()`) keeps the copy. This rule removes the copy.
    PreferArrayToReversed,
    oxc,
    perf,
    conditional_fix_suggestion,
    version = "next",
    short_description = "Prefer `Array#toReversed()` over copying and reversing arrays.",
);

impl Rule for PreferArrayToReversed {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !call_expr.arguments.is_empty() || call_expr.optional {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };
        let Some((_, static_property_name)) = member_expr.static_property_info() else {
            return;
        };
        if static_property_name != "reverse" {
            return;
        }

        let Some((source, source_optional)) = get_copy_source(member_expr.object()) else {
            return;
        };

        let call_span = call_expr.span;
        let source_span = source.span();
        // Build the replacement once and use it in both the message and the fix, so the
        // message shows exactly the code the fix produces, e.g. `(a ?? b).toReversed()`.
        let replacement = build_replacement(source, source_optional, ctx);
        let diagnostic = prefer_array_to_reversed_diagnostic(call_span, &replacement);

        // The fix keeps the source text and replaces everything around it.
        // Don't offer a fix if that would delete comments outside the source expression.
        if ctx.has_comments_between(Span::new(call_span.start, source_span.start))
            || ctx.has_comments_between(Span::new(source_span.end, call_span.end))
        {
            ctx.diagnostic(diagnostic);
            return;
        }

        if is_safe_fix(source, ctx) {
            ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace(call_span, replacement));
        } else {
            ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                fixer.replace(call_span, replacement)
            });
        }
    }
}

/// If `expr` is a copy pattern (`[...x]`, `x.slice()`, `x.slice(0)` or `x.concat()`),
/// returns `x` and whether the copy method was accessed with optional chaining (`x?.slice()`).
///
/// Only parentheses are stripped. TypeScript wrappers such as `as` and `!` are kept, so
/// the fix doesn't drop them.
fn get_copy_source<'a>(expr: &'a Expression<'a>) -> Option<(&'a Expression<'a>, bool)> {
    let expr = expr.without_parentheses();

    if let Expression::ArrayExpression(array) = expr {
        if array.elements.len() == 1
            && let ArrayExpressionElement::SpreadElement(spread) = &array.elements[0]
        {
            return Some((spread.argument.without_parentheses(), false));
        }
        return None;
    }

    let Expression::CallExpression(copy_call) = expr else {
        return None;
    };
    if copy_call.optional {
        return None;
    }
    let copy_member = copy_call.callee.get_member_expr()?;
    let (_, method_name) = copy_member.static_property_info()?;

    // `method_name` is a `JsStr`. It supports `==` with `&str`, but can't be matched
    // against string literal patterns, so compare with `==` instead of `match`.
    let is_copy = if method_name == "concat" {
        copy_call.arguments.is_empty()
    } else if method_name == "slice" {
        copy_call.arguments.is_empty()
            || (copy_call.arguments.len() == 1
                && copy_call.arguments[0].as_expression().is_some_and(|arg| {
                    matches!(
                        arg.without_parentheses(),
                        Expression::NumericLiteral(lit) if lit.value == 0.0
                    )
                }))
    } else {
        false
    };

    if is_copy {
        Some((copy_member.object().without_parentheses(), copy_member.optional()))
    } else {
        None
    }
}

/// Returns `true` only when `expr` is syntactically guaranteed to be an array.
fn is_safe_fix(expr: &Expression, ctx: &LintContext) -> bool {
    match expr {
        Expression::ArrayExpression(_) => true,
        Expression::CallExpression(call_expr) => {
            if call_expr.optional {
                return false;
            }
            let Some(member_expr) = call_expr.callee.get_member_expr() else {
                return false;
            };
            if member_expr.optional() {
                return false;
            }
            let Some((_, method_name)) = member_expr.static_property_info() else {
                return false;
            };
            let Expression::Identifier(ident) = member_expr.object().without_parentheses() else {
                return false;
            };
            if method_name == "from" || method_name == "of" {
                ident.is_global_reference_name(static_ident!("Array"), ctx.scoping())
            } else if method_name == "keys" || method_name == "values" || method_name == "entries" {
                ident.is_global_reference_name(static_ident!("Object"), ctx.scoping())
            } else {
                false
            }
        }
        _ => false,
    }
}

fn build_replacement(expr: &Expression, source_optional: bool, ctx: &LintContext) -> String {
    let source_text = ctx.source_range(expr.span());
    let accessor = if source_optional { "?." } else { "." };
    if needs_parens_for_property_access(expr) {
        format!("({source_text}){accessor}toReversed()")
    } else {
        format!("{source_text}{accessor}toReversed()")
    }
}

/// Whether `expr` must be wrapped in parentheses before `.toReversed()` can be appended.
///
/// `NumericLiteral` is deliberately not in the list: `5.toReversed()` is a syntax error.
/// `NewExpression` is not in the list: `new Foo.toReversed()` parses differently.
/// `ChainExpression` is not in the list: `obj?.arr.toReversed()` would short-circuit to
/// `undefined` when `obj` is nullish, while the original `[...obj?.arr]` throws. Wrapping it
/// as `(obj?.arr).toReversed()` keeps the original behavior.
fn needs_parens_for_property_access(expr: &Expression) -> bool {
    !matches!(
        expr,
        Expression::Identifier(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::PrivateFieldExpression(_)
            | Expression::CallExpression(_)
            | Expression::BigIntLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::TaggedTemplateExpression(_)
            | Expression::ThisExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::ParenthesizedExpression(_)
            | Expression::ImportMeta(_)
            | Expression::NewTarget(_)
    )
}

#[test]
fn test() {
    use crate::{fixer::FixKind, tester::Tester};

    let pass = vec![
        "arr.reverse()",
        "arr.toReversed()",
        "arr.slice(1).reverse()",
        "arr.slice(0, 5).reverse()",
        "arr.slice(\"0\").reverse()",
        "arr.concat(other).reverse()",
        "arr.slice?.().reverse()",
        "arr.concat?.().reverse()",
        "arr.filter(f).reverse()",
        "arr.map(f).reverse()",
        "[a, b].reverse()",
        "[...a, ...b].reverse()",
        "[...a, 1].reverse()",
        "[...arr].reverse",
        "[...arr].reverse?.()",
        "[...arr].reverse(1)",
        "const reversed = [...arr].reverse?.();",
    ];

    let fail = vec![
        "const reversed = arr.slice().reverse();",
        "const reversed = arr.slice(0).reverse();",
        "const reversed = arr.concat().reverse();",
        "const reversed = [...arr].reverse();",
        "const reversed = [...obj.arr].reverse();",
        "const reversed = obj.arr.slice().reverse();",
        "const reversed = obj.arr.concat().reverse();",
        "const reversed = someFunc().slice(0).reverse();",
        "const reversed = [...a ?? b].reverse();",
        "const reversed = [...a || b].reverse();",
        "const reversed = [...a ? b : c].reverse();",
        "obj.arr?.slice().reverse();",
        "obj.arr?.concat().reverse();",
        "arr.concat().reverse();",
        "console.log(arr.slice().reverse());",
        "[...Array.from(x)].reverse()",
        "Object.keys(o).slice().reverse()",
        "[...[1, 2, 3]].reverse()",
        "[...new Foo].reverse()",
        "[...(0, array)].reverse()",
        "[...await x].reverse()",
        "[...mySet].reverse()",
        "[...(arr as number[])].reverse()",
        "[...obj?.arr].reverse()",
        "const Array = { from: () => new Set() }; [...Array.from(x)].reverse()",
        r#"[...str].reverse().join("")"#,
        "[/* keep */ ...arr].reverse()",
    ];

    let fix = vec![
        (
            "const reversed = arr.slice().reverse();",
            "const reversed = arr.toReversed();",
            None,
            FixKind::Suggestion,
        ),
        (
            "const reversed = arr.slice(0).reverse();",
            "const reversed = arr.toReversed();",
            None,
            FixKind::Suggestion,
        ),
        (
            "const reversed = arr.concat().reverse();",
            "const reversed = arr.toReversed();",
            None,
            FixKind::Suggestion,
        ),
        (
            "const reversed = [...arr].reverse();",
            "const reversed = arr.toReversed();",
            None,
            FixKind::Suggestion,
        ),
        (
            "const reversed = [...obj.arr].reverse();",
            "const reversed = obj.arr.toReversed();",
            None,
            FixKind::Suggestion,
        ),
        (
            "const reversed = obj.arr.slice().reverse();",
            "const reversed = obj.arr.toReversed();",
            None,
            FixKind::Suggestion,
        ),
        (
            "const reversed = obj.arr.concat().reverse();",
            "const reversed = obj.arr.toReversed();",
            None,
            FixKind::Suggestion,
        ),
        (
            "const reversed = someFunc().slice(0).reverse();",
            "const reversed = someFunc().toReversed();",
            None,
            FixKind::Suggestion,
        ),
        (
            "const reversed = [...a ?? b].reverse();",
            "const reversed = (a ?? b).toReversed();",
            None,
            FixKind::Suggestion,
        ),
        (
            "const reversed = [...a || b].reverse();",
            "const reversed = (a || b).toReversed();",
            None,
            FixKind::Suggestion,
        ),
        (
            "const reversed = [...a ? b : c].reverse();",
            "const reversed = (a ? b : c).toReversed();",
            None,
            FixKind::Suggestion,
        ),
        ("obj.arr?.slice().reverse();", "obj.arr?.toReversed();", None, FixKind::Suggestion),
        ("obj.arr?.concat().reverse();", "obj.arr?.toReversed();", None, FixKind::Suggestion),
        ("[...Array.from(x)].reverse()", "Array.from(x).toReversed()", None, FixKind::Fix),
        ("Object.keys(o).slice().reverse()", "Object.keys(o).toReversed()", None, FixKind::Fix),
        ("[...[1, 2, 3]].reverse()", "[1, 2, 3].toReversed()", None, FixKind::Fix),
        ("[...new Foo].reverse()", "(new Foo).toReversed()", None, FixKind::Suggestion),
        ("[...(0, array)].reverse()", "(0, array).toReversed()", None, FixKind::Suggestion),
        ("[...await x].reverse()", "(await x).toReversed()", None, FixKind::Suggestion),
        ("[...mySet].reverse()", "mySet.toReversed()", None, FixKind::Suggestion),
        // TypeScript casts are kept.
        (
            "[...(arr as number[])].reverse()",
            "(arr as number[]).toReversed()",
            None,
            FixKind::Suggestion,
        ),
        // Optional chains are wrapped so a nullish `obj` still throws, as the original does.
        ("[...obj?.arr].reverse()", "(obj?.arr).toReversed()", None, FixKind::Suggestion),
        // A local `Array` is not the global one, so this must not be a safe fix.
        (
            "const Array = { from: () => new Set() }; [...Array.from(x)].reverse()",
            "const Array = { from: () => new Set() }; Array.from(x).toReversed()",
            None,
            FixKind::Suggestion,
        ),
        // Common string-reversal idiom: only a suggestion, never an automatic fix.
        (
            r#"[...str].reverse().join("")"#,
            r#"str.toReversed().join("")"#,
            None,
            FixKind::Suggestion,
        ),
        // A comment outside the source expression would be lost, so no fix is offered.
        (
            "[/* keep */ ...arr].reverse()",
            "[/* keep */ ...arr].reverse()",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(PreferArrayToReversed::NAME, PreferArrayToReversed::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
