use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference, JSXExpression, LogicalOperator},
};
use oxc_semantic::NodeId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::check_react_version,
};

fn jsx_no_leaked_render_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Potential leaked value that might cause unintentionally rendered values")
        .with_help(
            "Coerce the conditional to a boolean (e.g. `!!cond && ...`) or use a ternary (e.g. `cond ? ... : null`).",
        )
        .with_label(span)
}

/// Strategies that are considered safe ways to write a conditional render.
///
/// The first strategy listed is the one used by the autofix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum ValidStrategies {
    /// Coerce the left-hand side of the `&&` to a boolean, e.g. `!!cond && <Foo />`.
    Coerce,
    /// Use a ternary expression with an explicit `null` alternate, e.g. `cond ? <Foo /> : null`.
    Ternary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct JsxNoLeakedRenderConfig {
    /// Which strategies are considered valid for conditional rendering.
    ///
    /// Defaults to allowing both `"ternary"` and `"coerce"`.
    valid_strategies: Vec<ValidStrategies>,
    /// When `true`, logical expressions inside non-`children` attributes are ignored.
    /// Expressions assigned to `children` and any nested JSX are still checked.
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

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct JsxNoLeakedRender(Box<JsxNoLeakedRenderConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents problematic leaked values from being rendered in JSX when using the
    /// `&&` short-circuit operator for conditional rendering.
    ///
    /// ### Why is this bad?
    ///
    /// When the left-hand side of a logical `&&` is a falsy value such as `0`, `NaN`
    /// or `''`, that value is returned instead of being treated as "render nothing".
    /// In React on the web this renders the value (e.g. a stray `0`), and in React
    /// Native it can crash the render because strings/numbers must be wrapped in a
    /// `<Text>` component. Coercing the condition to a boolean, or using an explicit
    /// ternary, avoids leaking these values into the output.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// const Component = ({ count, title }) => <div>{count && title}</div>;
    /// const Example = () => <>{0 && <Something />}</>;
    ///
    /// const Component = ({ elements }) => {
    ///     return <div>{elements.length && <List elements={elements} />}</div>
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const Component = ({ count, title }) => <div>{!!count && title}</div>;
    /// const Example = () => <>{count ? <Something /> : null}</>;
    ///
    /// const Component = ({ elements }) => {
    ///     return <div>{!!elements.length && <List elements={elements} />}</div>
    /// }
    /// ```
    JsxNoLeakedRender,
    react,
    correctness,
    fix,
    config = JsxNoLeakedRender,
    version = "next",
    short_description = "Disallow problematic leaked values from being rendered in JSX.",
);

impl Rule for JsxNoLeakedRender {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXExpressionContainer(container) = node.kind() else {
            return;
        };

        match &container.expression {
            JSXExpression::LogicalExpression(expr) => {
                if self.0.ignore_attributes && is_within_attribute(node.id(), ctx) {
                    return;
                }

                if !matches!(expr.operator, LogicalOperator::And) {
                    return;
                }

                let coerce_allowed = self.0.valid_strategies.contains(&ValidStrategies::Coerce);

                if !coerce_allowed || !is_coerce_safe(&expr.left, ctx) {
                    let is_react_18_plus =
                        check_react_version(ctx.settings().react.version.as_ref(), 18, 0);
                    if is_react_18_plus
                        && matches!(expr.left.get_inner_expression(), Expression::StringLiteral(s) if s.value.is_empty())
                    {
                        return;
                    }

                    ctx.diagnostic_with_fix(jsx_no_leaked_render_diagnostic(expr.span), |fixer| {
                        let is_coerce_first =
                            self.0.valid_strategies.first() == Some(&ValidStrategies::Coerce);
                        let rendered = ctx.source_range(expr.right.span());

                        let replacement = if is_coerce_first {
                            let condition = coerce_fix_text(&expr.left, ctx);
                            format!("{condition} && {rendered}")
                        } else {
                            let condition = ctx.source_range(trim_left(&expr.left).span());
                            format!("{condition} ? {rendered} : null")
                        };

                        fixer.replace(expr.span, replacement)
                    });
                }
            }
            JSXExpression::ConditionalExpression(expr) => {
                if self.0.ignore_attributes && is_within_attribute(node.id(), ctx) {
                    return;
                }

                if self.0.valid_strategies.contains(&ValidStrategies::Ternary) {
                    return;
                }

                if matches!(expr.alternate, Expression::NullLiteral(_) | Expression::Identifier(_))
                {
                    ctx.diagnostic_with_fix(
                        jsx_no_leaked_render_diagnostic(container.expression.span()),
                        |fixer| {
                            let is_coerce_first =
                                self.0.valid_strategies.first() == Some(&ValidStrategies::Coerce);
                            let rendered = ctx.source_range(expr.consequent.span());

                            let replacement = if is_coerce_first {
                                let condition = coerce_fix_text(&expr.test, ctx);
                                format!("{condition} && {rendered}")
                            } else {
                                let condition = ctx.source_range(trim_left(&expr.test).span());
                                format!("{condition} ? {rendered} : null")
                            };

                            fixer.replace(container.expression.span(), replacement)
                        },
                    );
                }
            }
            _ => {}
        }
    }
}

