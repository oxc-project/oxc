use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        parse_general_jest_fn_call, JestFnKind, JestGeneralFnKind, ParsedGeneralJestFnCall,
        PossibleJestNode,
    },
};

#[derive(Debug, Default, Clone)]
pub struct NoDisabledTests;

declare_oxc_lint!(
    /// ### What it does
    /// This rule raises a warning about disabled tests.
    ///
    /// ### Why is this bad?
    ///
    /// Jest has a feature that allows you to temporarily mark tests as disabled. This
    /// feature is often helpful while debugging or to create placeholders for future
    /// tests. Before committing changes we may want to check that all tests are
    /// running.
    ///
    /// ### Example
    ///
    ///```js
    /// describe.skip('foo', () => {});
    /// it.skip('foo', () => {});
    /// test.skip('foo', () => {});
    ///
    /// describe['skip']('bar', () => {});
    /// it['skip']('bar', () => {});
    /// test['skip']('bar', () => {});
    ///
    /// xdescribe('foo', () => {});
    /// xit('foo', () => {});
    /// xtest('foo', () => {});
    ///
    /// it('bar');
    /// test('bar');
    ///
    /// it('foo', () => {
    ///   pending();
    /// });
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest/blob/main/docs/rules/no-disabled-tests.md),
    /// to use it, add the following configuration to your `.eslintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/no-disabled-tests": "error"
    ///   }
    /// }
    /// ```
    NoDisabledTests,
    correctness
);

fn no_disabled_tests_diagnostic(x1: &'static str, x2: &'static str, span3: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(x1).with_help(x2).with_label(span3)
}

enum Message {
    MissingFunction,
    Pending,
    DisabledSuiteWithSkip,
    DisabledSuiteWithX,
    DisabledTestWithSkip,
    DisabledTestWithX,
}

impl Message {
    pub fn details(&self) -> (&'static str, &'static str) {
        match self {
            Self::MissingFunction => ("Test is missing function argument", "Add function argument"),
            Self::Pending => ("Call to pending()", "Remove pending() call"),
            Self::DisabledSuiteWithSkip => ("Disabled test suite", "Remove the appending `.skip`"),
            Self::DisabledSuiteWithX => ("Disabled test suite", "Remove x prefix"),
            Self::DisabledTestWithSkip => ("Disabled test", "Remove the appending `.skip`"),
            Self::DisabledTestWithX => ("Disabled test", "Remove x prefix"),
        }
    }
}

