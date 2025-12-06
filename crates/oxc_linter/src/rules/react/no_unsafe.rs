use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{get_parent_component, is_es5_component},
};

fn no_unsafe_diagnostic(method_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unsafe lifecycle method `{method_name}` is not allowed"))
        .with_help(format!(
            "`{method_name}` is deprecated and may be removed in future React versions. Consider using alternative lifecycle methods or hooks."
        ))
        .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
#[serde(rename_all = "camelCase", default)]
struct NoUnsafeConfig {
    #[serde(default)]
    check_aliases: bool,
}

impl Default for NoUnsafeConfig {
    fn default() -> Self {
        Self { check_aliases: false }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NoUnsafe(NoUnsafeConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule identifies and restricts the use of unsafe React lifecycle methods.
    ///
    /// ### Why is this bad?
    ///
    /// Certain lifecycle methods (`componentWillMount`, `componentWillReceiveProps`, and `componentWillUpdate`)
    /// are considered unsafe and have been deprecated since React 16.9. They are frequently misused and cause
    /// problems in async rendering. Using their `UNSAFE_` prefixed versions or the deprecated names themselves
    /// should be avoided.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// // By default, UNSAFE_ prefixed methods are flagged
    /// class Foo extends React.Component {
    ///   UNSAFE_componentWillMount() {}
    ///   UNSAFE_componentWillReceiveProps() {}
    ///   UNSAFE_componentWillUpdate() {}
    /// }
    ///
    /// // With checkAliases: true, non-prefixed versions are also flagged
    /// class Bar extends React.Component {
    ///   componentWillMount() {}
    ///   componentWillReceiveProps() {}
    ///   componentWillUpdate() {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// class Foo extends React.Component {
    ///   componentDidMount() {}
    ///   componentDidUpdate() {}
    ///   render() {}
    /// }
    /// ```
    NoUnsafe,
    react,
    correctness,
    config = NoUnsafeConfig,
);

impl Rule for NoUnsafe {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(
            serde_json::from_value::<DefaultRuleConfig<NoUnsafeConfig>>(value)
                .unwrap_or_default()
                .into_inner(),
        )
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let react_version =
            ctx.settings().react.version.as_ref().and_then(|v| parse_react_version(v.as_str()));

        if let AstKind::MethodDefinition(method_def) = node.kind() {
            let method_name = method_def.key.static_name();
            if let Some(name) = method_name {
                if is_unsafe_method(name.as_ref(), self.0.check_aliases, react_version) {
                    if get_parent_component(node, ctx).is_some() {
                        ctx.diagnostic(no_unsafe_diagnostic(name.as_ref(), method_def.key.span()));
                    }
                }
            }
        }

        if let AstKind::ObjectProperty(obj_prop) = node.kind() {
            if let Some(name) = obj_prop.key.static_name() {
                if is_unsafe_method(name.as_ref(), self.0.check_aliases, react_version) {
                    for ancestor in ctx.nodes().ancestors(node.id()) {
                        if is_es5_component(ancestor) {
                            ctx.diagnostic(no_unsafe_diagnostic(
                                name.as_ref(),
                                obj_prop.key.span(),
                            ));
                            break;
                        }
                    }
                }
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

/// Check if a method name is an unsafe lifecycle method
fn is_unsafe_method(
    name: &str,
    check_aliases: bool,
    react_version: Option<(u32, u32, u32)>,
) -> bool {
    // React 16.3 introduced the UNSAFE_ prefixed lifecycle methods
    let check_unsafe_prefix =
        react_version.map_or(true, |(major, minor, _)| major > 16 || (major == 16 && minor >= 3));

    if check_unsafe_prefix
        && matches!(
            name,
            "UNSAFE_componentWillMount"
                | "UNSAFE_componentWillReceiveProps"
                | "UNSAFE_componentWillUpdate"
        )
    {
        return true;
    }

    if check_aliases
        && matches!(
            name,
            "componentWillMount" | "componentWillReceiveProps" | "componentWillUpdate"
        )
    {
        return true;
    }

    false
}

/// Parse React version string into (major, minor, patch) tuple
fn parse_react_version(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts.get(2).and_then(|p| p.parse::<u32>().ok()).unwrap_or(0);

    Some((major, minor, patch))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			        class Foo extends React.Component {
			          componentDidUpdate() {}
			          render() {}
			        }
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.4.0" } } })),
        ),
        (
            "
			        const Foo = createReactClass({
			          componentDidUpdate: function() {},
			          render: function() {}
			        });
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.4.0" } } })),
        ),
        (
            "
			        class Foo extends Bar {
			          componentWillMount() {}
			          componentWillReceiveProps() {}
			          componentWillUpdate() {}
			        }
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.4.0" } } })),
        ),
        (
            "
			        class Foo extends Bar {
			          UNSAFE_componentWillMount() {}
			          UNSAFE_componentWillReceiveProps() {}
			          UNSAFE_componentWillUpdate() {}
			        }
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.4.0" } } })),
        ),
        (
            "
			        const Foo = bar({
			          componentWillMount: function() {},
			          componentWillReceiveProps: function() {},
			          componentWillUpdate: function() {},
			        });
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.4.0" } } })),
        ),
        (
            "
			        const Foo = bar({
			          UNSAFE_componentWillMount: function() {},
			          UNSAFE_componentWillReceiveProps: function() {},
			          UNSAFE_componentWillUpdate: function() {},
			        });
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.4.0" } } })),
        ),
        (
            "
			        class Foo extends React.Component {
			          componentWillMount() {}
			          componentWillReceiveProps() {}
			          componentWillUpdate() {}
			        }
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.4.0" } } })),
        ),
        (
            "
			        class Foo extends React.Component {
			          UNSAFE_componentWillMount() {}
			          UNSAFE_componentWillReceiveProps() {}
			          UNSAFE_componentWillUpdate() {}
			        }
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.2.0" } } })),
        ),
        (
            "
			        const Foo = createReactClass({
			          componentWillMount: function() {},
			          componentWillReceiveProps: function() {},
			          componentWillUpdate: function() {},
			        });
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.4.0" } } })),
        ),
        (
            "
			        const Foo = createReactClass({
			          UNSAFE_componentWillMount: function() {},
			          UNSAFE_componentWillReceiveProps: function() {},
			          UNSAFE_componentWillUpdate: function() {},
			        });
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.2.0" } } })),
        ),
    ];

    let fail = vec![
        (
            "
			        class Foo extends React.Component {
			          componentWillMount() {}
			          componentWillReceiveProps() {}
			          componentWillUpdate() {}
			        }
			      ",
            Some(serde_json::json!([{ "checkAliases": true }])),
            Some(serde_json::json!({ "settings": { "react": { "version": "16.4.0" } } })),
        ),
        (
            "
			        class Foo extends React.Component {
			          UNSAFE_componentWillMount() {}
			          UNSAFE_componentWillReceiveProps() {}
			          UNSAFE_componentWillUpdate() {}
			        }
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.3.0" } } })),
        ),
        (
            "
			        const Foo = createReactClass({
			          componentWillMount: function() {},
			          componentWillReceiveProps: function() {},
			          componentWillUpdate: function() {},
			        });
			      ",
            Some(serde_json::json!([{ "checkAliases": true }])),
            Some(serde_json::json!({ "settings": { "react": { "version": "16.3.0" } } })),
        ),
        (
            "
			        const Foo = createReactClass({
			          UNSAFE_componentWillMount: function() {},
			          UNSAFE_componentWillReceiveProps: function() {},
			          UNSAFE_componentWillUpdate: function() {},
			        });
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "version": "16.3.0" } } })),
        ),
    ];

    Tester::new(NoUnsafe::NAME, NoUnsafe::PLUGIN, pass, fail).test_and_snapshot();
}
