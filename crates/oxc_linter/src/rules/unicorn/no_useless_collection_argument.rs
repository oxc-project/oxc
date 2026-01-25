use oxc_ast::{
    AstKind,
    ast::{Expression, LogicalExpression, NewExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::is_new_expression,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::is_empty_array_expression,
};

fn no_useless_collection_argument_diagnostic(span: Span, expr_type: &str) -> OxcDiagnostic {
    let message = format!("The {expr_type} is useless");
    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessCollectionArgument;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow useless values or fallbacks in Set, Map, WeakSet, or WeakMap
    ///
    /// ### Why is this bad?
    ///
    /// It's unnecessary to pass an empty array or string when constructing a Set, Map, WeakSet, or WeakMap, since they accept nullish values.
    /// It's also unnecessary to provide a fallback for possible nullish values.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const set = new Set([]);
    /// const set = new Set("");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const set = new Set();
    /// ```
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const set = new Set(foo ?? []);
    /// const set = new Set(foo ?? "");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const set = new Set(foo);
    /// ```
    NoUselessCollectionArgument,
    unicorn,
    style,
    suggestion,
);

impl Rule for NoUselessCollectionArgument {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        if !is_set_or_map(new_expr) {
            return;
        }

        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };

        let Some(first_arg_expr) = first_arg.as_expression() else { return };

        let first_arg_expr_inner = first_arg_expr.get_inner_expression();

        let (useless_expr, logical_expr) = if let Expression::LogicalExpression(logical_expr) =
            first_arg_expr_inner
            && logical_expr.operator.is_coalesce()
        {
            (logical_expr.right.get_inner_expression(), Some(logical_expr))
        } else {
            (first_arg_expr_inner, None)
        };

        let Some(description) = get_description(useless_expr) else {
            return;
        };

        ctx.diagnostic_with_suggestion(
            no_useless_collection_argument_diagnostic(useless_expr.span(), description),
            |fixer| {
                if let Some(logical_expr) = logical_expr {
                    remove_fallback(fixer, first_arg_expr, logical_expr)
                } else {
                    remove_argument(fixer, new_expr)
                }
            },
        );
    }
}

fn is_set_or_map(new_expr: &NewExpression<'_>) -> bool {
    is_new_expression(new_expr, &["Set", "Map", "WeakSet", "WeakMap"], Some(1), Some(1))
}

fn get_description(expr: &Expression) -> Option<&'static str> {
    if is_empty_array_expression(expr) {
        return Some("empty array");
    }

    if expr.is_specific_string_literal("") {
        return Some("empty string");
    }

    if expr.is_null() {
        return Some("null");
    }

    if expr.is_undefined() {
        return Some("undefined");
    }

    None
}

fn remove_argument(fixer: RuleFixer, new_expr: &NewExpression) -> RuleFix {
    let Some(first_arg) = new_expr.arguments.first() else {
        return fixer.noop();
    };

    let arg_end = first_arg.span().end;

    let source = fixer.source_text();
    let mut delete_end = arg_end;

    let after_arg = &source[arg_end as usize..];
    let trimmed = after_arg.trim_start();

    // remove trailing comma
    if trimmed.starts_with(',') {
        let Ok(comma_offset) = u32::try_from(after_arg.len() - trimmed.len()) else {
            return fixer.noop();
        };

        delete_end = arg_end + comma_offset + 1;
    }

    fixer.delete_range(Span::new(first_arg.span().start, delete_end))
}