impl Rule for NoDisabledTests {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    if let AstKind::CallExpression(call_expr) = node.kind() {
        if let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) {
            let ParsedGeneralJestFnCall { kind, members, name, .. } = jest_fn_call;
            // `test('foo')`
            let kind = match kind {
                JestFnKind::Expect | JestFnKind::ExpectTypeOf | JestFnKind::Unknown => return,
                JestFnKind::General(kind) => kind,
            };
            if matches!(kind, JestGeneralFnKind::Test)
                && call_expr.arguments.len() < 2
                && members.iter().all(|member| member.is_name_unequal("todo"))
            {
                let (error, help) = Message::MissingFunction.details();
                ctx.diagnostic(no_disabled_tests_diagnostic(error, help, call_expr.span));
                return;
            }

            // the only jest functions that are with "x" are "xdescribe", "xtest", and "xit"
            // `xdescribe('foo', () => {})`
            if name.starts_with('x') {
                let (error, help) = if matches!(kind, JestGeneralFnKind::Describe) {
                    Message::DisabledSuiteWithX.details()
                } else {
                    Message::DisabledTestWithX.details()
                };
                ctx.diagnostic(no_disabled_tests_diagnostic(error, help, call_expr.callee.span()));
                return;
            }

            // `it.skip('foo', function () {})'`
            // `describe.skip('foo', function () {})'`
            if members.iter().any(|member| member.is_name_equal("skip")) {
                let (error, help) = if matches!(kind, JestGeneralFnKind::Describe) {
                    Message::DisabledSuiteWithSkip.details()
                } else {
                    Message::DisabledTestWithSkip.details()
                };
                ctx.diagnostic(no_disabled_tests_diagnostic(error, help, call_expr.callee.span()));
            }
        } else if let Expression::Identifier(ident) = &call_expr.callee {
            if ident.name.as_str() == "pending"
                && ctx.semantic().is_reference_to_global_variable(ident)
            {
                // `describe('foo', function () { pending() })`
                let (error, help) = Message::Pending.details();
                ctx.diagnostic(no_disabled_tests_diagnostic(error, help, call_expr.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("describe('foo', function () {})", None),
        ("it('foo', function () {})", None),
        ("describe.only('foo', function () {})", None),
        ("it.only('foo', function () {})", None),
        ("it.each('foo', () => {})", None),
        ("it.concurrent('foo', function () {})", None),
        ("test('foo', function () {})", None),
        ("test.only('foo', function () {})", None),
        ("test.concurrent('foo', function () {})", None),
        ("describe[`${'skip'}`]('foo', function () {})", None),
        ("it.todo('fill this later')", None),
        ("var appliedSkip = describe.skip; appliedSkip.apply(describe)", None),
        ("var calledSkip = it.skip; calledSkip.call(it)", None),
        ("({ f: function () {} }).f()", None),
        ("(a || b).f()", None),
        ("itHappensToStartWithIt()", None),
        ("testSomething()", None),
        ("xitSomethingElse()", None),
        ("xitiViewMap()", None),
        (
            "import { pending } from 'actions'; test('foo', () => { expect(pending()).toEqual({}) })",
            None,
        ),
        (
            "const { pending } = require('actions'); test('foo', () => { expect(pending()).toEqual({}) })",
            None,
        ),
        (
            "test('foo', () => { const pending = getPending(); expect(pending()).toEqual({}) })",
            None,
        ),
        (
            "test('foo', () => { expect(pending()).toEqual({}) }); function pending() { return {} }",
            None,
        ),
        ("import { test } from './test-utils'; test('something');", None),
    ];

    let mut fail = vec![
        ("describe.skip('foo', function () {})", None),
        ("describe.skip.each([1, 2, 3])('%s', (a, b) => {});", None),
        ("xdescribe.each([1, 2, 3])('%s', (a, b) => {});", None),
        ("describe[`skip`]('foo', function () {})", None),
        ("describe['skip']('foo', function () {})", None),
        ("it.skip('foo', function () {})", None),
        ("it['skip']('foo', function () {})", None),
        ("test.skip('foo', function () {})", None),
        ("it.skip.each``('foo', function () {})", None),
        ("test.skip.each``('foo', function () {})", None),
        ("it.skip.each([])('foo', function () {})", None),
        ("test.skip.each([])('foo', function () {})", None),
        ("test['skip']('foo', function () {})", None),
        ("xdescribe('foo', function () {})", None),
        ("xit('foo', function () {})", None),
        ("xtest('foo', function () {})", None),
        ("xit.each``('foo', function () {})", None),
        ("xtest.each``('foo', function () {})", None),
        ("xit.each([])('foo', function () {})", None),
        ("xtest.each([])('foo', function () {})", None),
        ("it('has title but no callback')", None),
        ("test('has title but no callback')", None),
        ("it('contains a call to pending', function () { pending() })", None),
        ("pending()", None),
        ("describe('contains a call to pending', function () { pending() })", None),
        ("import { test } from '@jest/globals';test('something');", None),
    ];

    let pass_vitest = vec![
        r#"describe("foo", function () {})"#,
        r#"it("foo", function () {})"#,
        r#"describe.only("foo", function () {})"#,
        r#"it.only("foo", function () {})"#,
        r#"it.each("foo", () => {})"#,
        r#"it.concurrent("foo", function () {})"#,
        r#"test("foo", function () {})"#,
        r#"test.only("foo", function () {})"#,
        r#"test.concurrent("foo", function () {})"#,
        r#"describe[`${"skip"}`]("foo", function () {})"#,
        r#"it.todo("fill this later")"#,
        "var appliedSkip = describe.skip; appliedSkip.apply(describe)",
        "var calledSkip = it.skip; calledSkip.call(it)",
        "({ f: function () {} }).f()",
        "(a || b).f()",
        "itHappensToStartWithIt()",
        "testSomething()",
        "xitSomethingElse()",
        "xitiViewMap()",
        r#"
            import { pending } from "actions"
            test("foo", () => {
              expect(pending()).toEqual({})
            })
        "#,
        "
            import { test } from './test-utils';
	    test('something');
        ",
    ];

    let fail_vitest = vec![
        r#"describe.skip("foo", function () {})"#,
        r#"xtest("foo", function () {})"#,
        r#"xit.each``("foo", function () {})"#,
        r#"xtest.each``("foo", function () {})"#,
        r#"xit.each([])("foo", function () {})"#,
        r#"it("has title but no callback")"#,
        r#"test("has title but no callback")"#,
        r#"it("contains a call to pending", function () { pending() })"#,
        "pending();",
        r#"
            import { describe } from 'vitest'; 
            describe.skip("foo", function () {})
        "#,
    ];

    pass.extend(pass_vitest.into_iter().map(|x| (x, None)));
    fail.extend(fail_vitest.into_iter().map(|x| (x, None)));

    Tester::new(NoDisabledTests::NAME, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
