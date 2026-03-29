use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use oxc_ast::{AstKind, ast::ImportDeclarationSpecifier};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, parse_general_jest_fn_call},
};

fn consistent_vitest_vi_diagnostic(span: Span, fn_value: &VitestFnName) -> OxcDiagnostic {
    OxcDiagnostic::warn("The vitest function accessor used is not allowed")
        .with_help(format!(
            "Prefer using `{}` instead of `{}`.",
            fn_value.as_str(),
            fn_value.not().as_str()
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConsistentVitestVi(Box<ConsistentVitestConfig>);

impl std::ops::Deref for ConsistentVitestVi {
    type Target = ConsistentVitestConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VitestFnName {
    #[default]
    Vi,
    Vitest,
}

impl VitestFnName {
    fn not(&self) -> Self {
        match self {
            VitestFnName::Vi => VitestFnName::Vitest,
            VitestFnName::Vitest => VitestFnName::Vi,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            VitestFnName::Vi => "vi",
            VitestFnName::Vitest => "vitest",
        }
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub struct ConsistentVitestConfig {
    /// Decides whether to prefer vitest function accessor
    #[serde(rename = "fn", default)]
    function: VitestFnName,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule triggers an error when an unexpected vitest accessor is used.
    ///
    /// ### Why is this bad?
    ///
    /// Not having a consistent vitest accessor can lead to confusion
    /// when `vi` and `vitest` are used interchangeably.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// vitest.mock('./src/calculator.ts', { spy: true })
    ///
    /// vi.stubEnv('NODE_ENV', 'production')
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// vi.mock('./src/calculator.ts', { spy: true })
    ///
    /// vi.stubEnv('NODE_ENV', 'production')
    /// ```
    ConsistentVitestVi,
    vitest,
    style,
    fix,
    config = ConsistentVitestConfig,
);

impl Rule for ConsistentVitestVi {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import) => {
                if import.source.value != "vitest" {
                    return;
                }

                let opposite = self.function.not();
                let Some(vitest_import) = import
                    .specifiers
                    .as_ref()
                    .and_then(|specs| specs.iter().find(|spec| spec.name() == opposite.as_str()))
                else {
                    return;
                };

                ctx.diagnostic_with_fix(
                    consistent_vitest_vi_diagnostic(vitest_import.span(), &self.function),
                    |fixer| {
                        let mut specifiers_without_opposite_accessor: Vec<Cow<str>> = import
                            .specifiers
                            .as_ref()
                            .map(|specs| {
                                specs
                                    .iter()
                                    .filter(|spec| spec.name() != opposite.as_str())
                                    .map(ImportDeclarationSpecifier::name)
                                    .collect()
                            })
                            .unwrap_or_default();

                        if specifiers_without_opposite_accessor.is_empty() {
                            fixer.replace(vitest_import.local().span, self.function.as_str())
                        } else {
                            if !specifiers_without_opposite_accessor
                                .iter()
                                .any(|s| s.as_ref() == self.function.as_str())
                            {
                                specifiers_without_opposite_accessor
                                    .push(self.function.as_str().into());
                            }

                            let import_text = specifiers_without_opposite_accessor.join(", ");

                            let Some(first_specifier) =
                                import.specifiers.as_ref().and_then(|specs| specs.first())
                            else {
                                return fixer.noop();
                            };

                            let Some(last_specifier) =
                                import.specifiers.as_ref().and_then(|specs| specs.last())
                            else {
                                return fixer.noop();
                            };

                            let specifiers_span = Span::new(
                                first_specifier.local().span.start,
                                last_specifier.local().span.end,
                            );

                            fixer.replace(specifiers_span, import_text)
                        }
                    },
                );
            }
            AstKind::CallExpression(call_expr) => {
                let Some(vitest_fn) = parse_general_jest_fn_call(
                    call_expr,
                    &PossibleJestNode { node, original: None },
                    ctx,
                ) else {
                    return;
                };

                if vitest_fn.kind != JestFnKind::General(JestGeneralFnKind::Vitest) {
                    return;
                }

                if vitest_fn.name == self.function.not().as_str() {
                    let Some(member_expression) = call_expr.callee.as_member_expression() else {
                        return;
                    };

                    ctx.diagnostic_with_fix(
                        consistent_vitest_vi_diagnostic(
                            member_expression.object().span(),
                            &self.function,
                        ),
                        |fixer| {
                            fixer.replace(member_expression.object().span(), self.function.as_str())
                        },
                    );
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"import { expect, it } from "vitest";"#, None),
        (r#"import { vi } from "vitest";"#, None),
        (r#"import { vitest } from "vitest";"#, Some(serde_json::json!([{ "fn": "vitest" }]))),
        (
            r#"import { vi } from "vitest";
			vi.stubEnv("NODE_ENV", "production");"#,
            None,
        ),
        (r#"vi.stubEnv("NODE_ENV", "production");"#, None),
    ];

    let fail = vec![
        (r#"import { vitest } from "vitest";"#, None),
        (r#"import { expect, vi, vitest } from "vitest";"#, None),
        (
            r#"import { vitest } from "vitest";
			vitest.stubEnv("NODE_ENV", "production");"#,
            None,
        ),
        (
            r#"vi.stubEnv("NODE_ENV", "production");
			vi.clearAllMocks();"#,
            Some(serde_json::json!([{ "fn": "vitest" }])),
        ),
    ];

    let fix = vec![
        (r#"import { vitest } from "vitest";"#, r#"import { vi } from "vitest";"#, None), // WORKING
        (
            r#"import { expect, vi, vitest } from "vitest";"#,
            r#"import { expect, vi } from "vitest";"#,
            None,
        ),
        (
            r#"import { vitest } from "vitest";
			vitest.stubEnv("NODE_ENV", "production");"#,
            r#"import { vi } from "vitest";
			vi.stubEnv("NODE_ENV", "production");"#,
            None,
        ),
        (
            r#"vi.stubEnv("NODE_ENV", "production");
			vi.clearAllMocks();"#,
            r#"vitest.stubEnv("NODE_ENV", "production");
			vitest.clearAllMocks();"#,
            Some(serde_json::json!([{ "fn": "vitest" }])),
        ),
    ];
    Tester::new(ConsistentVitestVi::NAME, ConsistentVitestVi::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
