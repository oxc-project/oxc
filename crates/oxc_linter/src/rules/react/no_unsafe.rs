use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    config::ReactVersion,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{get_parent_component, is_es5_component},
};

fn no_unsafe_diagnostic(method_name: &str, span: Span) -> OxcDiagnostic {
    let replacement = match method_name {
        "componentWillMount" | "UNSAFE_componentWillMount" => "componentDidMount",
        "componentWillReceiveProps" | "UNSAFE_componentWillReceiveProps" => {
            "getDerivedStateFromProps"
        }
        "componentWillUpdate" | "UNSAFE_componentWillUpdate" => "componentDidUpdate",
        _ => "alternative lifecycle methods",
    };

    OxcDiagnostic::warn(format!("Unsafe lifecycle method `{method_name}` is not allowed"))
        .with_help(format!(
            "Use `{replacement}` instead. See https://legacy.reactjs.org/blog/2018/03/27/update-on-async-rendering.html"
        ))
        .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
struct NoUnsafeConfig {
    /// Whether to check for the non-prefixed lifecycle methods.
    /// If `true`, this means `componentWillMount`, `componentWillReceiveProps`,
    /// and `componentWillUpdate` will also be flagged, rather than just the
    /// UNSAFE_ versions. It is recommended to set this to `true` to fully
    /// avoid unsafe lifecycle methods.
    check_aliases: bool,
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
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::MethodDefinition(method_def) => {
                if let Some(name) = method_def.key.static_name()
                    && is_unsafe_method(name.as_ref(), self.0.check_aliases, ctx)
                    && get_parent_component(node, ctx).is_some()
                {
                    ctx.diagnostic(no_unsafe_diagnostic(name.as_ref(), method_def.key.span()));
                }
            }
            AstKind::ObjectProperty(obj_prop) => {
                if let Some(name) = obj_prop.key.static_name()
                    && is_unsafe_method(name.as_ref(), self.0.check_aliases, ctx)
                {
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
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

/// Check if a method name is an unsafe lifecycle method
fn is_unsafe_method(name: &str, check_aliases: bool, ctx: &LintContext) -> bool {
    let check_unsafe_prefix = ctx
        .settings()
        .react
        .version
        .as_ref()
        .is_none_or(ReactVersion::supports_unsafe_lifecycle_prefix);

    match name {
        "UNSAFE_componentWillMount"
        | "UNSAFE_componentWillReceiveProps"
        | "UNSAFE_componentWillUpdate"
            if check_unsafe_prefix =>
        {
            true
        }
        "componentWillMount" | "componentWillReceiveProps" | "componentWillUpdate"
            if check_aliases =>
        {
            true
        }
        _ => false,
    }
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
