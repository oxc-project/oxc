use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{CallExpression, ChainElement, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    rules::PossibleJestNode,
    utils::{
        JestFnKind, JestGeneralFnKind, KnownMemberExpressionProperty, parse_general_jest_fn_call,
    },
};

fn require_mock_type_parameters_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing type parameters on mock function call")
        .with_help(format!(
            "Add a type parameter to the mock function, e.g. `vi.{method_name}<() => void>()`."
        ))
        .with_label(span)
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct RequireMockTypeParameters(Box<RequireMockTypeParametersConfig>);

#[derive(Debug, Default, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct RequireMockTypeParametersConfig {
    /// Also require type parameters for `importActual` and `importMock`.
    check_import_functions: bool,
}

impl std::ops::Deref for RequireMockTypeParameters {
    type Target = RequireMockTypeParametersConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of type parameters on `vi.fn()`, and optionally on `vi.importActual()` and `vi.importMock()`.
    ///
    /// By default, only `vi.fn()` is checked. Set `checkImportFunctions` to `true` to also check `vi.importActual()` and `vi.importMock()`.
    ///
    /// ### Why is this bad?
    ///
    /// Without explicit type parameters, `vi.fn()` creates a mock typed as `(...args: any[]) => any`.
    /// This disables type checking between the mock and the real implementation, which can lead to two problems:
    ///
    /// - tests that fail due to incorrect mock usage when they should pass
    /// - or worse, tests that pass while the mock silently diverges from the actual runtime behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule configured as `{ "checkImportFunctions": false }`:
    /// ```ts
    /// import { vi } from 'vitest'
    ///
    /// test('foo', () => {
    ///   const myMockedFn = vi.fn()
    /// })
    /// ```
    ///
    /// Examples of **incorrect** code for this rule configured as `{ "checkImportFunctions": true }`:
    /// ```ts
    /// import { vi } from 'vitest'
    ///
    /// vi.mock('./example.js', async () => {
    ///   const originalModule = await vi.importActual('./example.js')
    ///
    ///   return { ...originalModule }
    /// })
    /// const fs = await vi.importMock('fs')
    /// ```
    ///
    /// Examples of **correct** code for this rule configured as `{ "checkImportFunctions": false }`:
    /// ```ts
    /// import { vi } from 'vitest'
    ///
    /// test('foo', () => {
    ///   const myMockedFnOne = vi.fn<(arg1: string, arg2: boolean) => number>()
    ///   const myMockedFnTwo = vi.fn<() => void>()
    ///   const myMockedFnThree = vi.fn<any>()
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule configured as `{ "checkImportFunctions": true }`:
    /// ```ts
    /// import { vi } from 'vitest'
    ///
    /// vi.mock('./example.js', async () => {
    ///   const originalModule = await vi.importActual<any>('./example.js')
    ///
    ///   return { ...originalModule }
    /// })
    /// const fs = await vi.importMock<any>('fs')
    /// ```
    RequireMockTypeParameters,
    vitest,
    correctness,
    config = RequireMockTypeParametersConfig,
    version = "1.58.0",
);

impl Rule for RequireMockTypeParameters {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &crate::rules::PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        self.run_rule(jest_node, ctx);
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|extension| {
            let extension_str = extension.to_string_lossy();
            extension_str.ends_with("ts") || extension_str.ends_with("tsx")
        })
    }
}

impl RequireMockTypeParameters {
    fn run_rule<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(vi_fn) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) else {
            return;
        };

        if vi_fn.kind != JestFnKind::General(JestGeneralFnKind::Vitest) {
            return;
        }

        let Some(member) = vi_fn
            .members
            .iter()
            .find(|member| is_require_mock_type(member, self.check_import_functions))
        else {
            return;
        };

        if is_member_call_typed(call_expr, member) {
            return;
        }

        let method_name = if let Some(method) = member.name() {
            CompactStr::from(method)
        } else {
            CompactStr::new("fn")
        };

        ctx.diagnostic(require_mock_type_parameters_diagnostic(member.span, method_name.as_str()));
    }
}

const MOCK_REQUIRED_TYPES: [&str; 3] = ["fn", "importMock", "importActual"];

fn is_require_mock_type(
    member: &KnownMemberExpressionProperty<'_>,
    check_import_functions: bool,
) -> bool {
    if !check_import_functions {
        return member.is_name_equal("fn");
    }

    MOCK_REQUIRED_TYPES.iter().any(|&mock_function_name| member.is_name_equal(mock_function_name))
}

fn is_member_call_typed(
    call_expr: &CallExpression<'_>,
    member: &KnownMemberExpressionProperty<'_>,
) -> bool {
    find_member_call_type_arguments(call_expr, member).unwrap_or(true)
}

