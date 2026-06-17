use oxc_ast::{
    AstKind,
    ast::{
        Expression, IdentifierReference, JSXExpression, JSXExpressionContainer, LogicalOperator,
        PropertyKey::JSXElement,
    },
};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn jsx_no_leaked_render_diagnostic(
    span: Span,
    condition_span: Span,
    ctx: &LintContext,
) -> OxcDiagnostic {
    let source = ctx.source_range(condition_span);
    OxcDiagnostic::warn("Potential leaked value that might cause unintentionally rendered values")
        .with_help(format!("Coerce the conditional to a boolean (`!!{source} && ...`) or use a ternary (`{source} ? ... : null`)."))
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
struct ConfigElement0 {
    /// Which strategies are considered valid for conditional rendering.
    ///
    /// Defaults to allowing both `"ternary"` and `"coerce"`.
    valid_strategies: FxHashSet<ValidStrategies>,
    /// When `true`, logical expressions inside non-`children` attributes are ignored.
    /// Expressions assigned to `children` and any nested JSX are still checked.
    ignore_attributes: bool,
}

impl Default for ConfigElement0 {
    fn default() -> Self {
        Self {
            valid_strategies: [ValidStrategies::Ternary, ValidStrategies::Coerce]
                .into_iter()
                .collect(),
            ignore_attributes: false,
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct JsxNoLeakedRender(ConfigElement0);

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
    pending,
    config = JsxNoLeakedRender,
    version = "next",
    short_description = "Disallow problematic leaked values from being rendered in JSX.",
);

impl Rule for JsxNoLeakedRender {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match &node.kind() {
            AstKind::JSXExpressionContainer(container) => {
                dbg!(container);

                let JSXExpression::LogicalExpression(expr) = &container.expression else {
                    return;
                };

                if !matches!(expr.operator, LogicalOperator::And) {
                    return;
                }

                match &expr.left {
                    Expression::StringLiteral(str) => {
                        if str.value.is_empty() {
                            ctx.diagnostic(jsx_no_leaked_render_diagnostic(
                                expr.span, str.span, ctx,
                            ));
                        }
                    }
                    Expression::NumericLiteral(num) => {
                        if num.value == 0.0 {
                            ctx.diagnostic(jsx_no_leaked_render_diagnostic(
                                expr.span, num.span, ctx,
                            ));
                        }
                    }
                    Expression::Identifier(ident) => {
                        ctx.diagnostic(jsx_no_leaked_render_diagnostic(expr.span, ident.span, ctx));
                    }
                    Expression::StaticMemberExpression(member) => {
                        ctx.diagnostic(jsx_no_leaked_render_diagnostic(
                            expr.span,
                            member.span,
                            ctx,
                        ));
                    }
                    Expression::ComputedMemberExpression(computed_member) => {
                        ctx.diagnostic(jsx_no_leaked_render_diagnostic(
                            expr.span,
                            computed_member.span,
                            ctx,
                        ));
                    }
                    Expression::ParenthesizedExpression(parens_expr) => {
                        if !matches!(parens_expr.expression, Expression::LogicalExpression(_)) {
                            return;
                        };

                        ctx.diagnostic(jsx_no_leaked_render_diagnostic(
                            expr.span,
                            parens_expr.span,
                            ctx,
                        ));
                    }
                    _ => return,
                }
            }
            _ => return,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let _pass: Vec<(&str, Option<Value>, Option<Value>)> = vec![
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
    ];

    let fail: Vec<(&str, Option<Value>, Option<Value>)> = vec![
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

    let _fix: Vec<(&str, &str, Option<Value>)> = vec![
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
                  None
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
                  None
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
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }]))),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{count ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", "
                    const Component = ({ count }) => {
                      return <div>{count ? <span>There are {count} results</span> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{elements.length ? <List elements={elements}/> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", "
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length ? <List elements={nestedCollection.elements}/> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{elements[0] ? <List elements={elements}/> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", "
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) ? <Results>{numberA+numberB}</Results> : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ someCondition, title }) => {
                      return <div>{!someCondition && title}</div>
                    }
                  ", "
                    const Component = ({ someCondition, title }) => {
                      return <div>{!someCondition ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{!!count && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{count ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count > 0 && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{count > 0 ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{0 != count && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{0 != count ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ count, total, title }) => {
                      return <div>{count < total && title}</div>
                    }
                  ", "
                    const Component = ({ count, total, title }) => {
                      return <div>{count < total ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ count, title, somethingElse }) => {
                      return <div>{!!(count && somethingElse) && title}</div>
                    }
                  ", "
                    const Component = ({ count, title, somethingElse }) => {
                      return <div>{count && somethingElse ? title : null}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["ternary"] }]))),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count && title}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{!!count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
        ("
                    const Component = ({ count }) => {
                      return <div>{count && <span>There are {count} results</span>}</div>
                    }
                  ", "
                    const Component = ({ count }) => {
                      return <div>{!!count && <span>There are {count} results</span>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements.length && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{!!elements.length && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
        ("
                    const Component = ({ nestedCollection }) => {
                      return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", "
                    const Component = ({ nestedCollection }) => {
                      return <div>{!!nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
        ("
                    const Component = ({ elements }) => {
                      return <div>{elements[0] && <List elements={elements}/>}</div>
                    }
                  ", "
                    const Component = ({ elements }) => {
                      return <div>{!!elements[0] && <List elements={elements}/>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
        ("
                    const Component = ({ numberA, numberB }) => {
                      return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", "
                    const Component = ({ numberA, numberB }) => {
                      return <div>{!!(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
        ("
                    const Component = ({ connection, hasError, hasErrorUpdate}) => {
                      return <div>{connection && (hasError || hasErrorUpdate)}</div>
                    }
                  ", "
                    const Component = ({ connection, hasError, hasErrorUpdate}) => {
                      return <div>{!!connection && (hasError || hasErrorUpdate)}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{count ? title : null}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{!!count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
        ("
                    const Component = ({ count, title }) => {
                      return <div>{!count ? title : null}</div>
                    }
                  ", "
                    const Component = ({ count, title }) => {
                      return <div>{!count && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
        ("
                    const Component = ({ count, somethingElse, title }) => {
                      return <div>{count && somethingElse ? title : null}</div>
                    }
                  ", "
                    const Component = ({ count, somethingElse, title }) => {
                      return <div>{!!count && !!somethingElse && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
        ("
                    const Component = ({ items, somethingElse, title }) => {
                      return <div>{items.length > 0 && somethingElse && title}</div>
                    }
                  ", "
                    const Component = ({ items, somethingElse, title }) => {
                      return <div>{items.length > 0 && !!somethingElse && title}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
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
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce", "ternary"] }]))),
        ("
                    const MyComponent = () => {
                      return <div>{maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
                    }
                  ", "
                    const MyComponent = () => {
                      return <div>{!!maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
                    }
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
        ("
                    const Component = ({ enabled, checked }) => {
                      return <CheckBox checked={enabled && checked} />
                    }
                  ", "
                    const Component = ({ enabled, checked }) => {
                      return <CheckBox checked={enabled ? checked : null} />
                    }
                  ",
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
                  ", Some(serde_json::json!([{ "validStrategies": ["coerce"] }]))),
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
                  ", Some(serde_json::json!([{ "ignoreAttributes": true }])))
    ];

    Tester::new(
        JsxNoLeakedRender::NAME,
        JsxNoLeakedRender::PLUGIN,
        // pass,
        vec![],
        fail,
    )
    // .expect_fix(fix)
    .test_and_snapshot();
}
