use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{Expression, JSXChild},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn jsx_max_depth_diagnostic(depth: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "JSX nesting depth of {depth} exceeds the configured maximum of {max}"
    ))
    .with_label(span)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JsxMaxDepth(Box<JsxMaxDepthConfig>);

impl std::ops::Deref for JsxMaxDepth {
    type Target = JsxMaxDepthConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct JsxMaxDepthConfig {
    #[serde(default = "JsxMaxDepthConfig::default_max")]
    pub max: usize,
}

impl JsxMaxDepthConfig {
    const fn default_max() -> usize {
        2
    }
}

impl Default for JsxMaxDepthConfig {
    fn default() -> Self {
        Self { max: Self::default_max() }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a maximum depth for nested JSX elements and fragments.
    ///
    /// ### Why is this bad?
    ///
    /// Excessively nested JSX makes components harder to read and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// const Component = () => (
    ///   <div>
    ///     <div>
    ///       <div>
    ///         <span />
    ///       </div>
    ///     </div>
    ///   </div>
    /// );
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const Component = () => (
    ///   <div>
    ///     <div>
    ///       <span />
    ///     </div>
    ///   </div>
    /// );
    /// ```
    ///
    /// ### Options
    ///
    /// `react/jsx-max-depth: [<enabled>, { "max": <number> }]`
    ///
    /// The `max` option defaults to `2`.
    JsxMaxDepth,
    react,
    style,
    config = JsxMaxDepthConfig,
);

impl Rule for JsxMaxDepth {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .map(DefaultRuleConfig::into_inner)
            .unwrap_or_default())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(_) | AstKind::JSXFragment(_) => {}
            _ => return,
        }

        if !is_leaf_jsx_node(node) {
            return;
        }

        let ancestor_depth = jsx_ancestor_depth(node, ctx);

        let mut visited_symbols = FxHashSet::default();
        let child_depth = calculate_node_jsx_depth(node, ctx, &mut visited_symbols);

        let total_depth = ancestor_depth + child_depth;

