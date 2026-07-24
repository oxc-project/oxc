use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, Expression, IdentifierReference, LogicalExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{
    GlobalContext,
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext, PropertyReadSideEffects},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{is_same_expression, is_same_member_expression},
};

fn prefer_includes_over_repeated_comparisons_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `.includes()` instead of repeated equality checks.")
        .with_help("Rewrite the repeated `===` comparisons as a single `.includes()` call.")
        .with_label(span)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferIncludesOverRepeatedComparisonsConfig {
    /// The minimum number of equality comparisons before reporting.
    #[serde(default = "default_minimum_comparisons")]
    minimum_comparisons: u32,
}

fn default_minimum_comparisons() -> u32 {
    3
}

impl Default for PreferIncludesOverRepeatedComparisonsConfig {
    fn default() -> Self {
        Self { minimum_comparisons: default_minimum_comparisons() }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PreferIncludesOverRepeatedComparisons(Box<PreferIncludesOverRepeatedComparisonsConfig>);

impl std::ops::Deref for PreferIncludesOverRepeatedComparisons {
    type Target = PreferIncludesOverRepeatedComparisonsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers `.includes()` over repeated strict equality comparisons joined by `||`.
    ///
    /// ### Why is this bad?
    ///
    /// Comparing the same expression against multiple values is easier to scan as a
    /// membership check. An `Array#includes()` rewrite communicates intent more clearly.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// value === 'a' || value === 'b' || value === 'c';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// ['a', 'b', 'c'].includes(value);
    /// ```
    ///
    /// ### Options
    ///
    /// #### minimumComparisons
    ///
    /// `{ type: integer, minimum: 2, default: 3 }`
    ///
    /// The minimum number of equality comparisons before reporting.
    PreferIncludesOverRepeatedComparisons,
    unicorn,
    style,
    none,
    config = PreferIncludesOverRepeatedComparisonsConfig,
    version = "next",
    short_description = "Prefer `.includes()` over repeated equality comparisons.",
);

impl Rule for PreferIncludesOverRepeatedComparisons {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        let config = value.as_array().and_then(|arr| arr.first()).cloned().map_or_else(
            || Ok(PreferIncludesOverRepeatedComparisonsConfig::default()),
            serde_json::from_value,
        )?;

        let minimum_comparisons = config.minimum_comparisons.max(2);
        Ok(Self(Box::new(PreferIncludesOverRepeatedComparisonsConfig { minimum_comparisons })))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LogicalExpression(logical_expr) = node.kind() else {
            return;
        };

        if logical_expr.operator != LogicalOperator::Or {
            return;
        }

        // Only handle the outermost `||` expression in a chain.
        if let AstKind::LogicalExpression(parent) = ctx.nodes().parent_kind(node.id())
            && parent.operator == LogicalOperator::Or
        {
            return;
        }

        let Some(comparisons) = get_logical_or_operands(logical_expr) else {
            return;
        };
        if comparisons.len() < self.minimum_comparisons as usize {
            return;
        }

        if comparisons.iter().any(|comparison| {
            !matches!(comparison.operator, BinaryOperator::StrictEquality)
                || contains_optional_chain(&comparison.left)
                || contains_optional_chain(&comparison.right)
        }) {
            return;
        }

        let Some(shared_reference) = get_shared_reference(&comparisons, ctx) else {
            return;
        };

        if is_nan_value(shared_reference, ctx) {
            return;
        }

        let side_effects_ctx = SideEffectsContext { ctx };
        if !comparisons.iter().all(|comparison| {
            get_compared_value(comparison, shared_reference, ctx).is_some_and(|compared_value| {
                !compared_value.may_have_side_effects(&side_effects_ctx)
                    && !is_nan_value(compared_value, ctx)
            })
        }) {
            return;
        }

        ctx.diagnostic(prefer_includes_over_repeated_comparisons_diagnostic(logical_expr.span));
    }
}

fn get_logical_or_operands<'a, 'b>(
    node: &'b LogicalExpression<'a>,
) -> Option<Vec<&'b BinaryExpression<'a>>> {
    let mut operands = Vec::new();
    collect_logical_or_operands(node, &mut operands)?;
    Some(operands)
}

fn collect_logical_or_operands<'a, 'b>(
    node: &'b LogicalExpression<'a>,
    out: &mut Vec<&'b BinaryExpression<'a>>,
) -> Option<()> {
    for child in [&node.left, &node.right] {
        match child.without_parentheses() {
            Expression::LogicalExpression(logical_expr)
                if logical_expr.operator == LogicalOperator::Or =>
            {
                collect_logical_or_operands(logical_expr, out)?;
            }
            Expression::BinaryExpression(bin_expr) => out.push(bin_expr),
            _ => return None,
        }
    }
    Some(())
}

fn get_shared_reference<'a, 'b>(
    comparisons: &[&'b BinaryExpression<'a>],
    ctx: &LintContext<'a>,
) -> Option<&'b Expression<'a>> {
    let first = comparisons.first()?;
    let mut candidates: Vec<&Expression<'a>> = [&first.left, &first.right]
        .into_iter()
        .filter(|expr| is_reference(expr) && !is_undefined(expr, ctx))
        .collect();

    for comparison in comparisons.iter().skip(1) {
        candidates.retain(|candidate| {
            is_same_reference(candidate, &comparison.left, ctx)
                || is_same_reference(candidate, &comparison.right, ctx)
        });
        if candidates.is_empty() {
            return None;
        }
    }

    if candidates.len() == 1 { Some(candidates[0]) } else { None }
}

fn get_compared_value<'a, 'b>(
    comparison: &'b BinaryExpression<'a>,
    shared_reference: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'b Expression<'a>> {
    let left_is_shared = is_same_reference(&comparison.left, shared_reference, ctx);
    let right_is_shared = is_same_reference(&comparison.right, shared_reference, ctx);

    match (left_is_shared, right_is_shared) {
        (true, false) => Some(&comparison.right),
        (false, true) => Some(&comparison.left),
        _ => None,
    }
}

fn is_reference(expr: &Expression<'_>) -> bool {
    matches!(
        expr.get_inner_expression(),
        Expression::Identifier(_)
            | Expression::Super(_)
            | Expression::ThisExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::PrivateFieldExpression(_)
    )
}

fn is_same_reference<'a>(
    left: &Expression<'a>,
    right: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let left = left.get_inner_expression();
    let right = right.get_inner_expression();

    if let (Some(left_member), Some(right_member)) =
        (left.as_member_expression(), right.as_member_expression())
    {
        return is_same_member_expression(left_member, right_member, ctx);
    }

    is_same_expression(left, right, ctx)
}

fn is_undefined<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(ident) if ident.name == "undefined" => {
            ident.is_global_reference(ctx.scoping())
        }
        _ => false,
    }
}

