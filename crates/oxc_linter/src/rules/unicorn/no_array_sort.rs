use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use oxc_ast::{
    AstKind,
    ast::{Argument, ArrayExpressionElement, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    ast_util::leftmost_identifier_reference,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_import_symbol,
};

fn no_array_sort_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `Array#toSorted()` instead of `Array#sort()`.")
        .with_help("`Array#sort()` mutates the original array. Use `Array#toSorted()` to return a new sorted array without modifying the original.")
        .with_label(span)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoArraySort {
    /// When set to `true` (default), allows `array.sort()` as an expression statement.
    /// Set to `false` to forbid `Array#sort()` even if it's an expression statement.
    ///
    /// Example of **incorrect** code for this rule with `allowExpressionStatement` set to `false`:
    /// ```js
    /// array.sort();
    /// ```
    allow_expression_statement: bool,

    /// When set to `true`, allows sorting a fresh array created by a spread, e.g. `[...iterable].sort()`.
    /// This avoids the double allocation of `toSorted()` when sorting an iterable such as a `Set`.
    ///
    /// Example of **correct** code for this rule with `allowAfterSpread` set to `true`:
    /// ```js
    /// const sorted = [...mySet].sort();
    /// ```
    allow_after_spread: bool,
}

impl Default for NoArraySort {
    fn default() -> Self {
        Self { allow_expression_statement: true, allow_after_spread: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer using `Array#toSorted()` over `Array#sort()`.
    ///
    /// ### Why is this bad?
    ///
    /// `Array#sort()` modifies the original array in place, which can lead to unintended side effects—especially
    /// when the original array is used elsewhere in the code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const sorted = [...array].sort();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const sorted = [...array].toSorted();
    /// ```
    NoArraySort,
    unicorn,
    suspicious,
    fix,
    config = NoArraySort,
    version = "1.15.0",
    short_description = "Prefer using `Array#toSorted()` over `Array#sort()`.",
);

impl Rule for NoArraySort {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        if call_expr.optional {
            return;
        }
        if call_expr.arguments.len() > 1 {
            return;
        }
        if call_expr.arguments.len() == 1
            && matches!(call_expr.arguments[0], Argument::SpreadElement(_))
        {
            return;
        }
        // Skip calls whose single argument is incompatible with
        // `Array.prototype.sort(compareFn?)`. The legitimate signature accepts
        // either zero arguments or a function-shaped value. Object, string,
        // numeric, template-literal, and array-literal arguments cannot be
        // valid `compareFn`s, so they almost always indicate a non-array
        // receiver (e.g. a query-builder API like Mongoose's
        // `Model.find().sort({ field: 1 })`). See issue #22487.
        if call_expr.arguments.len() == 1 && is_non_compare_fn_argument(&call_expr.arguments[0]) {
            return;
        }
        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };
        let Some((span, static_property_name)) = member_expr.static_property_info() else {
            return;
        };
        if static_property_name != "sort" {
            return;
        }
        if leftmost_identifier_reference(member_expr.object())
            .is_ok_and(|ident| is_import_symbol(ident, "effect", "Chunk", ctx))
        {
            return;
        }

        let is_spread = match member_expr.object() {
            Expression::ArrayExpression(array) => {
                array.elements.len() == 1
                    && matches!(array.elements[0], ArrayExpressionElement::SpreadElement(_))
            }
            _ => false,
        };

        if self.allow_after_spread && is_spread {
            return;
        }

        if self.allow_expression_statement && !is_spread {
            let parent = ctx.nodes().parent_node(node.id());
            let parent_is_expression_statement = match parent.kind() {
                AstKind::ExpressionStatement(_) => true,
                AstKind::ChainExpression(_) => {
                    let grand_parent = ctx.nodes().parent_node(parent.id());
                    matches!(grand_parent.kind(), AstKind::ExpressionStatement(_))
                }
                _ => false,
            };
            if parent_is_expression_statement {
                return;
            }
        }

        ctx.diagnostic_with_fix(no_array_sort_diagnostic(span), |fixer| {
            fixer.replace(span, "toSorted")
        });
    }
}

/// Returns `true` when `arg` cannot be a valid `compareFn` for
/// `Array.prototype.sort`. Used to filter out query-builder style calls such
/// as Mongoose's `Model.find().sort({ field: 1 })` or
/// `query.sort("-createdAt")` which are not `Array#sort` despite the shared
/// method name.
fn is_non_compare_fn_argument(arg: &Argument<'_>) -> bool {
    let Some(expr) = arg.as_expression().map(Expression::without_parentheses) else {
        return false;
    };

    match expr {
        Expression::ObjectExpression(_)
        | Expression::StringLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::ArrayExpression(_) => true,
        // `query.sort(-1)` / `query.sort(+1)` — unary on a numeric literal.
        Expression::UnaryExpression(unary) => {
            unary.operator.is_arithmetic()
                && unary.argument.without_parentheses().is_number_literal()
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("sorted =[...array].toSorted()", None),
        ("sorted =array.toSorted()", None),
        ("sorted =[...array].sort", None),
        ("sorted =[...array].sort?.()", None),
        ("array.sort()", None),
        ("array.sort?.()", None),
        ("array?.sort()", None),
        ("if (true) array.sort()", None),
        ("sorted = array.sort(...[])", None),
        ("sorted = array.sort(...[compareFn])", None),
        ("sorted = array.sort(compareFn, extraArgument)", None),
        (r#"import { Chunk } from "effect"; const sorted = Chunk.sort(compareFn)"#, None),
        (r#"import { Chunk as C } from "effect"; const sorted = C.sort(compareFn)"#, None),
        ("sorted = collection.sort({field: 1})", None),
        (r#"sorted = query.sort("field")"#, None),
        ("sorted = query.sort(1)", None),
        ("sorted = query.sort(-1)", None),
        ("sorted = query.sort(+1)", None),
        ("sorted = query.sort(`field`)", None),
        ("sorted = query.sort([criteria])", None),
        ("const docs = collection.find({id}).sort({expireAt: -1}).limit(1).toArray()", None),
        ("[...array].sort({field: 1})", None),
        (
            "collection.sort({field: 1})",
            Some(serde_json::json!([{ "allowExpressionStatement": false }])),
        ),
        ("User.find().sort({ createdAt: -1 })", None),
        (r#"User.find().sort("-createdAt")"#, None),
        (r#"Post.find({ published: true }).sort({ updatedAt: "desc" })"#, None),
        ("sorted = collection.sort(({field: 1}))", None),
        (r#"sorted = query.sort(("field"))"#, None),
        ("sorted = [...mySet].sort()", Some(serde_json::json!([{ "allowAfterSpread": true }]))),
        ("sorted = [...array].sort()", Some(serde_json::json!([{ "allowAfterSpread": true }]))),
        ("sorted = [...array]?.sort()", Some(serde_json::json!([{ "allowAfterSpread": true }]))),
        (
            "sorted = [...array].sort(compareFn)",
            Some(serde_json::json!([{ "allowAfterSpread": true }])),
        ),
        ("[...array].sort()", Some(serde_json::json!([{ "allowAfterSpread": true }]))),
    ];

    let fail = vec![
        ("sorted = [...array].sort()", None),
        ("sorted = [...array]?.sort()", None),
        ("sorted = array.sort()", None),
        ("sorted = array?.sort()", None),
        ("sorted = [...array].sort(compareFn)", None),
        ("sorted = [...array]?.sort(compareFn)", None),
        ("sorted = array.sort(compareFn)", None),
        ("sorted = array?.sort(compareFn)", None),
        ("array.sort()", Some(serde_json::json!([{ "allowExpressionStatement": false }]))),
        ("array?.sort()", Some(serde_json::json!([{ "allowExpressionStatement": false }]))),
        ("[...array].sort()", Some(serde_json::json!([{ "allowExpressionStatement": false }]))),
        ("sorted = [...(0, array)].sort()", None),
        ("sorted = array.sort()", Some(serde_json::json!([{ "allowAfterSpread": true }]))),
        ("sorted = [...array].sort()", Some(serde_json::json!([{ "allowAfterSpread": false }]))),
    ];

    let fix = vec![
        ("sorted = [...array].sort()", "sorted = [...array].toSorted()", None),
        ("sorted = [...array]?.sort()", "sorted = [...array]?.toSorted()", None),
        (
            "a.sort()",
            "a.toSorted()",
            Some(serde_json::json!([{ "allowExpressionStatement": false }])),
        ),
        ("sorted = array?.sort()", "sorted = array?.toSorted()", None),
    ];

    Tester::new(NoArraySort::NAME, NoArraySort::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