        if total_depth > self.max {
            ctx.diagnostic(jsx_max_depth_diagnostic(total_depth, self.max, node.span()));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

fn calculate_variable_jsx_depth(
    symbol_id: SymbolId,
    ctx: &LintContext<'_>,
    visited: &mut FxHashSet<SymbolId>,
) -> usize {
    if !visited.insert(symbol_id) {
        return 0;
    }

    let decl_id = ctx.semantic().scoping().symbol_declaration(symbol_id);
    let decl_node = ctx.nodes().get_node(decl_id);

    if let AstKind::VariableDeclarator(declarator) = decl_node.kind()
        && let Some(init) = &declarator.init
    {
        return calculate_expression_jsx_depth(init, ctx, visited);
    }

    0
}

fn calculate_expression_jsx_depth(
    expr: &Expression<'_>,
    ctx: &LintContext<'_>,
    visited: &mut FxHashSet<SymbolId>,
) -> usize {
    match expr {
        Expression::JSXElement(elem) => calculate_jsx_children_depth(&elem.children, ctx, visited),
        Expression::JSXFragment(frag) => calculate_jsx_children_depth(&frag.children, ctx, visited),
        Expression::Identifier(ident) => ident
            .reference_id
            .get()
            .and_then(|ref_id| ctx.semantic().scoping().get_reference(ref_id).symbol_id())
            .map_or(0, |symbol_id| calculate_variable_jsx_depth(symbol_id, ctx, visited)),
        Expression::ParenthesizedExpression(paren) => {
            calculate_expression_jsx_depth(&paren.expression, ctx, visited)
        }
        _ => 0,
    }
}

fn calculate_node_jsx_depth(
    node: &AstNode<'_>,
    ctx: &LintContext<'_>,
    visited_symbols: &mut FxHashSet<SymbolId>,
) -> usize {
    let children = match node.kind() {
        AstKind::JSXElement(elem) => &elem.children,
        AstKind::JSXFragment(frag) => &frag.children,
        _ => return 0,
    };
    calculate_jsx_children_depth(children, ctx, visited_symbols)
}

fn calculate_jsx_children_depth(
    children: &[JSXChild<'_>],
    ctx: &LintContext<'_>,
    visited_symbols: &mut FxHashSet<SymbolId>,
) -> usize {
    if children.is_empty() {
        return 0;
    }

    let mut max_depth = 0;

    for child in children {
        let depth = match child {
            JSXChild::Element(elem) => {
                calculate_jsx_children_depth(&elem.children, ctx, visited_symbols) + 1
            }
            JSXChild::Fragment(frag) => {
                calculate_jsx_children_depth(&frag.children, ctx, visited_symbols) + 1
            }
            JSXChild::ExpressionContainer(container) => {
                if let Some(Expression::Identifier(ident)) = container.expression.as_expression() {
                    let depth = ident
                        .reference_id
                        .get()
                        .and_then(|ref_id| {
                            ctx.semantic().scoping().get_reference(ref_id).symbol_id()
                        })
                        .map_or(0, |symbol_id| {
                            calculate_variable_jsx_depth(symbol_id, ctx, visited_symbols)
                        });
                    if depth > 0 { depth + 1 } else { 0 }
                } else {
                    0
                }
            }
            _ => 0,
        };
        max_depth = max_depth.max(depth);
    }

    max_depth
}

fn is_leaf_jsx_node(node: &AstNode<'_>) -> bool {
    let children = match node.kind() {
        AstKind::JSXElement(elem) => &elem.children,
        AstKind::JSXFragment(frag) => &frag.children,
        _ => return true,
    };

    !children.iter().any(|child| matches!(child, JSXChild::Element(_) | JSXChild::Fragment(_)))
}

fn jsx_ancestor_depth(node: &AstNode<'_>, ctx: &LintContext<'_>) -> usize {
    ctx.nodes()
        .ancestors(node.id())
        .filter(|ancestor| {
            matches!(ancestor.kind(), AstKind::JSXElement(_) | AstKind::JSXFragment(_))
        })
        .count()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			        <App />
			      ",
            None,
        ),
        (
            "
			        <App>
			          <foo />
			        </App>
			      ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
			        <App>
			          <foo>
			            <bar />
			          </foo>
			        </App>
			      ",
            None,
        ),
        (
            "
			        <App>
			          <foo>
			            <bar />
			          </foo>
			        </App>
			      ",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "
			        const x = <div><em>x</em></div>;
			        <div>{x}</div>
			      ",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        ("const foo = (x) => <div><em>{x}</em></div>;", Some(serde_json::json!([{ "max": 2 }]))),
        (
            "
			        <></>
			      ",
            None,
        ),
        (
            "
			        <>
			          <foo />
			        </>
			      ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
			        const x = <><em>x</em></>;
			        <>{x}</>
			      ",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "
			        const x = (
			          <tr>
			            <td>1</td>
			            <td>2</td>
			          </tr>
			        );
			        <tbody>
			          {x}
			        </tbody>
			      ",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "
			        const Example = props => {
			          for (let i = 0; i < length; i++) {
			            return <Text key={i} />;
			          }
			        };
			      ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
			        export function MyComponent() {
			          const A = <React.Fragment>{<div />}</React.Fragment>;
			          return <div>{A}</div>;
			        }
			      ",
            None,
        ),
        (
            r#"
			        function Component() {
			          let first = "";
			          const second = first;
			          first = second;
			          return <div id={first} />;
			        };
			      "#,
            None,
        ),
        (
            r#"
			        function Component() {
			          let first = "";
			          let second = "";
			          let third = "";
			          let fourth = "";
			          const fifth = first;
			          first = second;
			          second = third;
			          third = fourth;
			          fourth = fifth;
			          return <div id={first} />;
			        };
			      "#,
            None,
        ),
    ];

    let fail = vec![
        (
            "
			        <App>
			          <foo />
			        </App>
			      ",
            Some(serde_json::json!([{ "max": 0 }])),
        ),
        (
            "
			        <App>
			          <foo>{bar}</foo>
			        </App>
			      ",
            Some(serde_json::json!([{ "max": 0 }])),
        ),
        (
            "
			        <App>
			          <foo>
			            <bar />
			          </foo>
			        </App>
			      ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
			        const x = <div><span /></div>;
			        <div>{x}</div>
			      ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
			        const x = <div><span /></div>;
			        let y = x;
			        <div>{y}</div>
			      ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
			        const x = <div><span /></div>;
			        let y = x;
			        <div>{x}-{y}</div>
			      ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
			        <div>
			        {<div><div><span /></div></div>}
			        </div>
			      ",
            None,
        ),
        (
            "
			        <>
			          <foo />
			        </>
			      ",
            Some(serde_json::json!([{ "max": 0 }])),
        ),
        (
            "
			        <>
			          <>
			            <bar />
			          </>
			        </>
			      ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
			        const x = <><span /></>;
			        let y = x;
			        <>{x}-{y}</>
			      ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
			        const x = (
			          <tr>
			            <td>1</td>
			            <td>2</td>
			          </tr>
			        );
			        <tbody>
			          {x}
			        </tbody>
			      ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            r#"
			        <div className="custom_modal">
			          <Modal className={classes.modal} open={isOpen} closeAfterTransition>
			            <Fade in={isOpen}>
			              <DialogContent>
			                <Icon icon="cancel" onClick={onClose} popoverText="Close Modal" />
			                <div className="modal_content">{children}</div>
			                <div className={clxs('modal_buttons', classes.buttons)}>
			                  <Button className="modal_buttons--cancel" onClick={onCancel}>
			                    {cancelMsg ? cancelMsg : 'Cancel'}
			                  </Button>
			                </div>
			              </DialogContent>
			            </Fade>
			          </Modal>
			        </div>
			      "#,
            Some(serde_json::json!([{ "max": 4 }])),
        ),
    ];

    Tester::new(JsxMaxDepth::NAME, JsxMaxDepth::PLUGIN, pass, fail).test_and_snapshot();
}