fn remove_fallback(
    fixer: RuleFixer,
    arg_expr: &Expression,
    logical_expr: &LogicalExpression,
) -> RuleFix {
    let left_end = logical_expr.left.span().end;
    let logical_end = logical_expr.span.end;
    let logical_start = logical_expr.span.start;

    let arg_start = arg_expr.span().start;
    let arg_end = arg_expr.span().end;
    let has_outer_parens = arg_start < logical_start || arg_end > logical_end;

    if has_outer_parens {
        let source = fixer.source_text();

        let before_logical = &source[arg_start as usize..logical_start as usize];
        let after_logical = &source[logical_end as usize..arg_end as usize];

        let before_cleaned = before_logical.trim_start_matches('(');
        let after_cleaned = after_logical.trim_end_matches(')');

        let left_text = fixer.source_range(logical_expr.left.span());

        let replacement = format!("{before_cleaned}{left_text}{after_cleaned}");
        fixer.replace(arg_expr.span(), replacement)
    } else {
        fixer.delete_range(Span::new(left_end, logical_end))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "new Set()",
        "new Set",
        "new Set(foo)",
        "new Set(foo || [])",
        "new Set(foo && [])",
        "new Not_Set([])",
        "Set([])",
        "new Set([], extraArgument)",
        "new Set(...([]))",
        r#"new Set([""])"#,
        r#"new Set("not-empty")"#,
        r#"new Set("null")"#,
        r#"new Set("undefined")"#,
        "new Set(0)",
        "new ([])(Set)",
        "new globalThis.Set([])",
        "new Set(foo || [])",
        "new Set(foo && [])",
        "new Not_Set(foo ?? [])",
        "Set(foo ?? [])",
        "new Set(foo ?? [], extraArgument)",
        "new Set(...(foo ?? []))",
        r#"new Set(foo ?? [""])"#,
        r#"new Set(foo ?? "not-empty")"#,
        "new Set(foo ?? 0)",
        "new (foo ?? [])(Set)",
        "new globalThis.Set(foo ?? [])",
    ];

    let fail = vec![
        "new Set([])",
        r#"new Set("")"#,
        "new Set(undefined)",
        "new Set(null)",
        "new WeakSet([])",
        "new Map([])",
        "new WeakMap([])",
        "new Set( (([])) )",
        "new Set([],)",
        "new Set( (([])), )",
        "new Set(foo ?? [])",
        r#"new Set(foo ?? "")"#,
        "new Set(foo ?? undefined)",
        "new Set(foo ?? null)",
        "new WeakSet(foo ?? [])",
        "new Map(foo ?? [])",
        "new WeakMap(foo ?? [])",
        "new Set( ((foo ?? [])) )",
        "new Set( (( foo )) ?? [] )",
        "new Set( foo ?? (( [] )) )",
        "new Set( (await foo) ?? [] )",
        "new Set( (0, foo) ?? [] )",
        "new Set( (( (0, foo) ?? [] )) )",
        "new Set(document.all ?? [])",
        r#"new Set([] ?? "")"#,
        r#"new Set( (( (( "" )) ?? (( [] )) )) )"#,
        "new Set(foo ?? bar ?? [])",
    ];

    let fix = vec![
        ("new Set([])", "new Set()"),
        (r#"new Set("")"#, "new Set()"),
        ("new Set(undefined)", "new Set()"),
        ("new Set(null)", "new Set()"),
        ("new WeakSet([])", "new WeakSet()"),
        ("new Map([])", "new Map()"),
        ("new WeakMap([])", "new WeakMap()"),
        ("new Set( (([])) )", "new Set(  )"),
        ("new Set([],)", "new Set()"),
        ("new Set( (([])), )", "new Set(  )"),
        ("new Set(foo ?? [])", "new Set(foo)"),
        (r#"new Set(foo ?? "")"#, "new Set(foo)"),
        ("new Set(foo ?? undefined)", "new Set(foo)"),
        ("new Set(foo ?? null)", "new Set(foo)"),
        ("new WeakSet(foo ?? [])", "new WeakSet(foo)"),
        ("new Map(foo ?? [])", "new Map(foo)"),
        ("new WeakMap(foo ?? [])", "new WeakMap(foo)"),
        ("new Set( ((foo ?? [])) )", "new Set( foo )"),
        ("new Set( (( foo )) ?? [] )", "new Set( (( foo )) )"),
        ("new Set( foo ?? (( [] )) )", "new Set( foo )"),
        ("new Set( (await foo) ?? [] )", "new Set( (await foo) )"),
        ("new Set( (0, foo) ?? [] )", "new Set( (0, foo) )"),
        ("new Set( (( (0, foo) ?? [] )) )", "new Set(  (0, foo)  )"),
        ("new Set(document.all ?? [])", "new Set(document.all)"),
        (r#"new Set([] ?? "")"#, "new Set([])"),
        (r#"new Set( (( (( "" )) ?? (( [] )) )) )"#, r#"new Set(  (( "" ))  )"#),
        ("new Set(foo ?? bar ?? [])", "new Set(foo ?? bar)"),
    ];

    Tester::new(NoUselessCollectionArgument::NAME, NoUselessCollectionArgument::PLUGIN, pass, fail)
        .change_rule_path_extension("mjs")
        .expect_fix(fix)
        .test_and_snapshot();
}
