use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn depend_ban_dependencies_diagnostic(span: Span, pkg: &str, reason: &str) -> OxcDiagnostic {
    let message = if reason.is_empty() {
        format!("Package \"{pkg}\" is banned")
    } else {
        format!("Package \"{pkg}\" is banned: {reason}")
    };
    OxcDiagnostic::warn(message)
        .with_help(format!("Remove the import of \"{pkg}\" and use the suggested alternative."))
        .with_label(span)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct BannedPackage {
    pub name: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct DependBanDependenciesConfig {
    banned: Vec<BannedPackage>,
}

impl Default for DependBanDependenciesConfig {
    fn default() -> Self {
        Self {
            banned: vec![
                BannedPackage {
                    name: "moment".to_string(),
                    reason: "Use date-fns or dayjs instead".to_string(),
                },
                BannedPackage {
                    name: "lodash".to_string(),
                    reason: "Use native Array/Object methods or lodash-es for tree-shaking"
                        .to_string(),
                },
                BannedPackage {
                    name: "underscore".to_string(),
                    reason: "Use native Array/Object methods instead".to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DependBanDependencies(Box<DependBanDependenciesConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Bans imports of specific packages that have better alternatives.
    ///
    /// ### Why is this bad?
    ///
    /// Some packages are outdated, bloated, or have better alternatives.
    /// Banning them encourages the use of modern, lighter, or more
    /// performant replacements.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (with default config):
    /// ```js
    /// import moment from "moment";
    /// const _ = require("lodash");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { format } from "date-fns";
    /// import { map } from "lodash-es";
    /// ```
    DependBanDependencies,
    oxc,
    restriction,
    none,
    config = DependBanDependenciesConfig
);

impl Rule for DependBanDependencies {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<DependBanDependenciesConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import) => {
                let source = import.source.value.as_str();
                self.check_source(source, import.source.span, ctx);
            }
            AstKind::CallExpression(call_expr) => {
                let Expression::Identifier(callee) = &call_expr.callee else {
                    return;
                };
                if callee.name != "require" {
                    return;
                }
                let Some(Expression::StringLiteral(arg)) =
                    call_expr.arguments.first().and_then(|a| a.as_expression())
                else {
                    return;
                };
                self.check_source(arg.value.as_str(), arg.span, ctx);
            }
            _ => {}
        }
    }
}

impl DependBanDependencies {
    fn check_source(&self, source: &str, span: Span, ctx: &LintContext<'_>) {
        for banned in &self.0.banned {
            if source == banned.name || source.starts_with(&format!("{}/", banned.name)) {
                ctx.diagnostic(depend_ban_dependencies_diagnostic(
                    span,
                    &banned.name,
                    &banned.reason,
                ));
                return;
            }
        }
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"import { format } from "date-fns";"#, None),
        (r#"import { map } from "lodash-es";"#, None),
        (r#"const x = require("safe-package");"#, None),
    ];

    let fail = vec![
        (r#"import moment from "moment";"#, None),
        (r#"const _ = require("lodash");"#, None),
        (r#"import _ from "underscore";"#, None),
        (r#"import merge from "lodash/merge";"#, None),
        (
            r#"import banned from "my-banned-pkg";"#,
            Some(
                json!([{ "banned": [{ "name": "my-banned-pkg", "reason": "Use something else" }] }]),
            ),
        ),
    ];

    Tester::new(DependBanDependencies::NAME, DependBanDependencies::PLUGIN, pass, fail)
        .test_and_snapshot();
}
