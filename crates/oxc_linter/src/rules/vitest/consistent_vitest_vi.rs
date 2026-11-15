use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{Expression, ImportDeclarationSpecifier},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, is_vitest_import_source,
        parse_general_jest_fn_call,
    },
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

/// Returns `true` when every resolved reference to `symbol_id` is the object of
/// a member expression used as the callee of a call expression.
///
/// This is the only shape that can be safely renamed by the import/call fixes
/// without leaving other references unbound.
fn is_safe_to_rename_binding(
    symbol_id: SymbolId,
    target_name: &str,
    target_symbol_id: Option<SymbolId>,
    ctx: &LintContext<'_>,
) -> bool {
    ctx.semantic().symbol_references(symbol_id).all(|reference| {
        if ctx.scoping().find_binding(reference.scope_id(), target_name.into()) != target_symbol_id
        {
            return false;
        }

        let reference_node = ctx.semantic().nodes().get_node(reference.node_id());

        let AstKind::IdentifierReference(ident) = reference_node.kind() else {
            return false;
        };

        let parent = ctx.semantic().nodes().parent_node(reference_node.id());

        let (member_span, object_span) = match parent.kind() {
            AstKind::StaticMemberExpression(member) => (member.span, member.object.span()),
            AstKind::ComputedMemberExpression(member) => (member.span, member.object.span()),
            _ => return false,
        };

        if object_span != ident.span {
            return false;
        }

        let call_parent = ctx.semantic().nodes().parent_node(parent.id());

        let AstKind::CallExpression(call_expr) = call_parent.kind() else {
            return false;
        };

        call_expr.callee.span() == member_span
            && parse_general_jest_fn_call(
                call_expr,
                &PossibleJestNode { node: call_parent, original: None },
                ctx,
            )
            .is_some_and(|call| {
                call.kind == JestFnKind::General(JestGeneralFnKind::Vitest)
                    && call.name == ident.name.as_str()
            })
    })
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
    /// This rule triggers an error when an unexpected Vitest accessor is used.
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
    version = "1.37.0",
    short_description = "This rule triggers an error when an unexpected Vitest accessor is used.",
);

impl Rule for ConsistentVitestVi {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import) => {
                if !import.source.value.as_str().is_some_and(is_vitest_import_source) {
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

                let symbol_id = vitest_import.symbol_id();
                let target_name = self.function.as_str();
                let target_import = import
                    .specifiers
                    .as_ref()
                    .and_then(|specs| specs.iter().find(|spec| spec.name() == target_name));
                let target_symbol_id = target_import.and_then(|specifier| {
                    let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                        return None;
                    };
                    (import.import_kind.is_value()
                        && specifier.import_kind.is_value()
                        && specifier.imported.name() == target_name)
                        .then(|| specifier.local.symbol_id())
                });
                let conflicts_with_binding = target_import.is_none()
                    && ctx
                        .scoping()
                        .get_binding(ctx.scoping().symbol_scope_id(symbol_id), target_name.into())
                        .is_some();
                if !conflicts_with_binding
                    && is_safe_to_rename_binding(symbol_id, target_name, target_symbol_id, ctx)
                {
                    ctx.diagnostic_with_fix(
                        consistent_vitest_vi_diagnostic(vitest_import.span(), &self.function),
                        |fixer| {
                            let fixer = fixer.for_multifix();
                            let mut specifiers_without_opposite_accessor: Vec<&str> =
                                import.specifiers.as_ref().map_or_default(|specs| {
                                    specs
                                        .iter()
                                        .filter(|spec| spec.name() != opposite.as_str())
                                        .map(|spec| fixer.source_range(spec.span()))
                                        .collect()
                                });

                            let mut fix = if specifiers_without_opposite_accessor.is_empty() {
                                fixer.replace(vitest_import.local().span, self.function.as_str())
                            } else {
                                let target_already_bound =
                                    import.specifiers.as_ref().is_some_and(|specs| {
                                        specs.iter().any(|spec| {
                                            spec.name() != opposite.as_str()
                                                && spec.name() == self.function.as_str()
                                        })
                                    });
                                if !target_already_bound {
                                    specifiers_without_opposite_accessor
                                        .push(self.function.as_str());
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
                                    first_specifier.span().start,
                                    last_specifier.span().end,
                                );

                                fixer.replace(specifiers_span, import_text)
                            };
                            for reference in ctx.semantic().symbol_references(symbol_id) {
                                let reference_node = ctx.nodes().get_node(reference.node_id());
                                fix.push(
                                    fixer.replace(reference_node.span(), self.function.as_str()),
                                );
                            }
                            fix.with_message("Rename the Vitest import and its calls")
                        },
                    );
                } else {
                    ctx.diagnostic(consistent_vitest_vi_diagnostic(
                        vitest_import.span(),
                        &self.function,
                    ));
                }
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

                    let diagnostic = consistent_vitest_vi_diagnostic(
                        member_expression.object().span(),
                        &self.function,
                    );

