use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::PROMISE_STATIC_METHODS,
};

fn spec_only(prop_name: &str, member_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Avoid using non-standard `Promise.{prop_name}` method."))
        .with_label(member_span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct SpecOnly(Box<SpecOnlyConfig>);

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SpecOnlyConfig {
    /// List of Promise static methods that are allowed to be used.
    allowed_methods: Option<FxHashSet<CompactStr>>,
}

impl std::ops::Deref for SpecOnly {
    type Target = SpecOnlyConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow use of non-standard Promise static methods.
    ///
    /// ### Why is this bad?
    ///
    /// Non-standard Promises may cost more maintenance work.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Promise.done()
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// Promise.resolve()
    /// ```
    SpecOnly,
    promise,
    restriction,
    config = SpecOnlyConfig,
);

impl Rule for SpecOnly {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<SpecOnly>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(member_expr) = node.kind().as_member_expression_kind() else {
            return;
        };

        if !member_expr.object().is_specific_id("Promise") {
            return;
        }

        let Some(prop_name) = member_expr.static_property_name().map(|s| s.as_str()) else {
            return;
        };
        if PROMISE_STATIC_METHODS.contains(&prop_name) {
            return;
        }

        if let Some(allowed_methods) = &self.allowed_methods
            && allowed_methods.contains(prop_name)
        {
            return;
        }

        ctx.diagnostic(spec_only(prop_name, member_expr.span()));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Promise.resolve()", None),
        ("Promise.reject()", None),
        ("Promise.all()", None),
        ("Promise.all()", Some(serde_json::json!([{ "allowedMethods": [] }]))),
        ("Promise.race()", None),
        ("Promise.withResolvers()", None),
        ("new Promise(function (resolve, reject) {})", None),
        ("SomeClass.resolve()", None),
        ("doSomething(Promise.all)", None),
        (
            "Promise.permittedMethod()",
            Some(serde_json::json!([{ "allowedMethods": ["permittedMethod"] }])),
        ),
    ];

    let fail = vec![
        ("Promise.done()", None),
        ("Promise.done()", Some(serde_json::json!([{ "allowedMethods": [] }]))),
        ("Promise.something()", None),
        ("new Promise.done()", None),
        (
            "
            function foo() {
              var a = getA()
              return Promise.done(a)
            }
            ",
            None,
        ),
        (
            "
            function foo() {
              getA(Promise.done)
            }
            ",
            None,
        ),
        (
            "Promise.notPermittedMethod()",
            Some(serde_json::json!([{ "allowedMethods": ["permittedMethod"] }])),
        ),
        (
            // test case-sensitive match
            "Promise.differingCase()",
            Some(serde_json::json!([{ "allowedMethods": ["differingcase"] }])),
        ),
    ];

    Tester::new(SpecOnly::NAME, SpecOnly::PLUGIN, pass, fail).test_and_snapshot();
}
