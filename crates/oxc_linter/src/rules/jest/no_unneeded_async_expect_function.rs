use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{ParsedJestFnCallNew, PossibleJestNode, parse_jest_fn_call},
};

fn no_unneeded_async_expect_function_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary async function wrapper")
        .with_help("Remove the async wrapper and pass the promise directly to expect")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnneededAsyncExpectFunction;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows unnecessary async function wrapper for expected promises.
    ///
    /// ### Why is this bad?
    ///
    /// When the only statement inside an async wrapper is `await someCall()`,
    /// the call should be passed directly to `expect` instead. This makes the
    /// test code more concise and easier to read.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// await expect(async () => {
    ///   await doSomethingAsync();
    /// }).rejects.toThrow();
    ///
    /// await expect(async () => await doSomethingAsync()).rejects.toThrow();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// await expect(doSomethingAsync()).rejects.toThrow();
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/no-unneeded-async-expect-function.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/no-unneeded-async-expect-function": "error"
    ///   }
    /// }
    /// ```
    NoUnneededAsyncExpectFunction,
    jest,
    style,
    fix
);

impl Rule for NoUnneededAsyncExpectFunction {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(ParsedJestFnCallNew::Expect(parsed_expect_call)) =
            parse_jest_fn_call(call_expr, jest_node, ctx)
        else {
            return;
        };

        // Ensure we have a valid matcher (this ensures we're processing the full chain)
        if parsed_expect_call.matcher().is_none() {
            return;
        }

        // Get the expect() CallExpression from head.parent
        let Some(Expression::CallExpression(expect_call_expr)) = parsed_expect_call.head.parent
        else {
            return;
        };

        // Get the first argument of expect()
        let Some(first_arg) = expect_call_expr.arguments.first() else {
            return;
        };

        // Check if it's an async function expression and get the inner call span
        let (func_span, inner_call_span) = match first_arg {
            Argument::ArrowFunctionExpression(arrow) => {
                if !arrow.r#async {
                    return;
                }
                let Some(inner_span) = get_awaited_call_span_from_arrow(arrow) else {
                    return;
                };
                (arrow.span, inner_span)
            }
            Argument::FunctionExpression(func) => {
                if !func.r#async {
                    return;
                }
                let Some(body) = &func.body else {
                    return;
                };
                let Some(inner_span) = get_awaited_call_span_from_block(body) else {
                    return;
                };
                (func.span, inner_span)
            }
            _ => return,
        };

        ctx.diagnostic_with_fix(no_unneeded_async_expect_function_diagnostic(func_span), |fixer| {
            fixer.replace(func_span, fixer.source_range(inner_call_span).to_string())
        });
    }
}

/// Gets the span of the awaited call expression from an async arrow function.
/// Returns `None` if the function body doesn't contain exactly one await of a call expression.
fn get_awaited_call_span_from_arrow(arrow: &oxc_ast::ast::ArrowFunctionExpression) -> Option<Span> {
    // Case 1: Arrow function with expression body (async () => await doSomething())
    if arrow.expression {
        if let Some(first) = arrow.body.statements.first()
            && let Statement::ExpressionStatement(expr_stmt) = first
            && let Expression::AwaitExpression(await_expr) = &expr_stmt.expression
            && let Expression::CallExpression(call) = &await_expr.argument
        {
            return Some(call.span);
        }
        return None;
    }

    // Case 2: Arrow function with block body
    get_awaited_call_span_from_block(&arrow.body)
}