fn is_nan_value<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(ident) if ident.name == "NaN" => {
            ident.is_global_reference(ctx.scoping())
        }
        Expression::StaticMemberExpression(member)
            if member.property.name == "NaN"
                && matches!(
                    member.object.get_inner_expression(),
                    Expression::Identifier(ident)
                        if ident.name == "Number" && ident.is_global_reference(ctx.scoping())
                ) =>
        {
            true
        }
        _ => false,
    }
}

fn contains_optional_chain(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::ChainExpression(_) => true,
        Expression::ParenthesizedExpression(paren) => contains_optional_chain(&paren.expression),
        Expression::TSAsExpression(expr) => contains_optional_chain(&expr.expression),
        Expression::TSSatisfiesExpression(expr) => contains_optional_chain(&expr.expression),
        Expression::TSTypeAssertion(expr) => contains_optional_chain(&expr.expression),
        Expression::TSNonNullExpression(expr) => contains_optional_chain(&expr.expression),
        Expression::TSInstantiationExpression(expr) => contains_optional_chain(&expr.expression),
        Expression::StaticMemberExpression(member) => {
            member.optional || contains_optional_chain(&member.object)
        }
        Expression::ComputedMemberExpression(member) => {
            member.optional
                || contains_optional_chain(&member.object)
                || contains_optional_chain(&member.expression)
        }
        Expression::PrivateFieldExpression(member) => {
            member.optional || contains_optional_chain(&member.object)
        }
        Expression::CallExpression(call) => {
            call.optional
                || contains_optional_chain(&call.callee)
                || call
                    .arguments
                    .iter()
                    .any(|arg| arg.as_expression().is_some_and(contains_optional_chain))
        }
        _ => false,
    }
}

struct SideEffectsContext<'a, 'c> {
    ctx: &'c LintContext<'a>,
}

impl<'a> GlobalContext<'a> for SideEffectsContext<'_, 'a> {
    fn is_global_reference(&self, reference: &IdentifierReference<'a>) -> bool {
        reference.is_global_reference(self.ctx.scoping())
    }
}

impl<'a> MayHaveSideEffectsContext<'a> for SideEffectsContext<'_, 'a> {
    fn annotations(&self) -> bool {
        false
    }

