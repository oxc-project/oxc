use oxc_ast::{AstKind, ast::Argument};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{PossibleJestNode, parse_general_jest_fn_call},
};

fn prefer_import_in_mock_diagnostic(span: Span, path: &str) -> OxcDiagnostic {
    let help = format!(
        "Dynamic import improves the type information and IntelliSense. Substitute `{path}` with `import({path})`"
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
#[serde(deny_unknown_fields)]
pub struct PreferImportInMockConfig {
    /// Whether the rule should generate fixes or not.
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
    /// This rule enforces using a dynamic `import()` in `vi.mock()` or `vi.doMock()`, which improves type information and IntelliSense for the mocked module.
    ///
    /// ### Why is this bad?
    ///
    /// A lack of type information and IntelliSense increases the risk of mismatches between the real module and its mock.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// vi.mock('./path/to/module')
    /// vi.doMock('./path/to/module')
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// vi.mock(import('./path/to/module'))
    /// vi.doMock(import('./path/to/module'))
    /// ```
    PreferImportInMock,
    vitest,
    style,
    fix,
    config = PreferImportInMockConfig,
    version = "1.49.0",
    short_description = "This rule enforces using a dynamic `import()` in `vi.mock()` or `vi.doMock()`, which improves type information and IntelliSense for the mocked module.",
);

impl Rule for PreferImportInMock {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(Self(Box::new(
            DefaultRuleConfig::<PreferImportInMockConfig>::from_value(value)
                .unwrap_or_default()
                .into_inner(),
        )))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(_) = node.kind() else { return };
        self.run(node, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
    }
}

const MOCK_METHODS: [&str; 2] = ["mock", "doMock"];

impl PreferImportInMock {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(callee_name) = call_expr.callee_name() else {
            return;
        };

        if !MOCK_METHODS.contains(&callee_name) {
            return;
        }

        let Some(possible_jest_node) = PossibleJestNode::from_call_expression(node, call_expr, ctx)
        else {
            return;
        };
        if parse_general_jest_fn_call(call_expr, &possible_jest_node, ctx).is_none() {
            return;
        }

        let Some(Argument::StringLiteral(import_value)) = call_expr.arguments.first() else {
            return;
        };

        // Use the raw source text so the fix preserves the original quote style
        // and escapes; re-serializing the cooked value can change the module
        // specifier or produce invalid syntax.
        let raw = import_value
            .raw
            .map_or_else(|| ctx.source_range(import_value.span), |raw| raw.as_str());

        ctx.diagnostic_with_fix(
            prefer_import_in_mock_diagnostic(call_expr.arguments_span().unwrap(), raw),
            |fixer| {
                if !self.fixable {
                    return fixer.noop();
                }

                fixer.replace(import_value.span, format!("import({raw})"))
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
        (r#"vi.doMock(import("foo"))"#, None),
        (r#"vi.doMock(import("node:fs/promises"))"#, None),
        (r#"vi.doMock(import("./foo.js"), () => ({ Foo: vi.fn() }))"#, None),
        (r#"vi.mock(import("foo"))"#, None),
        (r#"vi.mock(import("node:fs/promises"))"#, None),
        (r#"vi.mock(import("./foo.js"), () => ({ Foo: vi.fn() }))"#, None),
        (r#"vi.mock(import("./foo.js"), { spy: true });"#, None),
    ];

    let fail = vec![
        // Lone surrogates are part of the value.
        // A search or prefix check must still see the rest.
        (r#"vi.mock("\uD800")"#, None),
        (r#"vi.doMock("a\uDC00b", () => ({}))"#, None),
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
        ("vi.doMock('foo', () => {})", Some(serde_json::json!([ { "fixable": false, }, ]))),
        (r#"vi.doMock("node:fs/promises")"#, Some(serde_json::json!([ { "fixable": false, }, ]))),
        (
            r#"vi.doMock("./foo.js", () => ({ Foo: vi.fn() }))"#,
            Some(serde_json::json!([ { "fixable": false, }, ])),
        ),
        (
            "
                    import { vi as renamedVi } from 'vitest';
                    renamedVi.doMock('./foo.js', () => ({ Foo: vi.fn() }))
                  ",
            Some(serde_json::json!([ { "fixable": false, }, ])),
        ),
        ("vi.mock('foo', () => {})", None),
        (r#"vi.mock("node:fs/promises")"#, None),
        (r#"vi.mock("./it's.js", () => ({ x: 1 }))"#, None),
        (r"vi.mock('./dir\\file.js', () => ({ x: 2 }))", None),
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
        // Paths with a lone surrogate are preserved as-is.
        (r#"vi.mock("\uD800")"#, r#"vi.mock(import("\uD800"))"#),
        (r#"vi.doMock("a\uDC00b", () => ({}))"#, r#"vi.doMock(import("a\uDC00b"), () => ({}))"#),
        ("vi.mock('foo', () => {})", "vi.mock(import('foo'), () => {})"),
        (
            r#"vi.mock("./it's.js", () => ({ x: 1 }))"#,
            r#"vi.mock(import("./it's.js"), () => ({ x: 1 }))"#,
        ),
        (
            r"vi.mock('./dir\\file.js', () => ({ x: 2 }))",
            r"vi.mock(import('./dir\\file.js'), () => ({ x: 2 }))",
        ),
        (r#"vi.mock("node:fs/promises")"#, r#"vi.mock(import("node:fs/promises"))"#),
        (
            r#"vi.mock("./foo.js", () => ({ Foo: vi.fn() }))"#,
            r#"vi.mock(import("./foo.js"), () => ({ Foo: vi.fn() }))"#,
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
        ("vi.doMock('foo', () => {})", "vi.doMock(import('foo'), () => {})"),
        (r#"vi.doMock("node:fs/promises")"#, r#"vi.doMock(import("node:fs/promises"))"#),
        (
            r#"vi.doMock("./foo.js", () => ({ Foo: vi.fn() }))"#,
            r#"vi.doMock(import("./foo.js"), () => ({ Foo: vi.fn() }))"#,
        ),
        (
            "
                    import { vi as renamedVi } from 'vitest';
                    renamedVi.doMock('./foo.js', () => ({ Foo: vi.fn() }))
                  ",
            "
                    import { vi as renamedVi } from 'vitest';
                    renamedVi.doMock(import('./foo.js'), () => ({ Foo: vi.fn() }))
                  ",
        ),
    ];

    Tester::new(PreferImportInMock::NAME, PreferImportInMock::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
