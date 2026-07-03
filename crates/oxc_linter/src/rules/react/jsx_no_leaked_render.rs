use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{ConditionalExpression, Expression, LogicalExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{LogicalOperator, UnaryOperator};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

/// Operands of an `&&` chain. Real-world JSX chains are nearly always 2–4
/// operands, so this stays on the stack in the common case.
type ChainOperands<'a, 'b> = SmallVec<[&'b Expression<'a>; 4]>;

fn jsx_no_leaked_render_diagnostic(span: Span, help: &'static str) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Potential leaked value that might cause unintentionally rendered values or rendering crashes",
    )
    .with_help(help)
    .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct JsxNoLeakedRenderConfig {
    /// Strategies the user accepts for guarding JSX expressions. The first entry
    /// determines the preferred fix when both strategies are valid.
    valid_strategies: Vec<ValidStrategies>,
    /// Skip JSX attribute values (children of nested JSX inside attributes are
    /// still linted because each `JSXExpressionContainer` is its own node).
    ignore_attributes: bool,
}

impl Default for JsxNoLeakedRenderConfig {
    fn default() -> Self {
        Self {
            valid_strategies: vec![ValidStrategies::Ternary, ValidStrategies::Coerce],
            ignore_attributes: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum ValidStrategies {
    Coerce,
    Ternary,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct JsxNoLeakedRender(Box<JsxNoLeakedRenderConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow problematic leaked values from being rendered in JSX.
    ///
    /// ### Why is this bad?
    ///
    /// Using the `&&` short-circuit operator in JSX can render unintended values
    /// such as `0`, `''`, or `NaN` directly into the DOM. In React Native this
    /// can crash the application. Conditionally render with a ternary returning
    /// `null` (`{x ? <Y/> : null}`) or coerce the gate to a boolean (`{!!x && <Y/>}`).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div>{count && <Results/>}</div>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div>{count ? <Results/> : null}</div>
    /// <div>{!!count && <Results/>}</div>
    /// ```
    JsxNoLeakedRender,
    react,
    pedantic,
    fix,
    config = JsxNoLeakedRender,
    version = "next",
);

impl Rule for JsxNoLeakedRender {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXExpressionContainer(container) = node.kind() else { return };

        let Some(expr) = container.expression.as_expression() else { return };

        if self.0.ignore_attributes
            && matches!(ctx.nodes().parent_kind(node.id()), AstKind::JSXAttribute(_))
        {
            return;
        }

        let strategies = &self.0.valid_strategies;
        let allow_coerce = strategies.contains(&ValidStrategies::Coerce);
        let allow_ternary = strategies.contains(&ValidStrategies::Ternary);
        // The first strategy listed determines which fix shape we emit.
        let prefer_coerce = strategies.first().copied() == Some(ValidStrategies::Coerce);

        match expr.get_inner_expression() {
            Expression::LogicalExpression(logical) if logical.operator == LogicalOperator::And => {
                // React 18+ no longer renders empty strings as text, so an `'' && X`
                // gate is harmless and is not reported under React 18+.
                if let Expression::StringLiteral(s) = logical.left.get_inner_expression()
                    && s.value.as_str().is_empty()
                    && ctx.settings().react.version.is_some_and(|v| v.major() >= 18)
                {
                    return;
                }

                // When `coerce` is not in the user's strategies, every `&&` is a violation.
                // Otherwise, only chains with a leakable non-final gate are violations.
                if allow_coerce && !chain_has_leakable_gate(logical) {
                    return;
                }

                let target_span = logical.span;
                let (help, replacement) = if prefer_coerce {
                    (
                        "Coerce the gate of `&&` to a boolean using `!!`",
                        build_coerce_from_logical(logical, ctx),
                    )
                } else {
                    (
                        "Use a ternary expression returning `null` instead of `&&`",
                        build_ternary_from_logical(logical, ctx),
                    )
                };

                ctx.diagnostic_with_fix(
                    jsx_no_leaked_render_diagnostic(target_span, help),
                    |fixer| fixer.replace(target_span, replacement),
                );
            }
            Expression::ConditionalExpression(cond)
                if allow_coerce && !allow_ternary && conditional_is_violation(cond) =>
            {
                let target_span = cond.span;
                let replacement = build_coerce_from_conditional(cond, ctx);
                ctx.diagnostic_with_fix(
                    jsx_no_leaked_render_diagnostic(
                        target_span,
                        "Coerce the condition to a boolean using `!!` instead of a ternary",
                    ),
                    |fixer| fixer.replace(target_span, replacement),
                );
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

/// An expression is "safe" (its result is a definite boolean) if it is `!x`/`!!x`,
/// a comparison/equality, or a boolean literal. Such operands do not need `!!` coercion.
fn is_safe_boolean(expr: &Expression<'_>) -> bool {
    match expr.get_inner_expression() {
        Expression::UnaryExpression(u) => u.operator == UnaryOperator::LogicalNot,
        Expression::BinaryExpression(b) => b.operator.is_compare() || b.operator.is_equality(),
        Expression::BooleanLiteral(_) => true,
        _ => false,
    }
}

/// Collect the operands of an `&&` chain in source order. `a && b && c` parses
/// as `(a && b) && c`, a left-leaning spine, so we descend into each node's
/// `.left` while it is itself an `&&`, pushing the `.right` operands as we go and
/// the innermost `.left` last, then reverse to get `[a, b, c]`. `get_inner_expression()`
/// lets us flatten through parenthesized and TypeScript-wrapped `&&` nodes.
///
/// Kept iterative on purpose: a pathologically deep chain would overflow the
/// stack under recursion, so we walk the spine with an explicit loop instead.
fn collect_and_chain<'a, 'b>(logical: &'b LogicalExpression<'a>) -> ChainOperands<'a, 'b> {
    let mut operands = ChainOperands::new();
    let mut current = logical;
    loop {
        operands.push(&current.right);
        match current.left.get_inner_expression() {
            Expression::LogicalExpression(inner) if inner.operator == LogicalOperator::And => {
                current = inner;
            }
            _ => {
                operands.push(&current.left);
                break;
            }
        }
    }
    operands.reverse();
    operands
}

/// Returns true if the `&&` chain has at least one non-final gate that is leakable.
/// The last operand is the rendered value, not a gate, so it is excluded.
fn chain_has_leakable_gate(logical: &LogicalExpression<'_>) -> bool {
    let operands = collect_and_chain(logical);
    operands.split_last().is_some_and(|(_, gates)| gates.iter().any(|op| !is_safe_boolean(op)))
}

/// Render an `&&` chain coerce-form into `out`. When `coerce_last` is false, the
/// final operand (the rendered value) is emitted verbatim — the typical `<chain> &&
/// <rendered>` violation. When true, every operand is treated as a gate — used when
/// the chain is the test of a ternary that we are rewriting.
fn write_coerced_chain<'a>(
    operands: &[&Expression<'a>],
    ctx: &LintContext<'a>,
    coerce_last: bool,
    out: &mut String,
) {
    let last_idx = operands.len() - 1;
    for (i, op) in operands.iter().enumerate() {
        if i > 0 {
            out.push_str(" && ");
        }
        let is_gate = coerce_last || i != last_idx;
        if is_gate && !is_safe_boolean(op) {
            out.push_str("!!");
        }
        out.push_str(ctx.source_range(op.span()));
    }
}

/// Pre-size a fix-output buffer with room for the rewritten chain plus a small
/// allowance for `!!` and ` && ` insertions.
fn fix_buffer_for(span: Span, operand_count: usize) -> String {
    String::with_capacity(span.size() as usize + operand_count * 4)
}

/// Build the coerce-form replacement for an `&&` chain expression: wrap each
/// non-final leakable operand with `!!`, leave safe operands and the final
/// (rendered) operand untouched.
fn build_coerce_from_logical<'a>(logical: &LogicalExpression<'a>, ctx: &LintContext<'a>) -> String {
    let operands = collect_and_chain(logical);
    let mut out = fix_buffer_for(logical.span, operands.len());
    write_coerced_chain(&operands, ctx, false, &mut out);
    out
}

/// Build the ternary-form replacement for an `&&` chain expression. Splits the
/// chain at the rightmost `&&` into `<test_chain> ? <rendered> : null`. If the
/// test chain is `!!(x)`, strip the redundant `!!`.
fn build_ternary_from_logical<'a>(
    logical: &LogicalExpression<'a>,
    ctx: &LintContext<'a>,
) -> String {
    let rendered = ctx.source_range(logical.right.span());
    let test_text = ternary_test_text(&logical.left, ctx);
    let mut out = fix_buffer_for(logical.span, 2);
    out.push_str(&test_text);
    out.push_str(" ? ");
    out.push_str(rendered);
    out.push_str(" : null");
    out
}

/// Render the `test` part of a ternary fix. Strips a redundant outer `!!` and any
/// surrounding parentheses left behind by stripping it. Returns borrowed source
/// text in the common case; only allocates when stripping `!!`.
fn ternary_test_text<'a>(test: &Expression<'a>, ctx: &LintContext<'a>) -> Cow<'a, str> {
    if let Expression::UnaryExpression(outer) = test.without_parentheses()
        && outer.operator == UnaryOperator::LogicalNot
        && let Expression::UnaryExpression(inner) = outer.argument.without_parentheses()
        && inner.operator == UnaryOperator::LogicalNot
    {
        let stripped = inner.argument.without_parentheses();
        return Cow::Borrowed(ctx.source_range(stripped.span()));
    }
    Cow::Borrowed(ctx.source_range(test.span()))
}

/// Returns true when a `ConditionalExpression` inside a JSX expression container
/// should be flagged under `coerce`-only strategy. Three cases:
/// - `cond ? cons : null` (alternate is `null`)
/// - `cond ? false : alt` (consequent is the literal `false`)
/// - `<&&-chain> ? cons : alt` whose test has a leakable gate
fn conditional_is_violation(cond: &ConditionalExpression<'_>) -> bool {
    if matches!(&cond.alternate, Expression::NullLiteral(_)) {
        return true;
    }
    if let Expression::BooleanLiteral(b) = &cond.consequent
        && !b.value
    {
        return true;
    }
    if let Expression::LogicalExpression(logical) = cond.test.get_inner_expression()
        && logical.operator == LogicalOperator::And
        && chain_has_leakable_gate(logical)
    {
        return true;
    }
    false
}

/// Build the coerce-form replacement for a `ConditionalExpression` violation.
/// - `cond ? cons : null`: `<maybe-coerced cond> && <cons>`.
/// - `cond ? false : alt`: `!(<cond>) && <alt>` (parenthesizing the test to make
///   `!` bind correctly when `cond` is non-trivial).
/// - `<&&-chain> ? cons : alt`: rewrite test with each leakable gate coerced,
///   keeping the `? cons : alt` shape.
fn build_coerce_from_conditional<'a>(
    cond: &ConditionalExpression<'a>,
    ctx: &LintContext<'a>,
) -> String {
    // Case: alternate is `null` — collapse to `<coerced test> && <cons>`.
    if matches!(&cond.alternate, Expression::NullLiteral(_)) {
        return collapse_to_coerced_and(&cond.test, &cond.consequent, cond.span, ctx);
    }
    // Case: consequent is `false` — collapse to `!(<test>) && <alt>`.
    if let Expression::BooleanLiteral(b) = &cond.consequent
        && !b.value
    {
        let test_text = ctx.source_range(cond.test.span());
        let alternate_text = ctx.source_range(cond.alternate.span());
        let mut out = fix_buffer_for(cond.span, 2);
        out.push('!');
        out.push('(');
        out.push_str(test_text);
        out.push_str(") && ");
        out.push_str(alternate_text);
        return out;
    }
    // Case: test is an `&&` chain with leakable gates — rewrite test only.
    if let Expression::LogicalExpression(logical) = cond.test.get_inner_expression()
        && logical.operator == LogicalOperator::And
    {
        let operands = collect_and_chain(logical);
        let consequent_text = ctx.source_range(cond.consequent.span());
        let alternate_text = ctx.source_range(cond.alternate.span());
        let mut out = fix_buffer_for(cond.span, operands.len() + 2);
        write_coerced_chain(&operands, ctx, true, &mut out);
        out.push_str(" ? ");
        out.push_str(consequent_text);
        out.push_str(" : ");
        out.push_str(alternate_text);
        return out;
    }
    // Fallback (shouldn't happen given `conditional_is_violation` filter): leave unchanged.
    ctx.source_range(cond.span).to_owned()
}

/// Helper for the `cond ? cons : null` collapse case.
fn collapse_to_coerced_and<'a>(
    test: &Expression<'a>,
    consequent: &Expression<'a>,
    span: Span,
    ctx: &LintContext<'a>,
) -> String {
    let consequent_text = ctx.source_range(consequent.span());
    if let Expression::LogicalExpression(logical) = test.get_inner_expression()
        && logical.operator == LogicalOperator::And
    {
        let operands = collect_and_chain(logical);
        let mut out = fix_buffer_for(span, operands.len() + 1);
        write_coerced_chain(&operands, ctx, true, &mut out);
        out.push_str(" && ");
        out.push_str(consequent_text);
        return out;
    }
    let test_text = ctx.source_range(test.span());
    let mut out = fix_buffer_for(span, 2);
    if !is_safe_boolean(test) {
        out.push_str("!!");
    }
    out.push_str(test_text);
    out.push_str(" && ");
    out.push_str(consequent_text);
    out
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![];

    let fail = vec![
        ("
                    const Example = () => {
                      return (
                        <>
                          {0 && <Something/>}
                          {'' && <Something/>}
                          {NaN && <Something/>}
                        </>
                      )
                    }
                  ", None, Some(serde_json::json!({ "settings": { "react": { "version": "17.999.999" } } }))),
        ("
                    const Example = () => {
                      return (
                        <>
                          {0 && <Something/>}
                          {'' && <Something/>}
                          {NaN && <Something/>}
                        </>
                      )
                    }
                  ", None, Some(serde_json::json!({ "settings": { "react": { "version": "18.0.0" } } }))),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", None, None),
        ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", None, None),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", None, None),
        ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", None, None),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", None, None),
        ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", None, None),
        ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])), None),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ someCondition, title }) => {
                      return <div>{!someCondition && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{!!count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count > 0 && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{0 != count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ count, total, title }) => {
                      return <div>{count < total && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ count, title, somethingElse }) => {
                      return <div>{!!(count && somethingElse) && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ connection, hasError, hasErrorUpdate}) => {
                      return <div>{connection && (hasError || hasErrorUpdate)}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{!count ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ count, somethingElse, title }) => {
                      return <div>{count && somethingElse ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ items, somethingElse, title }) => {
                      return <div>{items.length > 0 && somethingElse && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const MyComponent = () => {
                      const items = []
                      const breakpoint = { phones: true }

                      return <div>{items.length > 0 && breakpoint.phones && <span />}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])), None),
        ("
                    const MyComponent = () => {
                      return <div>{maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ enabled, checked }) => {
                      return <CheckBox checked={enabled && checked} />
                    }
                  ", None, None),
        ("
                    const MyComponent = () => {
                      return <Something checked={isIndeterminate ? false : isChecked} />
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const MyComponent = () => {
                      return <Something checked={cond && isIndeterminate ? false : isChecked} />
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const isOpen = 0;
                    const Component = () => {
                      return <Popover open={isOpen && items.length > 0} />
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
        ("
                    const Component = ({ enabled }) => {
                      return (
                        <Foo bar={
                          <Something>{enabled && <MuchWow />}</Something>
                        } />
                      )
                    }
                  ", Some(serde_json::json!([{ "ignoreAttributes": true }])), None)
    ];

    let fix = vec![
        ("
                    const Example = () => {
                      return (
                        <>
                          {0 && <Something/>}
                          {'' && <Something/>}
                          {NaN && <Something/>}
                        </>
                      )
                    }
                  ", "
                    const Example = () => {
                      return (
                        <>
                          {0 ? <Something/> : null}
                          {'' ? <Something/> : null}
                          {NaN ? <Something/> : null}
                        </>
                      )
                    }
                  ", None, Some(serde_json::json!({ "settings": { "react": { "version": "17.999.999" } } }))),
        ("
                    const Example = () => {
                      return (
                        <>
                          {0 && <Something/>}
                          {'' && <Something/>}
                          {NaN && <Something/>}
                        </>
                      )
                    }
                  ", "
                    const Example = () => {
                      return (
                        <>
                          {0 ? <Something/> : null}
                          {'' && <Something/>}
                          {NaN ? <Something/> : null}
                        </>
                      )
                    }
                  ", None, Some(serde_json::json!({ "settings": { "react": { "version": "18.0.0" } } }))),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{count ? title : null}</div>
                    }
                  ", None, None),
        ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", "
                    const Component = ({ count }) => {
                      return <div>{count ? <span>There are {count} results</span> : null}</div>
                    }
                  ", None, None),
    ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{elements.length ? <List elements={elements}/> : null}</div>
                    }
                  ", None, None),
    ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", "
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length ? <List elements={nestedCollection.elements}/> : null}</div>
                    }
                  ", None, None),
    ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{elements[0] ? <List elements={elements}/> : null}</div>
                    }
                  ", None, None),
    ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", "
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) ? <Results>{numberA+numberB}</Results> : null}</div>
                    }
                  ", None, None),
    ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", "
                    const Component = ({ numberA, numberB }) => {
                      return <div>{!!(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])), None),
    ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{count ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", "
                    const Component = ({ count }) => {
                      return <div>{count ? <span>There are {count} results</span> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{elements.length ? <List elements={elements}/> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", "
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length ? <List elements={nestedCollection.elements}/> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{elements[0] ? <List elements={elements}/> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", "
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) ? <Results>{numberA+numberB}</Results> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ someCondition, title }) => {
                      return <div>{!someCondition && title}</div>
                    }
                  ", "
                    const Component = ({ someCondition, title }) => {
                      return <div>{!someCondition ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ count, title }) => {
                      return <div>{!!count && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{count ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ count, title }) => {
                      return <div>{count > 0 && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{count > 0 ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ count, title }) => {
                      return <div>{0 != count && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{0 != count ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ count, total, title }) => {
                      return <div>{count < total && title}</div>
                    }
                  ", "
                    const Component = ({ count, total, title }) => {
                      return <div>{count < total ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ count, title, somethingElse }) => {
                      return <div>{!!(count && somethingElse) && title}</div>
                    }
                  ", "
                    const Component = ({ count, title, somethingElse }) => {
                      return <div>{count && somethingElse ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])), None),
    ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{!!count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", "
                    const Component = ({ count }) => {
                      return <div>{!!count && <span>There are {count} results</span>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{!!elements.length && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", "
                    const Component = ({ nestedCollection }) => {
                      return <div>{!!nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{!!elements[0] && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", "
                    const Component = ({ numberA, numberB }) => {
                      return <div>{!!(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ connection, hasError, hasErrorUpdate}) => {
                      return <div>{connection && (hasError || hasErrorUpdate)}</div>
                    }
                  ", "
                    const Component = ({ connection, hasError, hasErrorUpdate}) => {
                      return <div>{!!connection && (hasError || hasErrorUpdate)}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ count, title }) => {
                      return <div>{count ? title : null}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{!!count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ count, title }) => {
                      return <div>{!count ? title : null}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{!count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ count, somethingElse, title }) => {
                      return <div>{count && somethingElse ? title : null}</div>
                    }
                  ", "
                    const Component = ({ count, somethingElse, title }) => {
                      return <div>{!!count && !!somethingElse && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ items, somethingElse, title }) => {
                      return <div>{items.length > 0 && somethingElse && title}</div>
                    }
                  ", "
                    const Component = ({ items, somethingElse, title }) => {
                      return <div>{items.length > 0 && !!somethingElse && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const MyComponent = () => {
                      const items = []
                      const breakpoint = { phones: true }

                      return <div>{items.length > 0 && breakpoint.phones && <span />}</div>
                    }
                  ", "
                    const MyComponent = () => {
                      const items = []
                      const breakpoint = { phones: true }

                      return <div>{items.length > 0 && !!breakpoint.phones && <span />}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])), None),
    ("
                    const MyComponent = () => {
                      return <div>{maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
                    }
                  ", "
                    const MyComponent = () => {
                      return <div>{!!maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ enabled, checked }) => {
                      return <CheckBox checked={enabled && checked} />
                    }
                  ", "
                    const Component = ({ enabled, checked }) => {
                      return <CheckBox checked={enabled ? checked : null} />
                    }
                  ", None, None),
    ("
                    const isOpen = 0;
                    const Component = () => {
                      return <Popover open={isOpen && items.length > 0} />
                    }
                  ", "
                    const isOpen = 0;
                    const Component = () => {
                      return <Popover open={!!isOpen && items.length > 0} />
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])), None),
    ("
                    const Component = ({ enabled }) => {
                      return (
                        <Foo bar={
                          <Something>{enabled && <MuchWow />}</Something>
                        } />
                      )
                    }
                  ", "
                    const Component = ({ enabled }) => {
                      return (
                        <Foo bar={
                          <Something>{enabled ? <MuchWow /> : null}</Something>
                        } />
                      )
                    }
                  ", Some(serde_json::json!([{ "ignoreAttributes": true }])), None)
    ];

    Tester::new(JsxNoLeakedRender::NAME, JsxNoLeakedRender::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
