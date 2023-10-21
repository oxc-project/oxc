use oxc_ast::{
    ast::{Argument, CallExpression, Expression, FormalParameters},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        get_node_name, parse_general_jest_fn_call, JestFnKind, JestGeneralFnKind,
        ParsedGeneralJestFnCall,
    },
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(no-done-callback): {0:?}")]
#[diagnostic(severity(warning), help("{1:?}"))]
struct NoDoneCallbackDiagnostic(&'static str, &'static str, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDoneCallback;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule checks the function parameter of hooks & tests for use of the done argument, suggesting you return a promise instead.
    ///
    /// ### Why is this bad?
    ///
    /// When calling asynchronous code in hooks and tests, jest needs to know when the asynchronous work is complete to progress the current run.
    /// Originally the most common pattern to achieve this was to use callbacks:
    ///
    /// ```javascript
    /// test('the data is peanut butter', done => {
    ///   function callback(data) {
    ///     try {
    ///       expect(data).toBe('peanut butter');
    ///       done();
    ///     } catch (error) {
    ///       done(error);
    ///     }
    ///   }
    ///
    ///   fetchData(callback);
    /// });
    /// ```
    ///
    /// This can be very error-prone however, as it requires careful understanding of how assertions work in tests or otherwise tests won't behave as expected.
    ///
    /// ### Example
    /// ```javascript
    /// beforeEach(done => {
    ///   // ...
    /// });
    ///
    /// test('myFunction()', done => {
    ///   // ...
    /// });
    ///
    /// test('myFunction()', function (done) {
    ///   // ...
    /// });
    /// ```
    NoDoneCallback,
    // TODO: add suggestion (see jest-community/eslint-plugin-jest#586)
    restriction
);

impl Rule for NoDoneCallback {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call_expr) = node.kind() {
            if let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, node, ctx) {
                let ParsedGeneralJestFnCall { kind, .. } = jest_fn_call;
                if !matches!(
                    kind,
                    JestFnKind::General(JestGeneralFnKind::Test | JestGeneralFnKind::Hook)
                ) {
                    return;
                }

                let is_jest_each = get_node_name(&call_expr.callee).ends_with("each");

                if is_jest_each
                    && !matches!(call_expr.callee, Expression::TaggedTemplateExpression(_))
                {
                    // isJestEach but not a TaggedTemplateExpression, so this must be
                    // the `jest.each([])()` syntax which this rule doesn't support due
                    // to its complexity (see jest-community/eslint-plugin-jest#710)
                    return;
                }

                let Some(Argument::Expression(expr)) =
                    find_argument_of_callback(call_expr, is_jest_each, kind)
                else {
                    return;
                };

                let callback_arg_index = usize::from(is_jest_each);

                match expr {
                    Expression::FunctionExpression(func_expr) => {
                        if func_expr.params.parameters_count() != 1 + callback_arg_index {
                            return;
                        }
                        let Some(span) = get_span_of_first_parameter(&func_expr.params) else {
                            return;
                        };

                        if func_expr.r#async {
                            let (message, help) = Message::UseAwaitInsteadOfCallback.details();
                            ctx.diagnostic(NoDoneCallbackDiagnostic(message, help, span));
                            return;
                        }

                        let (message, help) = Message::NoDoneCallback.details();
                        ctx.diagnostic(NoDoneCallbackDiagnostic(message, help, span));
                    }
                    Expression::ArrowExpression(arrow_expr) => {
                        if arrow_expr.params.parameters_count() != 1 + callback_arg_index {
                            return;
                        }

                        let Some(span) = get_span_of_first_parameter(&arrow_expr.params) else {
                            return;
                        };

                        if arrow_expr.r#async {
                            let (message, help) = Message::UseAwaitInsteadOfCallback.details();
                            ctx.diagnostic(NoDoneCallbackDiagnostic(message, help, span));
                            return;
                        }

                        let (message, help) = Message::NoDoneCallback.details();
                        ctx.diagnostic(NoDoneCallbackDiagnostic(message, help, span));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn get_span_of_first_parameter(params: &FormalParameters) -> Option<Span> {
    let span = params.items.first().map(|param| param.span);
    if span.is_none() {
        return params.rest.as_ref().map(|rest| rest.span);
    }

    span
}

fn find_argument_of_callback<'a>(
    call_expr: &'a CallExpression<'a>,
    is_jest_each: bool,
    kind: JestFnKind,
) -> Option<&'a Argument<'a>> {
    if is_jest_each {
        return call_expr.arguments.get(1);
    }

    if matches!(kind, JestFnKind::General(JestGeneralFnKind::Hook)) {
        return call_expr.arguments.get(0);
    }

    if matches!(kind, JestFnKind::General(JestGeneralFnKind::Test)) {
        return call_expr.arguments.get(1);
    }

    None
}