                    if let Expression::Identifier(ident) = member_expression.object()
                        && ctx.scoping().get_reference(ident.reference_id()).symbol_id().is_none()
                        && ctx
                            .scoping()
                            .find_binding(
                                ctx.scoping().get_reference(ident.reference_id()).scope_id(),
                                self.function.as_str().into(),
                            )
                            .is_none()
                    {
                        ctx.diagnostic_with_fix(diagnostic, |fixer| {
                            fixer.replace(member_expression.object().span(), self.function.as_str())
                        });
                    } else {
                        ctx.diagnostic(diagnostic);
                    }
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
        (r#"import { vi } from "vite-plus/test";"#, None),
        (r#"import { vi } from "@effect/vitest";"#, None),
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
        (r#"import { vitest } from "vite-plus/test";"#, None),
        (r#"import { vitest } from "@effect/vitest";"#, None),
        (r#"import { expect, vi, vitest } from "vitest";"#, None),
        (
            r#"import { vitest } from "vitest";
			vitest.stubEnv("NODE_ENV", "production");"#,
            None,
        ),
        (
            r#"import { vitest } from "vite-plus/test";
			vitest.stubEnv("NODE_ENV", "production");"#,
            None,
        ),
        (
            r#"vi.stubEnv("NODE_ENV", "production");
			vi.clearAllMocks();"#,
            Some(serde_json::json!([{ "fn": "vitest" }])),
        ),
        (
            r#"import { vitest } from "vitest";
			const mocker = vitest;"#,
            None,
        ),
        (
            r#"import { vitest } from "vitest";
			const fn = vitest.fn;"#,
            None,
        ),
        (
            r#"import { vitest } from "vitest";
			vitest.fn();
			export { vitest };"#,
            None,
        ),
        (
            r#"import { vi } from "vitest";
			const mocker = vi;"#,
            Some(serde_json::json!([{ "fn": "vitest" }])),
        ),
    ];

    let mut fix = vec![
        (r#"import { vitest } from "vitest";"#, r#"import { vi } from "vitest";"#, None),
        (
            r#"import { vitest } from "vite-plus/test";"#,
            r#"import { vi } from "vite-plus/test";"#,
            None,
        ),
        (
            r#"import { vitest } from "@effect/vitest";"#,
            r#"import { vi } from "@effect/vitest";"#,
            None,
        ),
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
            r#"import { vitest } from "vite-plus/test";
			vitest.stubEnv("NODE_ENV", "production");"#,
            r#"import { vi } from "vite-plus/test";
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
        (
            r#"import { vitest, expect as toBe } from "vitest";"#,
            r#"import { expect as toBe, vi } from "vitest";"#,
            None,
        ),
        (
            r#"import { expect as e, vitest } from "vitest";"#,
            r#"import { expect as e, vi } from "vitest";"#,
            None,
        ),
        (
            r#"import { x as vi, vitest } from "vitest";"#,
            r#"import { x as vi } from "vitest";"#,
            None,
        ),
        (
            r#"import { x as vitest, vi } from "vitest";"#,
            r#"import { x as vitest } from "vitest";"#,
            Some(serde_json::json!([{ "fn": "vitest" }])),
        ),
    ];
    fix.push((
        r#"import { vitest } from "vitest"; vitest.fn(); vitest.mock("./dep");"#,
        r#"import { vi } from "vitest"; vi.fn(); vi.mock("./dep");"#,
        None,
    ));
    fix.push((
        r#"import { vi, vitest } from "vitest"; vitest.fn();"#,
        r#"import { vi } from "vitest"; vi.fn();"#,
        None,
    ));
    let no_fix = [
        (r#"import { vitest } from "vitest"; const mocker = vitest;"#, None),
        (r#"import { vitest } from "vitest"; const fn = vitest.fn;"#, None),
        (r#"import { vitest } from "vitest"; vitest.fn(); export { vitest };"#, None),
        (
            r#"import { vi } from "vitest"; const mocker = vi; vi.fn();"#,
            Some(serde_json::json!([{ "fn": "vitest" }])),
        ),
        (r#"import { vitest } from "vitest"; const mocker = vitest; vitest.fn();"#, None),
        (r#"import { vitest } from "vitest"; use(vitest); vitest.fn();"#, None),
        (r#"import { vitest } from "vitest"; const mock = vitest.fn().mock;"#, None),
        (r#"import { vitest } from "vitest"; vitest.fn()();"#, None),
        (
            r#"import { vitest } from "vitest"; const alias = vitest; vitest.fn().mockReturnValue(1);"#,
            None,
        ),
        ("vitest.fn().mockReturnValue(1);", None),
        (r#"import { vitest } from "vitest"; const vi = {}; vitest.fn();"#, None),
        (r#"import { vitest } from "vitest"; const vi = {};"#, None),
        (r#"import { vitest } from "vitest"; function run(vi) { vitest.fn(); }"#, None),
        (r#"import { vi, vitest } from "vitest"; function run(vi) { vitest.fn(); }"#, None),
        (r#"import { expect as vi, vitest } from "vitest"; vitest.fn();"#, None),
        (
            r#"import { vi } from "vitest"; function run(vitest) { vi.fn(); }"#,
            Some(serde_json::json!([{ "fn": "vitest" }])),
        ),
        ("function run(vi) { vitest.fn(); }", None),
        (
            r#"// oxlint-disable-next-line vitest/consistent-vitest-vi
import { vitest } from "vitest";
vitest.fn();"#,
            None,
        ),
    ];
    fix.extend(no_fix.into_iter().map(|(source, config)| (source, source, config)));
    Tester::new(ConsistentVitestVi::NAME, ConsistentVitestVi::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