    fn manual_pure_functions(&self, _callee: &Expression) -> bool {
        false
    }

    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        // Match eslint-utils `hasSideEffect`: plain property reads are treated as pure.
        PropertyReadSideEffects::None
    }

    fn unknown_global_side_effects(&self) -> bool {
        false
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"value === "a";"#, None),
        (r#"value === "a" || otherValue === "b";"#, None),
        (r#"value == "a" || value == "b";"#, None),
        (r#"value !== "a" && value !== "b";"#, None),
        (r#"value === "a" || isValue(value);"#, None),
        (r#"value === "a" || value !== "b";"#, None),
        (r#"value === "a" || value === "b" || otherValue === "c";"#, None),
        (r#"getValue() === "a" || getValue() === "b";"#, None),
        (r#"getObject().value === "a" || getObject().value === "b";"#, None),
        (r#"object[getKey()] === "a" || object[getKey()] === "b";"#, None),
        (r#"value === getValue() || value === "b";"#, None),
        (r#"value === "a" || value === getValue();"#, None),
        (r#"value === (otherValue = "a") || value === "b";"#, None),
        (r#"value === ++otherValue || value === "b";"#, None),
        ("value === Number.NaN || value === 1;", None),
        ("value === NaN || value === 1;", None),
        ("Number.NaN === value || Number.NaN === otherValue;", None),
        ("NaN === value || NaN === otherValue;", None),
        (r#"foo?.bar === "a" || foo?.bar === "b";"#, None),
        (r#"foo?.bar === "a" || foo.bar === "b";"#, None),
        (r#"foo.bar === "a" || foo?.bar === "b";"#, None),
        (r#"foo[bar?.baz] === "a" || foo[bar?.baz] === "b";"#, None),
        (r#"foo[bar?.()] === "a" || foo[bar?.()] === "b";"#, None),
        (r#"(foo?.bar).baz === "a" || (foo?.bar).baz === "b";"#, None),
        (r#"value === "a" || value === foo?.bar;"#, None),
        (r#"value === (foo?.bar ?? "a") || value === "b";"#, None),
        ("foo === bar || bar === foo;", None),
        ("foo === foo || foo === foo;", None),
        ("foo === bar || foo === foo;", None),
        ("foo === 1 || bar === 1;", None),
        (r#"value === "a" || value === "b";"#, None),
        (r#""a" === value || "b" === value;"#, None),
        (r#"value === "a" || "b" === value;"#, None),
        ("value === first || value === second;", None),
        (r#"args[0] === "-h" || args[0] === "--help";"#, None),
        (r#"object.value === "a" || object.value === "b";"#, None),
        (r#"object.foo === "a" || object["foo"] === "b";"#, None),
        (r#"object[key] === "a" || object[key] === "b";"#, None),
        ("value === object.a || value === object.b;", None),
        (r#"(value === "a") || (value === "b");"#, None),
        (r#"(value === "a" || value === "b") && otherValue;"#, None),
        ("state.a === undefined || state.b === undefined || state.c === undefined;", None),
        ("undefined === a || undefined === b || undefined === c;", None),
        ("a === null || b === null || c === null;", None),
        (r#"a === undefined || a === "x" || b === undefined;"#, None),
        ("a === (undefined as any) || b === (undefined as any) || c === (undefined as any);", None),
        (
            r#"value === "a" || value === "b";"#,
            Some(serde_json::json!([{"minimumComparisons": 4}])),
        ),
        (r#"(foo?.bar as Foo).baz === "a" || (foo?.bar as Foo).baz === "b";"#, None),
    ];

    let fail = vec![
        (r#"value === "a" || value === "b" || value === "c";"#, None),
        (r#"value === undefined || value === "a" || value === "b";"#, None),
        (
            "value === null || value === undefined;",
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            r#"value === "a" || value === "b";"#,
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            r#""a" === value || "b" === value;"#,
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            r#"value === "a" || "b" === value;"#,
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            "value === first || value === second;",
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            r#"args[0] === "-h" || args[0] === "--help";"#,
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            r#"object.value === "a" || object.value === "b";"#,
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            r#"object.foo === "a" || object["foo"] === "b";"#,
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            r#"object[key] === "a" || object[key] === "b";"#,
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            "value === object.a || value === object.b;",
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            r#"(value === "a") || (value === "b");"#,
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            r#"(value === "a" || value === "b") && otherValue;"#,
            Some(serde_json::json!([{"minimumComparisons": 2}])),
        ),
        (
            r#"value === "a" || value === "b" || value === "c";"#,
            Some(serde_json::json!([{"minimumComparisons": 3}])),
        ),
        (r#"value! === "a" || value! === "b" || value! === "c";"#, None),
        (
            r#"(value satisfies string) === "a" || (value satisfies string) === "b" || (value satisfies string) === "c";"#,
            None,
        ),
        (
            r#"(value as {foo?: string}) === "a" || (value as {foo?: string}) === "b" || (value as {foo?: string}) === "c";"#,
            None,
        ),
        (
            r#"(object satisfies Foo).value === "a" || (object satisfies Foo).value === "b" || (object satisfies Foo).value === "c";"#,
            None,
        ),
    ];

    Tester::new(
        PreferIncludesOverRepeatedComparisons::NAME,
        PreferIncludesOverRepeatedComparisons::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
