use std::marker::PhantomData;

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    rules::TestFramework,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call},
};

fn unexpected_hook_diagonsitc(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use setup or teardown hooks").with_label(span)
}

#[derive(Debug, Clone)]
pub struct NoHooks<F: TestFramework>(PhantomData<F>, Box<NoHooksConfig>);

impl<F: TestFramework> Default for NoHooks<F> {
    fn default() -> Self {
        Self(PhantomData, Box::default())
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoHooksConfig {
    allow: Vec<CompactStr>,
}

impl<F: TestFramework> std::ops::Deref for NoHooks<F> {
    type Target = NoHooksConfig;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows Jest setup and teardown hooks, such as `beforeAll`.
    ///
    /// ### Why is this bad?
    ///
    /// Jest provides global functions for setup and teardown tasks, which are
    /// called before/after each test case and each test suite. The use of these
    /// hooks promotes shared state between tests.
    ///
    /// This rule reports for the following function calls:
    /// * `beforeAll`
    /// * `beforeEach`
    /// * `afterAll`
    /// * `afterEach`
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function setupFoo(options) { /* ... */ }
    /// function setupBar(options) { /* ... */ }
    ///
    /// describe('foo', () => {
    ///     let foo;
    ///     beforeEach(() => {
    ///         foo = setupFoo();
    ///     });
    ///     afterEach(() => {
    ///         foo = null;
    ///     });
    ///     it('does something', () => {
    ///         expect(foo.doesSomething()).toBe(true);
    ///     });
    ///     describe('with bar', () => {
    ///         let bar;
    ///         beforeEach(() => {
    ///             bar = setupBar();
    ///         });
    ///         afterEach(() => {
    ///             bar = null;
    ///         });
    ///         it('does something with bar', () => {
    ///             expect(foo.doesSomething(bar)).toBe(true);
    ///         });
    ///     });
    /// });
    /// ```
    NoHooks,
    test,
    style,
);

impl<F: TestFramework> Rule for NoHooks<F> {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allow = value
            .get(0)
            .and_then(|config| config.get("allow"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Self(PhantomData, Box::new(NoHooksConfig { allow }))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        if !is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Hook)],
        ) {
            return;
        }

        if let Expression::Identifier(ident) = &call_expr.callee {
            let name = CompactStr::from(ident.name.as_str());
            if !self.allow.contains(&name) {
                ctx.diagnostic(unexpected_hook_diagonsitc(call_expr.callee.span()));
            }
        }
    }
}

#[test]
fn test() {
    use crate::rules::{TestFrameworkJest, TestFrameworkVitest};
    use crate::tester::Tester;

    let pass = vec![
        ("test(\"foo\")", None),
        ("describe(\"foo\", () => { it(\"bar\") })", None),
        ("test(\"foo\", () => { expect(subject.beforeEach()).toBe(true) })", None),
        (
            "afterEach(() => {}); afterAll(() => {});",
            Some(serde_json::json!([{ "allow": ["afterEach", "afterAll"] }])),
        ),
        ("test(\"foo\")", Some(serde_json::json!([{ "allow": "undefined" }]))),
    ];

    let fail = vec![
        ("beforeAll(() => {})", None),
        ("beforeEach(() => {})", None),
        ("afterAll(() => {})", None),
        ("afterEach(() => {})", None),
        (
            "beforeEach(() => {}); afterEach(() => { jest.resetModules() });",
            Some(serde_json::json!([{ "allow": ["afterEach"] }])),
        ),
        (
            "
                import { beforeEach as afterEach, afterEach as beforeEach } from '@jest/globals';

                afterEach(() => {});
                beforeEach(() => { jest.resetModules() });
            ",
            Some(serde_json::json!([{ "allow": ["afterEach"] }])),
        ),
    ];

    let pass_vitest = vec![
        (r#"test("foo")"#, None),
        (r#"describe("foo", () => { it("bar") })"#, None),
        (r#"test("foo", () => { expect(subject.beforeEach()).toBe(true) })"#, None),
        (
            "afterEach(() => {}); afterAll(() => {});",
            Some(serde_json::json!([{ "allow": ["afterEach", "afterAll"] }])),
        ),
        (r#"test("foo")"#, Some(serde_json::json!([{ "allow": null }]))),
    ];

    let fail_vitest = vec![
        ("beforeAll(() => {})", None),
        ("beforeEach(() => {})", None),
        ("afterAll(() => {})", None),
        ("afterEach(() => {})", None),
        (
            "beforeEach(() => {}); afterEach(() => { vi.resetModules() });",
            Some(serde_json::json!([{ "allow": ["afterEach"] }])),
        ),
        (
            "
			    import { beforeEach as afterEach, afterEach as beforeEach, vi } from 'vitest';
			    afterEach(() => {});
			    beforeEach(() => { vi.resetModules() });
            ",
            Some(serde_json::json!([{ "allow": ["afterEach"] }])),
        ), // { "parserOptions": { "sourceType": "module" } }
    ];

    Tester::new(
        NoHooks::<TestFrameworkJest>::NAME,
        NoHooks::<TestFrameworkJest>::PLUGIN,
        pass,
        fail,
    )
    .with_jest_plugin(true)
    .test_and_snapshot();

    Tester::new(
        NoHooks::<TestFrameworkVitest>::NAME,
        NoHooks::<TestFrameworkVitest>::PLUGIN,
        pass_vitest,
        fail_vitest,
    )
    .with_vitest_plugin(true)
    .test_and_snapshot();
}