fn get_awaited_call_span_from_block(body: &oxc_ast::ast::FunctionBody) -> Option<Span> {
    if body.statements.len() == 1
        && let Some(stmt) = body.statements.first()
        && let Statement::ExpressionStatement(expr_stmt) = stmt
        && let Expression::AwaitExpression(await_expr) = &expr_stmt.expression
        && let Expression::CallExpression(call) = &await_expr.argument
    {
        return Some(call.span);
    }

    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        "expect.hasAssertions()",
        "
			    it('pass', async () => {
			      expect();
			    })
			    ",
        "
			    it('pass', async () => {
			      await expect(doSomethingAsync()).rejects.toThrow();
			    })
			    ",
        "
			    it('pass', async () => {
			      await expect(doSomethingAsync(1, 2)).resolves.toBe(1);
			    })
			    ",
        "
			    it('pass', async () => {
			      await expect(async () => {
			        await doSomethingAsync();
			        await doSomethingTwiceAsync(1, 2);
			      }).rejects.toThrow();
			    })
			    ",
        "
			    it('pass', async () => {
			      await expect(async () => {
			        doSomethingAsync();
			      }).rejects.toThrow();
			    })
			    ",
        "
			    it('pass', async () => {
			      await expect(async () => {
			        const a = 1;
			        await doSomethingAsync(a);
			      }).rejects.toThrow();
			    })
			    ",
        "
			    it('pass for non-async expect', async () => {
			      await expect(() => {
			        doSomethingSync(a);
			      }).rejects.toThrow();
			    })
			    ",
        "
			    it('pass for await in expect', async () => {
			      await expect(await doSomethingAsync()).rejects.toThrow();
			    })
			    ",
        "
			    it('pass for different matchers', async () => {
			      await expect(await doSomething()).not.toThrow();
			      await expect(await doSomething()).toHaveLength(2);
			      await expect(await doSomething()).toHaveReturned();
			      await expect(await doSomething()).not.toHaveBeenCalled();
			      await expect(await doSomething()).not.toBeDefined();
			      await expect(await doSomething()).toEqual(2);
			    })
			    ",
        "
			    it('pass for using await within for-loop', async () => {
			      const b = [async () => Promise.resolve(1), async () => Promise.reject(2)];
			      await expect(async() => {
			        for (const a of b) {
			          await b();
			        }
			      }).rejects.toThrow();
			    })
			    ",
        "
			    it('pass for using await within array', async () => {
			      await expect(async() => [await Promise.reject(2)]).rejects.toThrow(2);
			    })
			    ",
        "
				import { expect as pleaseExpect } from '@jest/globals';
				it('pass', async () => {
				await pleaseExpect(doSomethingAsync()).rejects.toThrow();
				})",
    ];

    let fail = vec![
        "
			      it('should be fixed', async () => {
			        await expect(async () => {
			          await doSomethingAsync();
			        }).rejects.toThrow();
			      })
			      ",
        "
			      it('should be fixed', async () => {
			        await expect(async () => await doSomethingAsync()).rejects.toThrow();
			      })
			      ",
        "
			      it('should be fixed', async () => {
			        await expect(async function () {
			          await doSomethingAsync();
			        }).rejects.toThrow();
			      })
			      ",
        "
			        it('should be fixed for async arrow function', async () => {
			          await expect(async () => {
			            await doSomethingAsync(1, 2);
			          }).rejects.toThrow();
			        })
			      ",
        "
			        it('should be fixed for async normal function', async () => {
			          await expect(async function () {
			            await doSomethingAsync(1, 2);
			          }).rejects.toThrow();
			        })
			      ",
        "
			        it('should be fixed for Promise.all', async () => {
			          await expect(async function () {
			            await Promise.all([doSomethingAsync(1, 2), doSomethingAsync()]);
			          }).rejects.toThrow();
			        })
			      ",
        "
			        it('should be fixed for async ref to expect', async () => {
			          const a = async () => { await doSomethingAsync() };
			          await expect(async () => {
			            await a();
			          }).rejects.toThrow();
			        })
			      ",
    ];

    let fix = vec![
        ("
			      it('should be fixed', async () => {
			        await expect(async () => {
			          await doSomethingAsync();
			        }).rejects.toThrow();
			      })
			      ", "
			      it('should be fixed', async () => {
			        await expect(doSomethingAsync()).rejects.toThrow();
			      })
			      ", None),
("
			      it('should be fixed', async () => {
			        await expect(async () => await doSomethingAsync()).rejects.toThrow();
			      })
			      ", "
			      it('should be fixed', async () => {
			        await expect(doSomethingAsync()).rejects.toThrow();
			      })
			      ", None),
("
			      it('should be fixed', async () => {
			        await expect(async function () {
			          await doSomethingAsync();
			        }).rejects.toThrow();
			      })
			      ", "
			      it('should be fixed', async () => {
			        await expect(doSomethingAsync()).rejects.toThrow();
			      })
			      ", None),
("
			        it('should be fixed for async arrow function', async () => {
			          await expect(async () => {
			            await doSomethingAsync(1, 2);
			          }).rejects.toThrow();
			        })
			      ", "
			        it('should be fixed for async arrow function', async () => {
			          await expect(doSomethingAsync(1, 2)).rejects.toThrow();
			        })
			      ", None),
("
			        it('should be fixed for async normal function', async () => {
			          await expect(async function () {
			            await doSomethingAsync(1, 2);
			          }).rejects.toThrow();
			        })
			      ", "
			        it('should be fixed for async normal function', async () => {
			          await expect(doSomethingAsync(1, 2)).rejects.toThrow();
			        })
			      ", None),
("
			        it('should be fixed for Promise.all', async () => {
			          await expect(async function () {
			            await Promise.all([doSomethingAsync(1, 2), doSomethingAsync()]);
			          }).rejects.toThrow();
			        })
			      ", "
			        it('should be fixed for Promise.all', async () => {
			          await expect(Promise.all([doSomethingAsync(1, 2), doSomethingAsync()])).rejects.toThrow();
			        })
			      ", None),
("
			        it('should be fixed for async ref to expect', async () => {
			          const a = async () => { await doSomethingAsync() };
			          await expect(async () => {
			            await a();
			          }).rejects.toThrow();
			        })
			      ", "
			        it('should be fixed for async ref to expect', async () => {
			          const a = async () => { await doSomethingAsync() };
			          await expect(a()).rejects.toThrow();
			        })
			      ", None)
    ];

    let pass_vitest = vec![
        "
			    import { expect as pleaseExpect } from 'vitest';
			    it('pass', async () => {
			      await pleaseExpect(doSomethingAsync()).rejects.toThrow();
			    })
			    ",
    ];

    pass.extend(pass_vitest);

    Tester::new(
        NoUnneededAsyncExpectFunction::NAME,
        NoUnneededAsyncExpectFunction::PLUGIN,
        pass,
        fail,
    )
    .with_jest_plugin(true)
    .with_vitest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