fn is_coerce_safe(expr: &Expression, ctx: &LintContext) -> bool {
    match &expr.get_inner_expression() {
        Expression::LogicalExpression(e) => {
            is_coerce_safe(&e.left, ctx) && is_coerce_safe(&e.right, ctx)
        }
        Expression::UnaryExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::CallExpression(_) => true,
        Expression::Identifier(ident) => {
            if let Some(declaration) = resolve_declaration(ident, ctx)
                && let AstKind::VariableDeclarator(decl) = declaration.kind()
                && let Some(init) = &decl.init
                && matches!(init.get_inner_expression(), Expression::BooleanLiteral(_))
            {
                return true;
            }

            false
        }
        _ => false,
    }
}

fn resolve_declaration<'c, 'a>(
    ident: &IdentifierReference,
    ctx: &'c LintContext<'a>,
) -> Option<&'c AstNode<'a>> {
    let reference = ctx.scoping().get_reference(ident.reference_id());
    let symbol_id = reference.symbol_id()?; // None for globals/unresolved
    let decl_node_id = ctx.scoping().symbol_declaration(symbol_id);
    Some(ctx.nodes().get_node(decl_node_id))
}

fn is_within_attribute(id: NodeId, ctx: &LintContext) -> bool {
    for ancestor in ctx.nodes().ancestors(id) {
        match ancestor.kind() {
            AstKind::JSXAttribute(_) => return true,
            AstKind::JSXElement(_) | AstKind::JSXFragment(_) => return false,
            _ => {}
        }
    }

    false
}

fn trim_left<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    if let Expression::UnaryExpression(outer) = expr
        && let Expression::UnaryExpression(inner) = &outer.argument
    {
        return trim_left(inner.argument.get_inner_expression());
    }

    expr
}