enum Message {
    NoDoneCallback,
    UseAwaitInsteadOfCallback,
}

impl Message {
    fn details(&self) -> (&'static str, &'static str) {
        match self {
            Self::NoDoneCallback => (
                "Function parameter(s) use the `done` argument",
                "Return a Promise instead of relying on callback parameter",
            ),
            Self::UseAwaitInsteadOfCallback => (
                "Function parameter(s) use the `done` argument",
                "Use await instead of callback in async functions",
            ),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("test('something', () => {})", None),
        ("test('something', async () => {})", None),
        ("test('something', function() {})", None),
        ("test.each``('something', ({ a, b }) => {})", None),
        ("test.each()('something', ({ a, b }) => {})", None),
        ("it.each()('something', ({ a, b }) => {})", None),
        ("it.each([])('something', (a, b) => {})", None),
        ("it.each``('something', ({ a, b }) => {})", None),
        ("it.each([])('something', (a, b) => { a(); b(); })", None),
        ("it.each``('something', ({ a, b }) => { a(); b(); })", None),
        ("test('something', async function () {})", None),
        ("test('something', someArg)", None),
        ("beforeEach(() => {})", None),
        ("beforeAll(async () => {})", None),
        ("afterAll(() => {})", None),
        ("afterAll(async function () {})", None),
        ("afterAll(async function () {}, 5)", None),
    ];

    let fail = vec![
        ("test('something', (...args) => {args[0]();})", None),
        ("test('something', done => {done();})", None),
        ("test('something', (done,) => {done();})", None),
        ("test('something', finished => {finished();})", None),
        ("test('something', (done) => {done();})", None),
        ("test('something', done => done())", None),
        ("test('something', (done) => done())", None),
        ("test('something', function(done) {done();})", None),
        ("test('something', function (done) {done();})", None),
        ("test('something', async done => {done();})", None),
        ("test('something', async done => done())", None),
        ("test('something', async function (done) {done();})", None),
        (
            "
                test('my test', async (done) => {
                    await myAsyncTask();
                    expect(true).toBe(false);
                    done();
                });
            ",
            None,
        ),
        (
            "
                test('something', (done) => {
                    done();
                });
            ",
            None,
        ),
        ("afterEach((...args) => {args[0]();})", None),
        ("beforeAll(done => {done();})", None),
        ("beforeAll(finished => {finished();})", None),
        ("beforeEach((done) => {done();})", None),
        ("afterAll(done => done())", None),
        ("afterEach((done) => done())", None),
        ("beforeAll(function(done) {done();})", None),
        ("afterEach(function (done) {done();})", None),
        ("beforeAll(async done => {done();})", None),
        ("beforeAll(async done => done())", None),
        ("beforeAll(async function (done) {done();})", None),
        (
            "
                afterAll(async (done) => {
                    await myAsyncTask();
                    done();
                });
            ",
            None,
        ),
        (
            "
                beforeEach((done) => {
                    done();
                });
            ",
            None,
        ),
        (
            "
                import { beforeEach } from '@jest/globals';

                beforeEach((done) => {
                    done();
                });
            ",
            None,
        ),
        (
            "
                import { beforeEach as atTheStartOfEachTest } from '@jest/globals';

                atTheStartOfEachTest((done) => {
                    done();
                });
            ",
            None,
        ),
        ("test.each``('something', ({ a, b }, done) => { done(); })", None),
        ("it.each``('something', ({ a, b }, done) => { done(); })", None),
    ];

    Tester::new(NoDoneCallback::NAME, pass, fail).test_and_snapshot();
}
