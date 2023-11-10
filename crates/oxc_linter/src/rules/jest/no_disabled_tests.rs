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
        collect_possible_jest_call_node, parse_general_jest_fn_call_new, JestFnKind,
        JestGeneralFnKind, ParsedGeneralJestFnCallNew, PossibleJestNode,
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
    NoDisabledTests,
    correctness
);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(no-disabled-tests): {0:?}")]
#[diagnostic(severity(warning), help("{1:?}"))]
struct NoDisabledTestsDiagnostic(&'static str, &'static str, #[label] pub Span);

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
    fn run_once(&self, ctx: &LintContext) {
        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            run(possible_jest_node, ctx);
        }
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let original = possible_jest_node.original;
    if let AstKind::CallExpression(call_expr) = node.kind() {
        if let Some(jest_fn_call) = parse_general_jest_fn_call_new(call_expr, node, original, ctx) {
            let ParsedGeneralJestFnCallNew { kind, members, name } = jest_fn_call;
            // `test('foo')`
            let kind = match kind {
                JestFnKind::Expect | JestFnKind::Unknown => return,
                JestFnKind::General(kind) => kind,
            };
            if matches!(kind, JestGeneralFnKind::Test)
                && call_expr.arguments.len() < 2
                && members.iter().all(|member| member.is_name_unequal("todo"))
            {
                let (error, help) = Message::MissingFunction.details();
                ctx.diagnostic(NoDisabledTestsDiagnostic(error, help, call_expr.span));
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
                ctx.diagnostic(NoDisabledTestsDiagnostic(error, help, call_expr.callee.span()));
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
                ctx.diagnostic(NoDisabledTestsDiagnostic(error, help, call_expr.callee.span()));
            }
        } else if let Expression::Identifier(ident) = &call_expr.callee {
            if ident.name.as_str() == "pending"
                && ctx.semantic().is_reference_to_global_variable(ident)
            {
                // `describe('foo', function () { pending() })`
                let (error, help) = Message::Pending.details();
                ctx.diagnostic(NoDisabledTestsDiagnostic(error, help, call_expr.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
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

    let fail = vec![
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

    Tester::new(NoDisabledTests::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
