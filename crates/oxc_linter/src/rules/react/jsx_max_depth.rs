use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

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

#[derive(Debug, Clone, Default)]
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
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = serde_json::from_value::<DefaultRuleConfig<JsxMaxDepthConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .unwrap_or_default();

        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(_) | AstKind::JSXFragment(_) => {}
            _ => return,
        }

        let depth = jsx_ancestor_depth(node, ctx);
        if depth > self.max {
            ctx.diagnostic(jsx_max_depth_diagnostic(depth, self.max, node.span()));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
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
            r"
            const Component = () => (
              <div>
                <section />
              </div>
            );
            ",
            None,
        ),
        (
            r"
            const Component = () => (
              <div>
                <div>
                  <div />
                </div>
              </div>
            );
            ",
            Some(serde_json::json!([{ "max": 3 }])),
        ),
        (
            r"
            const Component = () => (
              <>
                <div>
                  <span />
                </div>
              </>
            );
            ",
            None,
        ),
    ];

    let fail = vec![
        (
            r"
            const Component = () => (
              <div>
                <div>
                  <div>
                    <span />
                  </div>
                </div>
              </div>
            );
            ",
            None,
        ),
        (
            r"
            const Component = () => (
              <>
                <div>
                  <>
                    <div />
                  </>
                </div>
              </>
            );
            ",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
    ];

    Tester::new(JsxMaxDepth::NAME, JsxMaxDepth::PLUGIN, pass, fail).test_and_snapshot();
}
