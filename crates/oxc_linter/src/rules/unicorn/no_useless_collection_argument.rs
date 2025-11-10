use oxc_ast::{
    AstKind,
    ast::{Expression, NewExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, ast_util::is_new_expression, context::LintContext, rule::Rule,
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
    pending,
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

        let first_arg_expr = first_arg_expr.get_inner_expression();

        let first_arg_expr = if let Expression::LogicalExpression(logical_expr) = first_arg_expr
            && logical_expr.operator.is_coalesce()
        {
            logical_expr.right.get_inner_expression()
        } else {
            first_arg_expr
        };

        let Some(description) = get_description(first_arg_expr) else {
            return;
        };

        ctx.diagnostic(no_useless_collection_argument_diagnostic(
            first_arg_expr.span(),
            description,
        ));
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

    Tester::new(NoUselessCollectionArgument::NAME, NoUselessCollectionArgument::PLUGIN, pass, fail)
        .test_and_snapshot();
}