fn coerce_fix_text(expr: &Expression, ctx: &LintContext) -> String {
    if let Expression::LogicalExpression(e) = expr
        && e.operator == LogicalOperator::And
    {
        let left = coerce_fix_text(&e.left, ctx);
        let right = coerce_fix_text(&e.right, ctx);
        return format!("{left} && {right}");
    }

    let source = ctx.source_range(expr.span());
    if is_coerce_safe(expr, ctx) {
        return source.to_string();
    }
    format!("!!{source}")
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("
                    const Component = () => {
                      return <div>{customTitle || defaultTitle}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>There are {elements.length} elements</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ elements, count }) => {
                      return <div>{!count && 'No results found'}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{!!elements.length && <List elements={elements}/>}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{Boolean(elements.length) && <List elements={elements}/>}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length > 0 && <List elements={elements}/>}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length ? <List elements={elements}/> : null}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ elements, count }) => {
                      return <div>{count ? <List elements={elements}/> : null}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ elements, count }) => {
                      return <div>{count ? <List elements={elements}/> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ elements, count }) => {
                      return <div>{!!count && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ elements, count }) => {
                      return <div>{count ? <List elements={elements}/> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ elements, count }) => {
                      return <div>{!!count && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ elements, count }) => {
                      return (
                        <div>
                          <div> {direction ? (direction === \"down\" ? \"▼\" : \"▲\") : \"\"} </div>
                          <div>{ containerName.length > 0 ? \"Loading several stuff\" : \"Loading\" }</div>
                        </div>
                      )
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ elements, count }) => {
                      return <div>{direction ? (direction === \"down\" ? \"▼\" : \"▲\") : \"\"}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ direction }) => {
                      return (
                        <div>
                          <div>{!!direction && direction === \"down\" && \"▼\"}</div>
                          <div>{direction === \"down\" && !!direction && \"▼\"}</div>
                          <div>{direction === \"down\" || !!direction && \"▼\"}</div>
                          <div>{(!display || display === DISPLAY.WELCOME) && <span>foo</span>}</div>
                        </div>
                      )
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ elements, count }) => {
                      return <div>{count ? <List elements={elements}/> : <EmptyList />}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ elements, count }) => {
                      return <div>{count ? <List elements={elements}/> : <EmptyList />}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const isOpen = true;
                    const Component = () => {
                      return <Popover open={isOpen && items.length > 0} />
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const isOpen = false;
                    const Component = () => {
                      return <Popover open={isOpen && items.length > 0} />
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ enabled, checked }) => {
                      return <CheckBox checked={enabled && checked} />
                    }
                  ", Some(serde_json::json!([{ "ignoreAttributes": true }])),
                  None
        ),
        // Known false negative, kept for parity with upstream `eslint-plugin-react`.
        //
        // `Number(elements.length)` can evaluate to `0` and leak it, so this should
        // ideally be reported. However, the coerce check is purely syntactic: any
        // `CallExpression` is treated as a valid (boolean-producing) left side,
        // regardless of which function is called. Upstream does the same
        ("
                    const Component = ({ elements }) => {
                      return <div>{Number(elements.length) && <List elements={elements}/>}</div>
                    }
                  ", None,
                  None
        ),
        // Sibling case for `UnaryExpression`: upstream treats *any* unary operator as
        // a valid left side, not just `!`. `-count`/`+count`/`~count` can still be `0`
        // and leak, but they are not reported
        ("
                    const Component = ({ count }) => {
                      return <div>{-count && <List/>}</div>
                    }
                  ", None,
                  None
        ),
    ];

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
                  ",
                  None,
                  Some(serde_json::json!({ "settings": { "react": { "version": "17.999.999" } } }))
        ),
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
                  ",
                None,
                Some(serde_json::json!({ "settings": { "react": { "version": "18.0.0" } } }))
        ),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", None,
                  None
        ),
        ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ someCondition, title }) => {
                      return <div>{!someCondition && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{!!count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count > 0 && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{0 != count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ count, total, title }) => {
                      return <div>{count < total && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ count, title, somethingElse }) => {
                      return <div>{!!(count && somethingElse) && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }])),
                  None
        ),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ connection, hasError, hasErrorUpdate}) => {
                      return <div>{connection && (hasError || hasErrorUpdate)}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{!count ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ count, somethingElse, title }) => {
                      return <div>{count && somethingElse ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ items, somethingElse, title }) => {
                      return <div>{items.length > 0 && somethingElse && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const MyComponent = () => {
                      const items = []
                      const breakpoint = { phones: true }

                      return <div>{items.length > 0 && breakpoint.phones && <span />}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }])),
                  None
        ),
        ("
                    const MyComponent = () => {
                      return <div>{maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const Component = ({ enabled, checked }) => {
                      return <CheckBox checked={enabled && checked} />
                    }
                  ", None,
                  None
        ),
        ("
                    const MyComponent = () => {
                      return <Something checked={isIndeterminate ? false : isChecked} />
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const MyComponent = () => {
                      return <Something checked={cond && isIndeterminate ? false : isChecked} />
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
        ("
                    const isOpen = 0;
                    const Component = () => {
                      return <Popover open={isOpen && items.length > 0} />
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }])),
                  None
        ),
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
                  ",
                  None,
                  Some(serde_json::json!({ "settings": { "react": { "version": "17.999.999" } } }))
        ),
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
                  ",
                  None,
                  Some(serde_json::json!({ "settings": { "react": { "version": "18.0.0" } } }))
        ),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{count ? title : null}</div>
                    }
                  ",
                  None,
                  None
        ),
        ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", "
                    const Component = ({ count }) => {
                      return <div>{count ? <span>There are {count} results</span> : null}</div>
                    }
                  ",
                  None,
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{elements.length ? <List elements={elements}/> : null}</div>
                    }
                  ",
                  None,
                  None
        ),
        ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", "
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length ? <List elements={nestedCollection.elements}/> : null}</div>
                    }
                  ",
                  None,
                  None
        ),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{elements[0] ? <List elements={elements}/> : null}</div>
                    }
                  ",
                  None,
                  None
        ),
        ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", "
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) ? <Results>{numberA+numberB}</Results> : null}</div>
                    }
                  ",
                  None,
                  None
        ),
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
                  ",
                  None,
                  None
        ),
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
                  ",
                  "
                    const Component = ({ enabled }) => {
                      return (
                        <Foo bar={
                          <Something>{enabled ? <MuchWow /> : null}</Something>
                        } />
                      )
                    }
                  ",
                  Some(serde_json::json!([{ "ignoreAttributes": true }])),
                  None
        ),
    ];

    Tester::new(JsxNoLeakedRender::NAME, JsxNoLeakedRender::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
