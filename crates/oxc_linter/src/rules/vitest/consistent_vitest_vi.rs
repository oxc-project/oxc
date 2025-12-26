use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, parse_general_jest_fn_call},
};

fn consistent_vitest_vi_diagnostic(span: Span, fn_value: &VitestFnName) -> OxcDiagnostic {
    let message = format!(
        "Prefer using `{}` instead of `{}`.",
        fn_value.get_string(),
        fn_value.get_opposite_accessor()
    );

    OxcDiagnostic::warn("The vitest function accessor used is not allowed")
        .with_help(message)
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentVitestVi(Box<ConsistentVitestConfig>);

impl std::ops::Deref for ConsistentVitestVi {
    type Target = ConsistentVitestConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, JsonSchema, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VitestFnName {
    #[default]
    Vi,
    Vitest,
}

impl VitestFnName {
    fn get_opposite_accessor(&self) -> CompactStr {
        match self {
            VitestFnName::Vi => CompactStr::new("vitest"),
            VitestFnName::Vitest => CompactStr::new("vi"),
        }
    }

    fn get_string(&self) -> CompactStr {
        match self {
            VitestFnName::Vi => CompactStr::new("vi"),
            VitestFnName::Vitest => CompactStr::new("vitest"),
        }
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct ConsistentVitestConfig {
    /// Decides whether to prefer vitest function accessor
    #[serde(rename = "fn", default)]
    function: VitestFnName,
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule triggers an error when a not expected vitest accessor is used.
    ///
    /// ### Why is this bad?
    ///
    /// Not having a consistent vitest accessor can lead to confusion on why
    /// on some contexts `vi` is used, and on other `vitest` is used.
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
    correctness,
    fix,
    config = ConsistentVitestConfig,
);

impl Rule for ConsistentVitestVi {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config: ConsistentVitestConfig =
            value.get(0).and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();

        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import) => {
                if import.source.value != "vitest" {
                    return;
                }

                let Some(vitest_import) = import.specifiers.as_ref().and_then(|specs| {
                    specs.iter().find(|spec| spec.name() == self.function.get_opposite_accessor())
                }) else {
                    return;
                };

                ctx.diagnostic_with_fix(
                    consistent_vitest_vi_diagnostic(vitest_import.span(), &self.function),
                    |fixer| {
                        let mut specifiers_without_opposite_accessor = import
                            .specifiers
                            .as_ref()
                            .map(|specs| {
                                specs
                                    .iter()
                                    .map(|spec| CompactStr::from(spec.name()))
                                    .filter(|spec_name| {
                                        *spec_name != self.function.get_opposite_accessor()
                                    })
                                    .collect::<Vec<CompactStr>>()
                            })
                            .unwrap_or(vec![]);

                        if specifiers_without_opposite_accessor.is_empty() {
                            match self.function {
                                VitestFnName::Vi => fixer.replace(vitest_import.local().span, "vi"),
                                VitestFnName::Vitest => {
                                    fixer.replace(vitest_import.local().span, "vitest")
                                }
                            }
                        } else {
                            if !specifiers_without_opposite_accessor
                                .contains(&self.function.get_string())
                            {
                                specifiers_without_opposite_accessor
                                    .push(self.function.get_string());
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

                if vitest_fn.name == self.function.get_opposite_accessor() {
                    let Some(member_expression) = call_expr.callee.as_member_expression() else {
                        return;
                    };

                    ctx.diagnostic_with_fix(
                        consistent_vitest_vi_diagnostic(
                            member_expression.object().span(),
                            &self.function,
                        ),
                        |fixer| {
                            fixer.replace(
                                member_expression.object().span(),
                                self.function.get_string(),
                            )
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
