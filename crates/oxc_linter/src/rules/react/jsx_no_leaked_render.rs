use oxc_ast::{
    AstKind,
    ast::{ConditionalExpression, Expression, LogicalExpression, LogicalOperator, UnaryOperator},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn jsx_no_leaked_render_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Potential leaked value that might cause unintentionally rendered values or rendering crashes",
    )
    .with_help(
        "Coerce the expression to a boolean using `!!` or use a ternary `? ... : null`",
    )
    .with_label(span)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ValidStrategy {
    Ternary,
    Coerce,
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct JsxNoLeakedRender {
    /// Allowed fix strategies. Possible values: `"ternary"`, `"coerce"`.
    valid_strategies: Vec<ValidStrategy>,
    /// When `true`, ignores logical expressions in JSX attribute values.
    ignore_attributes: bool,
}

impl Default for JsxNoLeakedRender {
    fn default() -> Self {
        Self {
            valid_strategies: vec![ValidStrategy::Ternary, ValidStrategy::Coerce],
            ignore_attributes: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow leaked values in JSX.
    ///
    /// ### Why is this bad?
    ///
    /// Using the `&&` operator to conditionally render JSX may unintentionally
    /// render values like `0`, `NaN`, or empty strings in the DOM. In React Native,
    /// rendering these values can crash the app.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// const Component = ({ count }) => {
    ///   return <div>{count && <span>There are {count} results</span>}</div>
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const Component = ({ count }) => {
    ///   return <div>{count ? <span>There are {count} results</span> : null}</div>
    /// }
    /// ```
    JsxNoLeakedRender,
    react,
    nursery,
    fix,
    config = JsxNoLeakedRender,
);

impl Rule for JsxNoLeakedRender {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXExpressionContainer(container) = node.kind() else { return };

        if self.ignore_attributes {
            let parent = ctx.nodes().parent_kind(node.id());
            if matches!(parent, AstKind::JSXAttribute(_)) {
                return;
            }
        }

        let Some(expr) = container.expression.as_expression() else { return };

        match expr {
            Expression::LogicalExpression(logical) if logical.operator == LogicalOperator::And => {
                check_and_expression(&self.valid_strategies, logical, expr, ctx);
            }
            Expression::ConditionalExpression(cond)
                if self.valid_strategies == [ValidStrategy::Coerce] =>
            {
                check_conditional_expression(cond, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

fn check_and_expression<'a>(
    strategies: &[ValidStrategy],
    logical: &LogicalExpression<'a>,
    expr: &Expression<'a>,
    ctx: &LintContext<'a>,
) {
    // {count > 0 && title} with ternary-only: all && in JSX must use ternary
    if strategies == [ValidStrategy::Ternary] {
        ctx.diagnostic_with_fix(jsx_no_leaked_render_diagnostic(logical.span), |fixer| {
            fix_with_ternary(logical, ctx, fixer)
        });
        return;
    }

    let operands = collect_and_operands(expr);
    let has_unsafe = operands[..operands.len() - 1].iter().any(|op| !is_safe_left_expression(op));

    if !has_unsafe {
        return;
    }

    let prefers_ternary = strategies.first() == Some(&ValidStrategy::Ternary);
    if prefers_ternary {
        ctx.diagnostic_with_fix(jsx_no_leaked_render_diagnostic(logical.span), |fixer| {
            fix_with_ternary(logical, ctx, fixer)
        });
    } else {
        ctx.diagnostic_with_fix(jsx_no_leaked_render_diagnostic(logical.span), |fixer| {
            fix_with_coerce(expr, fixer)
        });
    }
}

fn check_conditional_expression<'a>(cond: &ConditionalExpression<'a>, ctx: &LintContext<'a>) {
    // {count ? title : null} with coerce-only: ternary should use coerce
    if !is_null_literal(&cond.alternate) {
        return;
    }

    ctx.diagnostic_with_fix(jsx_no_leaked_render_diagnostic(cond.span), |fixer| {
        fix_ternary_to_coerce(cond, ctx, fixer)
    });
}

// `a && b && c` (parsed as `(a && b) && c`) → `[a, b, c]`
fn collect_and_operands<'a, 'b>(expr: &'b Expression<'a>) -> Vec<&'b Expression<'a>> {
    let mut operands = Vec::new();
    let mut current = expr;
    loop {
        if let Expression::LogicalExpression(logical) = current
            && logical.operator == LogicalOperator::And
        {
            operands.push(&logical.right);
            current = &logical.left;
            continue;
        }
        operands.push(current);
        break;
    }
    operands.reverse();
    operands
}

fn is_safe_left_expression(expr: &Expression) -> bool {
    match expr {
        // !x and !!x always produce boolean
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => true,
        // Comparison/equality operators always produce boolean
        Expression::BinaryExpression(binary) => {
            binary.operator.is_compare() || binary.operator.is_equality()
        }
        // Boolean(x) call
        Expression::CallExpression(call) => {
            matches!(&call.callee, Expression::Identifier(ident) if ident.name == "Boolean")
        }
        // Empty string is falsy — right side is never rendered, so no leak
        Expression::StringLiteral(s) if s.value.is_empty() => true,
        _ => false,
    }
}

fn is_null_literal(expr: &Expression) -> bool {
    matches!(expr, Expression::NullLiteral(_))
}

fn fix_with_ternary<'a>(
    logical: &LogicalExpression<'a>,
    ctx: &LintContext<'a>,
    fixer: RuleFixer<'_, 'a>,
) -> RuleFix {
    let source = ctx.source_text();
    let left_text = get_ternary_condition_text(&logical.left, source);
    let right_text = logical.right.span().source_text(source);
    fixer.replace(logical.span, format!("{left_text} ? {right_text} : null"))
}

fn get_ternary_condition_text(expr: &Expression, source: &str) -> String {
    if let Expression::UnaryExpression(outer) = expr
        && outer.operator == UnaryOperator::LogicalNot
        && let Expression::UnaryExpression(inner) = &outer.argument
        && inner.operator == UnaryOperator::LogicalNot
    {
        let inner_expr = inner.argument.without_parentheses();
        return inner_expr.span().source_text(source).to_string();
    }
    expr.span().source_text(source).to_string()
}

fn fix_with_coerce<'a>(expr: &Expression<'a>, fixer: RuleFixer<'_, 'a>) -> RuleFix {
    let operands = collect_and_operands(expr);
    let unsafe_count =
        operands[..operands.len() - 1].iter().filter(|op| !is_safe_left_expression(op)).count();

    let mut fix = fixer.new_fix_with_capacity(unsafe_count);
    for operand in &operands[..operands.len() - 1] {
        if !is_safe_left_expression(operand) {
            fix.push(fixer.insert_text_before(&operand.span(), "!!"));
        }
    }
    fix
}

fn fix_ternary_to_coerce<'a>(
    cond: &ConditionalExpression<'a>,
    ctx: &LintContext<'a>,
    fixer: RuleFixer<'_, 'a>,
) -> RuleFix {
    let source = ctx.source_text();
    let consequent_text = cond.consequent.span().source_text(source);
    let coerced_test = coerce_test_text(&cond.test, source);
    fixer.replace(cond.span, format!("{coerced_test} && {consequent_text}"))
}

fn coerce_test_text(expr: &Expression, source: &str) -> String {
    if is_safe_left_expression(expr) {
        return expr.span().source_text(source).to_string();
    }

    // For `a && b` test: coerce each unsafe operand
    if let Expression::LogicalExpression(logical) = expr
        && logical.operator == LogicalOperator::And
    {
        let operands = collect_and_operands(expr);
        let parts: Vec<String> = operands
            .iter()
            .map(|op| {
                let text = op.span().source_text(source);
                if is_safe_left_expression(op) { text.to_string() } else { format!("!!{text}") }
            })
            .collect();
        return parts.join(" && ");
    }

    format!("!!{}", expr.span().source_text(source))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Boolean-safe left sides (default config)
        (r"const C = () => <div>{!!count && <span />}</div>", None),
        (r"const C = () => <div>{Boolean(count) && <span />}</div>", None),
        (r"const C = () => <div>{count > 0 && <span />}</div>", None),
        (r"const C = () => <div>{count !== 0 && <span />}</div>", None),
        (r"const C = () => <div>{a === b && <span />}</div>", None),
        (r"const C = () => <div>{!condition && <span />}</div>", None),
        // Ternary is fine with default config
        (r"const C = () => <div>{count ? <span /> : null}</div>", None),
        // Coerce-only: coerced values are fine
        (
            r"const C = () => <div>{!!count && <span />}</div>",
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r"const C = () => <div>{Boolean(count) && <span />}</div>",
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        // Ternary-only: ternary is fine
        (
            r"const C = () => <div>{count ? <span /> : null}</div>",
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        // ignoreAttributes: attribute values are skipped
        (
            r"const C = () => <Foo checked={enabled && checked} />",
            Some(serde_json::json!([{ "ignoreAttributes": true }])),
        ),
    ];

    let fail = vec![
        // Boolean literals are flagged (redundant)
        (r"const C = () => <div>{true && <span />}</div>", None),
        // Default config: && with unsafe left side
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{count && title}</div>
            }
        "#,
            None,
        ),
        (
            r#"
            const Component = ({ count }) => {
              return <div>{count && <span>There are {count} results</span>}</div>
            }
        "#,
            None,
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements.length && <List elements={elements}/>}</div>
            }
        "#,
            None,
        ),
        (
            r#"
            const Component = ({ nestedCollection }) => {
              return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
            }
        "#,
            None,
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements[0] && <List elements={elements}/>}</div>
            }
        "#,
            None,
        ),
        (
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
            }
        "#,
            None,
        ),
        // Default config: && with both strategies explicit
        (
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])),
        ),
        // Ternary-only: ALL && are flagged even with boolean-safe left side
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{count && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ count }) => {
              return <div>{count && <span>There are {count} results</span>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements.length && <List elements={elements}/>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ nestedCollection }) => {
              return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements[0] && <List elements={elements}/>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ someCondition, title }) => {
              return <div>{!someCondition && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{!!count && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{count > 0 && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{0 != count && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ count, total, title }) => {
              return <div>{count < total && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ count, title, somethingElse }) => {
              return <div>{!!(count && somethingElse) && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        // Coerce-only: && with unsafe left side
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{count && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ count }) => {
              return <div>{count && <span>There are {count} results</span>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements.length && <List elements={elements}/>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ nestedCollection }) => {
              return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements[0] && <List elements={elements}/>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ connection, hasError, hasErrorUpdate}) => {
              return <div>{connection && (hasError || hasErrorUpdate)}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        // Coerce-only: ternary with null alternate should use coerce
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{count ? title : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{!count ? title : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ count, somethingElse, title }) => {
              return <div>{count && somethingElse ? title : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        // Chained && with coerce
        (
            r#"
            const Component = ({ items, somethingElse, title }) => {
              return <div>{items.length > 0 && somethingElse && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const MyComponent = () => {
              const items = []
              const breakpoint = { phones: true }
              return <div>{items.length > 0 && breakpoint.phones && <span />}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])),
        ),
        // Coerce-only with ternary inside &&
        (
            r#"
            const MyComponent = () => {
              return <div>{maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        // Attribute values (default: not ignored)
        (
            r#"
            const Component = ({ enabled, checked }) => {
              return <CheckBox checked={enabled && checked} />
            }
        "#,
            None,
        ),
        (
            r#"
            const isOpen = 0;
            const Component = () => {
              return <Popover open={isOpen && items.length > 0} />
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        // ignoreAttributes: still checks children inside attribute JSX
        (
            r#"
            const Component = ({ enabled }) => {
              return (
                <Foo bar={
                  <Something>{enabled && <MuchWow />}</Something>
                } />
              )
            }
        "#,
            Some(serde_json::json!([{ "ignoreAttributes": true }])),
        ),
    ];

    let fix = vec![
        // Default config: fix with ternary (preferred first strategy)
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{count && title}</div>
            }
        "#,
            r#"
            const Component = ({ count, title }) => {
              return <div>{count ? title : null}</div>
            }
        "#,
            None,
        ),
        (
            r#"
            const Component = ({ count }) => {
              return <div>{count && <span>There are {count} results</span>}</div>
            }
        "#,
            r#"
            const Component = ({ count }) => {
              return <div>{count ? <span>There are {count} results</span> : null}</div>
            }
        "#,
            None,
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements.length && <List elements={elements}/>}</div>
            }
        "#,
            r#"
            const Component = ({ elements }) => {
              return <div>{elements.length ? <List elements={elements}/> : null}</div>
            }
        "#,
            None,
        ),
        (
            r#"
            const Component = ({ nestedCollection }) => {
              return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
            }
        "#,
            r#"
            const Component = ({ nestedCollection }) => {
              return <div>{nestedCollection.elements.length ? <List elements={nestedCollection.elements}/> : null}</div>
            }
        "#,
            None,
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements[0] && <List elements={elements}/>}</div>
            }
        "#,
            r#"
            const Component = ({ elements }) => {
              return <div>{elements[0] ? <List elements={elements}/> : null}</div>
            }
        "#,
            None,
        ),
        (
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
            }
        "#,
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{(numberA || numberB) ? <Results>{numberA+numberB}</Results> : null}</div>
            }
        "#,
            None,
        ),
        // Explicit ["coerce", "ternary"]: first strategy is coerce
        (
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
            }
        "#,
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{!!(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])),
        ),
        // Ternary-only fixes
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{count && title}</div>
            }
        "#,
            r#"
            const Component = ({ count, title }) => {
              return <div>{count ? title : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ count }) => {
              return <div>{count && <span>There are {count} results</span>}</div>
            }
        "#,
            r#"
            const Component = ({ count }) => {
              return <div>{count ? <span>There are {count} results</span> : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements.length && <List elements={elements}/>}</div>
            }
        "#,
            r#"
            const Component = ({ elements }) => {
              return <div>{elements.length ? <List elements={elements}/> : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ nestedCollection }) => {
              return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
            }
        "#,
            r#"
            const Component = ({ nestedCollection }) => {
              return <div>{nestedCollection.elements.length ? <List elements={nestedCollection.elements}/> : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements[0] && <List elements={elements}/>}</div>
            }
        "#,
            r#"
            const Component = ({ elements }) => {
              return <div>{elements[0] ? <List elements={elements}/> : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
            }
        "#,
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{(numberA || numberB) ? <Results>{numberA+numberB}</Results> : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        // Ternary-only: strips ! for non-negated, keeps ! for negated
        (
            r#"
            const Component = ({ someCondition, title }) => {
              return <div>{!someCondition && title}</div>
            }
        "#,
            r#"
            const Component = ({ someCondition, title }) => {
              return <div>{!someCondition ? title : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        // Ternary-only: strips !! from left side
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{!!count && title}</div>
            }
        "#,
            r#"
            const Component = ({ count, title }) => {
              return <div>{count ? title : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{count > 0 && title}</div>
            }
        "#,
            r#"
            const Component = ({ count, title }) => {
              return <div>{count > 0 ? title : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{0 != count && title}</div>
            }
        "#,
            r#"
            const Component = ({ count, title }) => {
              return <div>{0 != count ? title : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ count, total, title }) => {
              return <div>{count < total && title}</div>
            }
        "#,
            r#"
            const Component = ({ count, total, title }) => {
              return <div>{count < total ? title : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        (
            r#"
            const Component = ({ count, title, somethingElse }) => {
              return <div>{!!(count && somethingElse) && title}</div>
            }
        "#,
            r#"
            const Component = ({ count, title, somethingElse }) => {
              return <div>{count && somethingElse ? title : null}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
        ),
        // Coerce-only fixes: add !! before unsafe operands
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{count && title}</div>
            }
        "#,
            r#"
            const Component = ({ count, title }) => {
              return <div>{!!count && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ count }) => {
              return <div>{count && <span>There are {count} results</span>}</div>
            }
        "#,
            r#"
            const Component = ({ count }) => {
              return <div>{!!count && <span>There are {count} results</span>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements.length && <List elements={elements}/>}</div>
            }
        "#,
            r#"
            const Component = ({ elements }) => {
              return <div>{!!elements.length && <List elements={elements}/>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ nestedCollection }) => {
              return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
            }
        "#,
            r#"
            const Component = ({ nestedCollection }) => {
              return <div>{!!nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ elements }) => {
              return <div>{elements[0] && <List elements={elements}/>}</div>
            }
        "#,
            r#"
            const Component = ({ elements }) => {
              return <div>{!!elements[0] && <List elements={elements}/>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
            }
        "#,
            r#"
            const Component = ({ numberA, numberB }) => {
              return <div>{!!(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ connection, hasError, hasErrorUpdate}) => {
              return <div>{connection && (hasError || hasErrorUpdate)}</div>
            }
        "#,
            r#"
            const Component = ({ connection, hasError, hasErrorUpdate}) => {
              return <div>{!!connection && (hasError || hasErrorUpdate)}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        // Coerce-only: convert ternary with null to coerce
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{count ? title : null}</div>
            }
        "#,
            r#"
            const Component = ({ count, title }) => {
              return <div>{!!count && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ count, title }) => {
              return <div>{!count ? title : null}</div>
            }
        "#,
            r#"
            const Component = ({ count, title }) => {
              return <div>{!count && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const Component = ({ count, somethingElse, title }) => {
              return <div>{count && somethingElse ? title : null}</div>
            }
        "#,
            r#"
            const Component = ({ count, somethingElse, title }) => {
              return <div>{!!count && !!somethingElse && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        // Chained && with coerce: only coerce unsafe operands
        (
            r#"
            const Component = ({ items, somethingElse, title }) => {
              return <div>{items.length > 0 && somethingElse && title}</div>
            }
        "#,
            r#"
            const Component = ({ items, somethingElse, title }) => {
              return <div>{items.length > 0 && !!somethingElse && title}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        (
            r#"
            const MyComponent = () => {
              const items = []
              const breakpoint = { phones: true }
              return <div>{items.length > 0 && breakpoint.phones && <span />}</div>
            }
        "#,
            r#"
            const MyComponent = () => {
              const items = []
              const breakpoint = { phones: true }
              return <div>{items.length > 0 && !!breakpoint.phones && <span />}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])),
        ),
        (
            r#"
            const MyComponent = () => {
              return <div>{maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
            }
        "#,
            r#"
            const MyComponent = () => {
              return <div>{!!maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        // Attribute values
        (
            r#"
            const Component = ({ enabled, checked }) => {
              return <CheckBox checked={enabled && checked} />
            }
        "#,
            r#"
            const Component = ({ enabled, checked }) => {
              return <CheckBox checked={enabled ? checked : null} />
            }
        "#,
            None,
        ),
        (
            r#"
            const isOpen = 0;
            const Component = () => {
              return <Popover open={isOpen && items.length > 0} />
            }
        "#,
            r#"
            const isOpen = 0;
            const Component = () => {
              return <Popover open={!!isOpen && items.length > 0} />
            }
        "#,
            Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
        ),
        // ignoreAttributes: still fixes children inside attribute JSX
        (
            r#"
            const Component = ({ enabled }) => {
              return (
                <Foo bar={
                  <Something>{enabled && <MuchWow />}</Something>
                } />
              )
            }
        "#,
            r#"
            const Component = ({ enabled }) => {
              return (
                <Foo bar={
                  <Something>{enabled ? <MuchWow /> : null}</Something>
                } />
              )
            }
        "#,
            Some(serde_json::json!([{ "ignoreAttributes": true }])),
        ),
    ];

    Tester::new(JsxNoLeakedRender::NAME, JsxNoLeakedRender::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
