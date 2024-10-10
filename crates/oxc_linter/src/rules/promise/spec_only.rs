use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule, utils::PROMISE_STATIC_METHODS, AstNode};

fn spec_only(prop_name: &str, member_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Avoid using non-standard `Promise.{prop_name}`"))
        .with_label(member_span)
}

#[derive(Debug, Default, Clone)]
pub struct SpecOnly(Box<SpecOnlyConfig>);

#[derive(Debug, Default, Clone)]
pub struct SpecOnlyConfig {
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
    restriction,
);

impl Rule for SpecOnly {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allowed_methods = value
            .get(0)
            .and_then(|v| v.get("allowedMethods"))
            .and_then(serde_json::Value::as_array)
            .map(|v| {
                v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect()
            });

        Self(Box::new(SpecOnlyConfig { allowed_methods }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MemberExpression(member_expr) = node.kind() else {
            return;
        };

        if !member_expr.object().is_specific_id("Promise") {
            return;
        }

        let Some(prop_name) = member_expr.static_property_name() else {
            return;
        };

        if PROMISE_STATIC_METHODS.contains(prop_name) {
            return;
        }

        if let Some(allowed_methods) = &self.allowed_methods {
            if allowed_methods.contains(prop_name) {
                return;
            }
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
        ("Promise.race()", None),
        ("Promise.withResolvers()", None),
        ("new Promise(function (resolve, reject) {})", None),
        ("SomeClass.resolve()", None),
        ("doSomething(Promise.all)", None),
        (
            "Promise.permittedMethod()",
            Some(serde_json::json!([ { "allowedMethods": ["permittedMethod"], }, ])),
        ),
    ];

    let fail = vec![
        ("Promise.done()", None),
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
    ];

    Tester::new(SpecOnly::NAME, pass, fail).test_and_snapshot();
}
