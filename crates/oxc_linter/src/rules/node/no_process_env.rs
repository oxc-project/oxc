use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{CompactStr, GetSpan, Span};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_process_env_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallowed usage of `process.env`.")
        .with_help("Remove usage of `process.env`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct NoProcessEnvConfig {
    /// Variable names which are allowed to be accessed on `process.env`.
    allowed_variables: FxHashSet<CompactStr>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct NoProcessEnv(Box<NoProcessEnvConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows use of `process.env`.
    ///
    /// ### Why is this bad?
    ///
    /// Directly reading `process.env` can lead to implicit runtime configuration,
    /// make code harder to test, and bypass configuration validation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if(process.env.NODE_ENV === "development") {
    ///   // ...
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import config from "./config";
    ///
    /// if(config.env === "development") {
    ///   //...
    /// }
    /// ```
    NoProcessEnv,
    node,
    restriction,
    config = NoProcessEnvConfig,
);

fn is_process_global_object(object_expr: &oxc_ast::ast::Expression, ctx: &LintContext) -> bool {
    let Some(obj_id) = object_expr.get_identifier_reference() else {
        return false;
    };
    obj_id.is_global_reference_name("process", ctx.scoping())
}

impl Rule for NoProcessEnv {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoProcessEnv>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Match `process.env` as either static `process.env` or computed `process["env"]`
        let span = match node.kind() {
            AstKind::StaticMemberExpression(mem)
                if mem.property.name.as_str() == "env"
                    && is_process_global_object(&mem.object, ctx) =>
            {
                mem.span
            }
            AstKind::ComputedMemberExpression(mem)
                if mem.static_property_name().is_some_and(|name| name.as_str() == "env")
                    && is_process_global_object(&mem.object, ctx) =>
            {
                mem.span
            }
            _ => return,
        };

        // Default: report any `process.env` usage
        let mut should_report = true;

        // If used as `process.env.ALLOWED` and `ALLOWED` is configured, do not report
        match ctx.nodes().parent_kind(node.id()) {
            AstKind::StaticMemberExpression(parent_mem) => {
                if let Some(obj_mem) = parent_mem.object.as_member_expression()
                    && obj_mem.span() == span
                {
                    let (.., prop_name) = parent_mem.static_property_info();
                    if self.0.allowed_variables.contains(prop_name) {
                        should_report = false;
                    }
                }
            }
            AstKind::ComputedMemberExpression(parent_mem) => {
                if let Some(obj_mem) = parent_mem.object.as_member_expression()
                    && obj_mem.span() == span
                    && let Some((_, name)) = parent_mem.static_property_info()
                    && self.0.allowed_variables.contains(name)
                {
                    should_report = false;
                }
            }
            _ => {}
        }

        if should_report {
            ctx.diagnostic(no_process_env_diagnostic(span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Process.env", None),
        ("process[env]", None),
        ("process.nextTick", None),
        ("process.execArgv", None),
        ("process.env.NODE_ENV", Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }]))),
        (
            "process.env['NODE_ENV']",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        (
            "process['env'].NODE_ENV",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        (
            "process['env']['NODE_ENV']",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
    ];

    let fail = vec![
        ("process.env", None),
        ("process['env']", None),
        ("process.env.ENV", None),
        ("f(process.env)", None),
        ("process.env.ENV", Some(serde_json::json!([{ "allowedVariables": [] }]))),
        ("f(process.env.NODE_ENV)", None),
        (
            "process.env['OTHER_VARIABLE']",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        (
            "process.env.OTHER_VARIABLE",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        (
            "process['env']['OTHER_VARIABLE']",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        (
            "process['env'].OTHER_VARIABLE",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        ("process.env[NODE_ENV]", Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }]))),
        (
            "process['env'][NODE_ENV]",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
    ];

    Tester::new(NoProcessEnv::NAME, NoProcessEnv::PLUGIN, pass, fail).test_and_snapshot();
}
