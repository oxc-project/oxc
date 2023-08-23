use oxc_ast::{
    ast::{Argument, Expression, FunctionBody, Statement},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    jest_ast_util::{parse_general_jest_fn_call, JestFnKind, JestGeneralFnKind},
    rule::Rule,
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("{0:?}")]
#[diagnostic(severity(warning), help("{1:?}"))]
struct ValidDescribeCallbackDiagnostic(&'static str, &'static str, #[label] pub Span);

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
    // Because this rule has one test case not passed, will set to correctness when finished.
    correctness
);

impl Rule for ValidDescribeCallback {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, node, ctx) else { return };
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

        let callback = &call_expr.arguments[1];
        match callback {
            Argument::Expression(expr) => match expr {
                Expression::FunctionExpression(fn_expr) => {
                    if fn_expr.r#async {
                        diagnostic(ctx, fn_expr.span, Message::NoAsyncDescribeCallback);
                    }
                    let no_each_fields =
                        jest_fn_call.members.iter().all(|member| member.is_name_unequal("each"));
                    if no_each_fields && fn_expr.params.parameters_count() > 0 {
                        diagnostic(ctx, fn_expr.span, Message::UnexpectedDescribeArgument);
                    }

                    let Some(ref body) = fn_expr.body else { return;};
                    if let Some(span) = find_first_return_stmt_span(body) {
                        diagnostic(ctx, span, Message::UnexpectedReturnInDescribe);
                    }
                }
                Expression::ArrowExpression(arrow_expr) => {
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
                        let Statement::ExpressionStatement(expr_stmt) = stmt else { return; };
                        if let Expression::CallExpression(call_expr) = &expr_stmt.expression {
                            diagnostic(ctx, call_expr.span, Message::UnexpectedReturnInDescribe);
                        }
                    }

                    if let Some(span) = find_first_return_stmt_span(&arrow_expr.body) {
                        diagnostic(ctx, span, Message::UnexpectedReturnInDescribe);
                    }
                }
                _ => diagnostic(ctx, expr.span(), Message::SecondArgumentMustBeFunction),
            },
            Argument::SpreadElement(spreed_element) => {
                diagnostic(ctx, spreed_element.span, Message::SecondArgumentMustBeFunction);
            }
        }
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
    ctx.diagnostic(ValidDescribeCallbackDiagnostic(error, help, span));
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

#[allow(clippy::too_many_lines)]
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

    Tester::new(ValidDescribeCallback::NAME, pass, fail).test_and_snapshot();
}
