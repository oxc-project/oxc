use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(no-hooks): Disallow setup and teardown hooks.")]
#[diagnostic(severity(warning))]
pub struct UnexpectedHookDiagonsitc(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoHooks {
    pub allow: Vec<String>,
}

declare_oxc_lint!(
    /// ### What it does
    /// Jest provides global functions for setup and teardown tasks, which are called before/after each test case
    /// and each test suite. The use of these hooks promotes shared state between tests.
    ///
    /// ### Why is this bad?
    ///
    /// This rule reports for the following function calls:
    /// * beforeAll
    /// * beforeEach
    /// * afterAll
    /// * afterEach
    ///
    /// ### Example
    ///
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
    style,
);

impl Rule for NoHooks {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allow = value
            .get(0)
            .and_then(|config| config.get("allow"))
            .and_then(serde_json::Value::as_array)
            .map(|v| {
                v.iter().filter_map(serde_json::Value::as_str).map(ToString::to_string).collect()
            })
            .unwrap_or_default();

        Self { allow }
    }

    fn run_once(&self, ctx: &LintContext) {
        for possible_jest_node in collect_possible_jest_call_node(ctx) {
            self.run(&possible_jest_node, ctx);
        }
    }
}

impl NoHooks {
    fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
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
            if !self.allow.contains(&ident.name.to_string()) {
                ctx.diagnostic(UnexpectedHookDiagonsitc(call_expr.callee.span()));
            }
        }
    }
}

#[test]
fn test() {
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

    Tester::new(NoHooks::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
