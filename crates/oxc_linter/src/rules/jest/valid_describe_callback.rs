use oxc_ast::{
    ast::{Argument, Expression, FunctionBody, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, parse_general_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

fn valid_describe_callback_diagnostic(x0: &str, x1: &str, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("eslint-plugin-jest(valid-describe-callback): {x0:?}"))
        .with_help(format!("{x1:?}"))
        .with_label(span2)
}

#[derive(Debug, Default, Clone)]
pub struct ValidDescribeCallback;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule validates that the second parameter of a `describe()` function is a
    /// callback function. This callback function:
    /// - should not be
    /// [async](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function)
    /// - should not contain any parameters
    /// - should not contain any `return` statements
    ///
    /// ### Why is this bad?
    ///
    /// Using an improper `describe()` callback function can lead to unexpected test
    /// errors.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // Async callback functions are not allowed
    /// describe('myFunction()', async () => {
    ///   // ...
    /// });
    ///
    /// // Callback function parameters are not allowed
    /// describe('myFunction()', done => {
    ///   // ...
    /// });
    ///
    /// // Returning a value from a describe block is not allowed
    /// describe('myFunction', () =>
    ///   it('returns a truthy value', () => {
    ///     expect(myFunction()).toBeTruthy();
    /// }));
    /// ```
    ValidDescribeCallback,
    correctness
);

impl Rule for ValidDescribeCallback {
    fn run_once(&self, ctx: &LintContext) {
        for node in &collect_possible_jest_call_node(ctx) {
            run(node, ctx);
        }
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) else {
        return;
    };
    if !matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe)) {
        return;
    }

    if call_expr.arguments.len() == 0 {
        diagnostic(ctx, call_expr.span, Message::NameAndCallback);
        return;
    }

    if call_expr.arguments.len() == 1 {
        // For better error notice, we locate it to arguments[0]
        diagnostic(ctx, call_expr.arguments[0].span(), Message::NameAndCallback);
        return;
    }

    match &call_expr.arguments[1] {
        Argument::FunctionExpression(fn_expr) => {
            if fn_expr.r#async {
                diagnostic(ctx, fn_expr.span, Message::NoAsyncDescribeCallback);
            }
            let no_each_fields =
                jest_fn_call.members.iter().all(|member| member.is_name_unequal("each"));
            if no_each_fields && fn_expr.params.parameters_count() > 0 {
                diagnostic(ctx, fn_expr.span, Message::UnexpectedDescribeArgument);
            }

            let Some(ref body) = fn_expr.body else {
                return;
            };
            if let Some(span) = find_first_return_stmt_span(body) {
                diagnostic(ctx, span, Message::UnexpectedReturnInDescribe);
            }
        }
        Argument::ArrowFunctionExpression(arrow_expr) => {
            if arrow_expr.r#async {
                diagnostic(ctx, arrow_expr.span, Message::NoAsyncDescribeCallback);
            }
            let no_each_fields =
                jest_fn_call.members.iter().all(|member| member.is_name_unequal("each"));
            if no_each_fields && arrow_expr.params.parameters_count() > 0 {
                diagnostic(ctx, arrow_expr.span, Message::UnexpectedDescribeArgument);
            }

            if arrow_expr.expression && arrow_expr.body.statements.len() > 0 {
                let stmt = &arrow_expr.body.statements[0];
                let Statement::ExpressionStatement(expr_stmt) = stmt else {
                    return;
                };
                if let Expression::CallExpression(call_expr) = &expr_stmt.expression {
                    diagnostic(ctx, call_expr.span, Message::UnexpectedReturnInDescribe);
                }
            }

            if let Some(span) = find_first_return_stmt_span(&arrow_expr.body) {
                diagnostic(ctx, span, Message::UnexpectedReturnInDescribe);
            }
        }
        callback => diagnostic(ctx, callback.span(), Message::SecondArgumentMustBeFunction),
    }
}

fn find_first_return_stmt_span(function_body: &FunctionBody) -> Option<Span> {
    function_body.statements.iter().find_map(|stmt| {
        if let Statement::ReturnStatement(return_stmt) = stmt {
            Some(return_stmt.span)
        } else {
            None
        }
    })
}

fn diagnostic(ctx: &LintContext, span: Span, message: Message) {
    let (error, help) = message.details();
    ctx.diagnostic(valid_describe_callback_diagnostic(error, help, span));
}

