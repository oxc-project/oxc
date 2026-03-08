use oxc_ast::{AstKind, ast::Argument};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{PossibleJestNode, parse_general_jest_fn_call},
};

fn prefer_import_in_mock_diagnostic(span: Span, path: &str) -> OxcDiagnostic {
    let help = format!(
        "Dynamic import improves the type information and IntelliSense. Substitute `{path}` with `import('{path}')`"
    );

    OxcDiagnostic::warn("Mocked modules must be dynamic imported.").with_help(help).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferImportInMock(Box<PreferImportInMockConfig>);

impl std::ops::Deref for PreferImportInMock {
    type Target = PreferImportInMockConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct PreferImportInMockConfig {
    fixable: bool,
}

impl Default for PreferImportInMockConfig {
    fn default() -> Self {
        Self { fixable: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces using a dynamic import() in `vi.mock()`, which improves type information and IntelliSense for the mocked module.
    ///
    /// ### Why is this bad?
    ///
    /// A lack of type information and IntelliSense increase the risk of mismatches between the real module and it's mock.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// vi.mock('./path/to/module')
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// vi.mock(import('./path/to/module'))
    /// ```
    PreferImportInMock,
    vitest,
    style,
    fix,
    config = PreferImportInMockConfig
);

impl Rule for PreferImportInMock {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<PreferImportInMockConfig>>(value)
                .unwrap_or_default()
                .into_inner(),
        )))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        self.run(jest_node, ctx);
    }
}

impl PreferImportInMock {
    fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.callee_name() != Some("mock") {
            return;
        }

        if parse_general_jest_fn_call(call_expr, possible_jest_node, ctx).is_none() {
            return;
        }

        let Some(Argument::StringLiteral(import_value)) = call_expr.arguments.first() else {
            return;
        };

        ctx.diagnostic_with_fix(
            prefer_import_in_mock_diagnostic(
                call_expr.arguments_span().unwrap(),
                import_value.value.as_ref(),
            ),
            |fixer| {
                if !self.fixable {
                    return fixer.noop();
                }

                fixer.replace(
                    import_value.span,
                    format!("import('{}')", import_value.value.as_ref()),
                )
            },
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"vi.mock(import("foo"))"#, None),
        (r#"vi.mock(import("node:fs/promises"))"#, None),
        (r#"vi.mock(import("./foo.js"), () => ({ Foo: vi.fn() }))"#, None),
        (r#"vi.mock(import("./foo.js"), { spy: true });"#, None),
        (
            "
                    describe.each(['webpack', 'turbopack'])('DevAppRouteRouteMatcher %s', (bundler) => {})
                    it.each([1])(\"matches the '$route.page' route specified with the provided files\", () => {})
                  ",
            None,
        ),
        (r#"vi.mock(import("foo"))"#, None),
        (r#"vi.mock(import("node:fs/promises"))"#, None),
        (r#"vi.mock(import("./foo.js"), () => ({ Foo: vi.fn() }))"#, None),
        (r#"vi.mock(import("./foo.js"), { spy: true });"#, None),
    ];

    let fail = vec![
        ("vi.mock('foo', () => {})", Some(serde_json::json!([ { "fixable": false, }, ]))),
        (r#"vi.mock("node:fs/promises")"#, Some(serde_json::json!([ { "fixable": false, }, ]))),
        (
            r#"vi.mock("./foo.js", () => ({ Foo: vi.fn() }))"#,
            Some(serde_json::json!([ { "fixable": false, }, ])),
        ),
        (
            "
                    import { vi as renamedVi } from 'vitest';
                    renamedVi.mock('./foo.js', () => ({ Foo: vi.fn() }))
                  ",
            Some(serde_json::json!([ { "fixable": false, }, ])),
        ),
        ("vi.mock('foo', () => {})", None),
        (r#"vi.mock("node:fs/promises")"#, None),
        (r#"vi.mock("./foo.js", () => ({ Foo: vi.fn() }))"#, None),
        (
            "
                    import { vi as renamedVi } from 'vitest';
                    renamedVi.mock('./foo.js', () => ({ Foo: vi.fn() }))
                  ",
            None,
        ),
    ];

    let fix = vec![
        ("vi.mock('foo', () => {})", "vi.mock(import('foo'), () => {})"),
        (r#"vi.mock("node:fs/promises")"#, "vi.mock(import('node:fs/promises'))"),
        (
            r#"vi.mock("./foo.js", () => ({ Foo: vi.fn() }))"#,
            "vi.mock(import('./foo.js'), () => ({ Foo: vi.fn() }))",
        ),
        (
            "
                    import { vi as renamedVi } from 'vitest';
                    renamedVi.mock('./foo.js', () => ({ Foo: vi.fn() }))
                  ",
            "
                    import { vi as renamedVi } from 'vitest';
                    renamedVi.mock(import('./foo.js'), () => ({ Foo: vi.fn() }))
                  ",
        ),
    ];

    Tester::new(PreferImportInMock::NAME, PreferImportInMock::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