fn find_member_call_type_arguments(
    call_expr: &CallExpression<'_>,
    member: &KnownMemberExpressionProperty<'_>,
) -> Option<bool> {
    if let Some(member_expression) = call_expr.callee.get_member_expr() {
        if member_expression
            .static_property_info()
            .is_some_and(|(span, _)| span.start == member.span.start && span.end == member.span.end)
        {
            return Some(call_expr.type_arguments.is_some());
        }

        return find_member_call_type_arguments_in_expression(member_expression.object(), member);
    }

    find_member_call_type_arguments_in_expression(&call_expr.callee, member)
}

fn find_member_call_type_arguments_in_expression(
    expr: &Expression<'_>,
    member: &KnownMemberExpressionProperty<'_>,
) -> Option<bool> {
    match expr.get_inner_expression() {
        Expression::CallExpression(inner_call) => {
            find_member_call_type_arguments(inner_call, member)
        }
        Expression::ChainExpression(chain_expr) => match &chain_expr.expression {
            ChainElement::CallExpression(inner_call) => {
                find_member_call_type_arguments(inner_call, member)
            }
            chain_element => chain_element.member_expression().and_then(|member_expression| {
                find_member_call_type_arguments_in_expression(member_expression.object(), member)
            }),
        },
        expr => expr.get_member_expr().and_then(|member_expression| {
            find_member_call_type_arguments_in_expression(member_expression.object(), member)
        }),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        ("vi.fn<(...args: any[]) => any>()", None, None, Some(PathBuf::from("test.ts"))),
        ("vi.fn<(...args: string[]) => any>()", None, None, Some(PathBuf::from("test.ts"))),
        ("vi.fn<(arg1: string) => string>()", None, None, Some(PathBuf::from("test.ts"))),
        ("vi.fn<(arg1: any) => string>()", None, None, Some(PathBuf::from("test.ts"))),
        ("vi.fn<(arg1: string) => void>()", None, None, Some(PathBuf::from("test.ts"))),
        (
            "vi.fn<(arg1: string, arg2: boolean) => string>()",
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ),
        (
            "vi.fn<(arg1: string, arg2: boolean, ...args: string[]) => string>()",
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ),
        ("vi.fn<MyProcedure>()", None, None, Some(PathBuf::from("test.ts"))),
        ("vi.fn<any>()", None, None, Some(PathBuf::from("test.ts"))),
        ("vi.fn<(...args: any[]) => any>(() => {})", None, None, Some(PathBuf::from("test.ts"))),
        (
            r#"vi.fn<() => string | undefined>().mockReturnValue("some error message");"#,
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ),
        (
            r#"vi.importActual<{ default: boolean }>("./example.js")"#,
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ),
        (
            r#"vi.importActual<MyModule>("./example.js")"#,
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ),
        (r#"vi.importActual<any>("./example.js")"#, None, None, Some(PathBuf::from("test.ts"))),
        (
            r#"vi.importMock<{ default: boolean }>("./example.js")"#,
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ),
        (r#"vi.importMock<MyModule>("./example.js")"#, None, None, Some(PathBuf::from("test.ts"))),
        (r#"vi.importMock<any>("./example.js")"#, None, None, Some(PathBuf::from("test.ts"))),
        (r#"vi.importActual("./example.js")"#, None, None, Some(PathBuf::from("test.ts"))),
        (r#"vi.importMock("./example.js")"#, None, None, Some(PathBuf::from("test.spec.ts"))),
        //Ignoring js files to avoid false positives
        ("vi.fn()", None, None, Some(PathBuf::from("test.spec.js"))),
        ("vi.fn()", None, None, Some(PathBuf::from("test.jsx"))),
        (
            r#"vi.fn<() => Promise<string | null>>()
                .mockResolvedValueOnce("a")
                .mockResolvedValueOnce("b");"#,
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ),
        ("vi.fn<() => void>().fn()", None, None, Some(PathBuf::from("test.ts"))),
    ];

    let fail = vec![
        ("vi.fn()", None, None, Some(PathBuf::from("test.ts"))),
        ("vi.fn(() => {})", None, None, Some(PathBuf::from("test.tsx"))),
        (
            r#"vi.importActual("./example.js")"#,
            Some(serde_json::json!([{ "checkImportFunctions": true }])),
            None,
            Some(PathBuf::from("test.ts")),
        ),
        (
            r#"vi.importMock("./example.js")"#,
            Some(serde_json::json!([{ "checkImportFunctions": true }])),
            None,
            Some(PathBuf::from("test.ts")),
        ),
        (
            r#"vi.fn()
                .mockResolvedValueOnce("a")
                .mockResolvedValueOnce("b");"#,
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ),
        ("vi.fn().fn()", None, None, Some(PathBuf::from("test.ts"))),
    ];

    Tester::new(RequireMockTypeParameters::NAME, RequireMockTypeParameters::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