#[derive(Clone, Copy)]
enum Message {
    NameAndCallback,
    SecondArgumentMustBeFunction,
    NoAsyncDescribeCallback,
    UnexpectedDescribeArgument,
    UnexpectedReturnInDescribe,
}

impl Message {
    pub fn details(self) -> (&'static str, &'static str) {
        match self {
            Self::NameAndCallback => (
                "Describe requires name and callback arguments",
                "Add name as first argument and callback as second argument",
            ),
            Self::SecondArgumentMustBeFunction => {
                ("Second argument must be a function", "Replace second argument with a function")
            }
            Self::NoAsyncDescribeCallback => {
                ("No async describe callback", "Remove `async` keyword")
            }
            Self::UnexpectedDescribeArgument => (
                "Unexpected argument(s) in describe callback",
                "Remove argument(s) of describe callback",
            ),
            Self::UnexpectedReturnInDescribe => (
                "Unexpected return statement in describe callback",
                "Remove return statement in your describe callback",
            ),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("describe.each([1, 2, 3])('%s', (a, b) => {});", None),
        ("describe('foo', function() {})", None),
        ("describe('foo', () => {})", None),
        ("describe(`foo`, () => {})", None),
        ("xdescribe('foo\', () => {})", None),
        ("fdescribe('foo', () => {})", None),
        ("describe.only('foo', () => {})", None),
        ("describe.skip('foo', () => {})", None),
        (
            "
            describe('foo', () => {
                it('bar', () => {
                    return Promise.resolve(42).then(value => {
                        expect(value).toBe(42)
                    })
                })
            })
            ",
            None,
        ),
        (
            "
            describe('foo', () => {
                it('bar', async () => {
                    expect(await Promise.resolve(42)).toBe(42)
                })
            })
            ",
            None,
        ),
        ("if (hasOwnProperty(obj, key)) {}", None),
        (
            "
            describe.each`
                foo  | foe
                ${'1'} | ${'2'}
            `('$something', ({ foo, foe }) => {});
            ",
            None,
        ),
    ];

    let fail = vec![
        ("describe.each()()", None),
        ("describe['each']()()", None),
        ("describe.each(() => {})()", None),
        ("describe.each(() => {})('foo')", None),
        ("describe.each()(() => {})", None),
        ("describe['each']()(() => {})", None),
        ("describe.each('foo')(() => {})", None),
        ("describe.only.each('foo')(() => {})", None),
        ("describe(() => {})", None),
        ("describe('foo')", None),
        ("describe('foo', 'foo2')", None),
        ("describe()", None),
        ("describe('foo', async () => {})", None),
        ("describe('foo', async function () {})", None),
        ("xdescribe('foo', async function () {})", None),
        ("fdescribe('foo', async function () {})", None),
        (
            "
            import { fdescribe } from '@jest/globals';
            fdescribe('foo', async function () {})
            ",
            None,
        ),
        ("describe.only('foo', async function () {})", None),
        ("describe.skip('foo', async function () {})", None),
        (
            "
            describe('sample case', () => {
                it('works', () => {
                    expect(true).toEqual(true);
                });
                describe('async', async () => {
                    await new Promise(setImmediate);
                    it('breaks', () => {
                        throw new Error('Fail');
                    });
                });
            });
            ",
            None,
        ),
        (
            "
            describe('foo', function () {
                return Promise.resolve().then(() => {
                    it('breaks', () => {
                        throw new Error('Fail')
                    })
                })
            })
            ",
            None,
        ),
        (
            "
            describe('foo', () => {
                return Promise.resolve().then(() => {
                    it('breaks', () => {
                        throw new Error('Fail')
                    })
                })
                describe('nested', () => {
                    return Promise.resolve().then(() => {
                        it('breaks', () => {
                            throw new Error('Fail')
                        })
                    })
                })
            })
            ",
            None,
        ),
        (
            "
            describe('foo', async () => {
                await something()
                it('does something')
                describe('nested', () => {
                    return Promise.resolve().then(() => {
                        it('breaks', () => {
                            throw new Error('Fail')
                        })
                    })
                })
            })
            ",
            None,
        ),
        ("describe('foo', () => test('bar', () => {})) ", None),
        ("describe('foo', done => {})", None),
        ("describe('foo', function (done) {})", None),
        ("describe('foo', function (one, two, three) {})", None),
        ("describe('foo', async function (done) {})", None),
    ];

    Tester::new(ValidDescribeCallback::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
