use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    AstNode as CrateAstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call,
        parse_expect_jest_fn_call,
    },
};

fn valid_expect_in_promise_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Promise containing expect was not returned or awaited")
        .with_help("Return or await the promise to ensure the expects in its chain are called")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ValidExpectInPromise;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that promises containing `expect` assertions are properly returned or awaited
    /// in test functions.
    ///
    /// ### Why is this bad?
    ///
    /// When a promise containing `expect` calls in its `.then()`, `.catch()`, or `.finally()`
    /// callbacks is not returned or awaited, the test may complete before the assertions run.
    /// This can lead to tests that pass even when the assertions would fail, giving false
    /// confidence in the code being tested.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// it('promises a person', () => {
    ///   api.getPersonByName('bob').then(person => {
    ///     expect(person).toHaveProperty('name', 'Bob');
    ///   });
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// it('promises a person', async () => {
    ///   await api.getPersonByName('bob').then(person => {
    ///     expect(person).toHaveProperty('name', 'Bob');
    ///   });
    /// });
    ///
    /// it('promises a person', () => {
    ///   return api.getPersonByName('bob').then(person => {
    ///     expect(person).toHaveProperty('name', 'Bob');
    ///   });
    /// });
    /// ```
    ValidExpectInPromise,
    jest,
    correctness,
    pending
);

impl Rule for ValidExpectInPromise {
    fn run<'a>(&self, node: &CrateAstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_potential_expect_call(call_expr) {
            return;
        }

        let jest_node = PossibleJestNode { node, original: None };
        if parse_expect_jest_fn_call(call_expr, &jest_node, ctx).is_none() {
            return;
        }

        if let Some(span) = find_unhandled_promise_chain(node, ctx) {
            ctx.diagnostic(valid_expect_in_promise_diagnostic(span));
        }
    }
}

fn is_potential_expect_call(call_expr: &CallExpression) -> bool {
    if call_expr.callee.is_specific_id("expect") {
        return true;
    }

    if let Some(member_expr) = call_expr.callee.get_member_expr() {
        let mut obj: &Expression<'_> = member_expr.object();
        loop {
            if let Expression::CallExpression(call) = obj {
                if call.callee.is_specific_id("expect") {
                    return true;
                }
                if let Some(inner_member) = call.callee.get_member_expr() {
                    obj = inner_member.object();
                    continue;
                }
            }
            break;
        }
    }

    false
}

/// Walks up from an expect() call to find if it's inside an unhandled promise callback.
fn find_unhandled_promise_chain<'a>(
    expect_node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<Span> {
    let mut current = expect_node;

    loop {
        let parent = ctx.nodes().parent_node(current.id());

        match parent.kind() {
            AstKind::ArrowFunctionExpression(_) | AstKind::Function(_) => {
                let grandparent = ctx.nodes().parent_node(parent.id());

                if let AstKind::CallExpression(call_expr) = grandparent.kind()
                    && is_promise_method_call(call_expr)
                {
                    let chain_root = find_promise_chain_root(grandparent, ctx);

                    if !is_in_test_callback(chain_root, ctx) {
                        return None;
                    }

                    // Bail out for legacy async pattern with done callback
                    if test_has_done_callback(chain_root, ctx) {
                        return None;
                    }

                    if !is_promise_handled(chain_root, ctx)
                        && let AstKind::CallExpression(root_call) = chain_root.kind()
                    {
                        return Some(root_call.span);
                    }
                    return None;
                }
            }

            AstKind::CallExpression(call_expr) => {
                let jest_node = PossibleJestNode { node: parent, original: None };
                if is_type_of_jest_fn_call(
                    call_expr,
                    &jest_node,
                    ctx,
                    &[
                        JestFnKind::General(JestGeneralFnKind::Test),
                        JestFnKind::General(JestGeneralFnKind::Hook),
                    ],
                ) {
                    return None;
                }
            }

            AstKind::Program(_) => return None,
            _ => {}
        }

        current = parent;
    }
}

fn is_promise_method_call(call_expr: &CallExpression) -> bool {
    if let Some(member_expr) = call_expr.callee.get_member_expr()
        && let Some(prop_name) = member_expr.static_property_name()
    {
        return matches!(prop_name, "then" | "catch" | "finally");
    }
    false
}

fn is_promise_static_call(call_expr: &CallExpression) -> bool {
    if let Some(member_expr) = call_expr.callee.get_member_expr()
        && member_expr.object().is_specific_id("Promise")
        && let Some(prop) = member_expr.static_property_name()
    {
        return matches!(prop, "all" | "race" | "allSettled" | "any" | "resolve" | "reject");
    }
    false
}

/// Finds the outermost call in a promise chain (e.g., `.catch()` in `a().then().catch()`).
fn find_promise_chain_root<'a, 'b>(
    promise_call: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> &'b AstNode<'a> {
    let mut current = promise_call;

    loop {
        let parent = ctx.nodes().parent_node(current.id());

        match parent.kind() {
            AstKind::StaticMemberExpression(_) | AstKind::ComputedMemberExpression(_) => {
                let grandparent = ctx.nodes().parent_node(parent.id());
                if let AstKind::CallExpression(call_expr) = grandparent.kind()
                    && is_promise_method_call(call_expr)
                {
                    current = grandparent;
                    continue;
                }
            }
            AstKind::ArrayExpression(_) => {
                let grandparent = ctx.nodes().parent_node(parent.id());
                if let AstKind::CallExpression(call_expr) = grandparent.kind()
                    && is_promise_static_call(call_expr)
                {
                    current = grandparent;
                    continue;
                }
            }
            _ => {}
        }

        return current;
    }
}

fn is_in_test_callback<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let mut current = node;

    loop {
        let parent = ctx.nodes().parent_node(current.id());

        if let AstKind::CallExpression(call_expr) = parent.kind() {
            let jest_node = PossibleJestNode { node: parent, original: None };
            if is_type_of_jest_fn_call(
                call_expr,
                &jest_node,
                ctx,
                &[
                    JestFnKind::General(JestGeneralFnKind::Test),
                    JestFnKind::General(JestGeneralFnKind::Hook),
                ],
            ) {
                return true;
            }
        }

        if matches!(parent.kind(), AstKind::Program(_)) {
            return false;
        }

        current = parent;
    }
}

/// Check if the test callback has a `done` parameter (legacy async pattern).
/// When present, we bail out since promise handling is too complex to analyze.
fn test_has_done_callback<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let mut current = node;

    loop {
        let parent = ctx.nodes().parent_node(current.id());

        match parent.kind() {
            AstKind::ArrowFunctionExpression(arrow) => {
                let grandparent = ctx.nodes().parent_node(parent.id());
                if let AstKind::CallExpression(call_expr) = grandparent.kind() {
                    let jest_node = PossibleJestNode { node: grandparent, original: None };
                    if is_type_of_jest_fn_call(
                        call_expr,
                        &jest_node,
                        ctx,
                        &[
                            JestFnKind::General(JestGeneralFnKind::Test),
                            JestFnKind::General(JestGeneralFnKind::Hook),
                        ],
                    ) {
                        return !arrow.params.items.is_empty();
                    }
                }
            }
            AstKind::Function(func) => {
                let grandparent = ctx.nodes().parent_node(parent.id());
                if let AstKind::CallExpression(call_expr) = grandparent.kind() {
                    let jest_node = PossibleJestNode { node: grandparent, original: None };
                    if is_type_of_jest_fn_call(
                        call_expr,
                        &jest_node,
                        ctx,
                        &[
                            JestFnKind::General(JestGeneralFnKind::Test),
                            JestFnKind::General(JestGeneralFnKind::Hook),
                        ],
                    ) {
                        return !func.params.items.is_empty();
                    }
                }
            }
            AstKind::Program(_) => return false,
            _ => {}
        }

        current = parent;
    }
}

fn is_promise_handled<'a>(promise_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let mut current = promise_node;

    loop {
        let parent = ctx.nodes().parent_node(current.id());

        match parent.kind() {
            AstKind::AwaitExpression(_) | AstKind::ReturnStatement(_) => return true,
            AstKind::ExpressionStatement(_) => {
                // Check for implicit return in expression arrow function
                let grandparent = ctx.nodes().parent_node(parent.id());
                if let AstKind::FunctionBody(_) = grandparent.kind() {
                    let great_grandparent = ctx.nodes().parent_node(grandparent.id());
                    if let AstKind::ArrowFunctionExpression(arrow) = great_grandparent.kind()
                        && arrow.expression
                    {
                        return true;
                    }
                }
                return false;
            }
            AstKind::VariableDeclarator(_) | AstKind::Program(_) | AstKind::FunctionBody(_) => {
                return false;
            }
            AstKind::CallExpression(call_expr) => {
                if is_promise_static_call(call_expr) {
                    current = parent;
                    continue;
                }
            }
            AstKind::ArrayExpression(_) => {
                current = parent;
                continue;
            }
            _ => {}
        }

        current = parent;
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"
            it('passes', async () => {
                await somePromise().then(data => {
                    expect(data).toBe('foo');
                });
            });
        "#,
        r#"
            it('passes', () => {
                return somePromise().then(data => {
                    expect(data).toBe('foo');
                });
            });
        "#,
        r#"
            it('passes', () => somePromise().then(data => expect(data).toBe('foo')));
        "#,
        r#"
            it('passes', async () => {
                await somePromise().catch(err => {
                    expect(err).toBeInstanceOf(Error);
                });
            });
        "#,
        r#"
            it('passes', () => {
                return somePromise().catch(err => {
                    expect(err).toBeInstanceOf(Error);
                });
            });
        "#,
        r#"
            it('passes', async () => {
                await somePromise().finally(() => {
                    expect(cleanup).toHaveBeenCalled();
                });
            });
        "#,
        r#"
            it('passes', async () => {
                await Promise.all([
                    somePromise().then(data => expect(data).toBe('foo')),
                    otherPromise().then(data => expect(data).toBe('bar'))
                ]);
            });
        "#,
        r#"
            it('passes', () => {
                return Promise.all([
                    somePromise().then(data => expect(data).toBe('foo')),
                    otherPromise().then(data => expect(data).toBe('bar'))
                ]);
            });
        "#,
        r#"
            it('passes', async () => {
                await Promise.race([
                    somePromise().then(data => expect(data).toBe('foo')),
                    otherPromise().then(data => expect(data).toBe('bar'))
                ]);
            });
        "#,
        r#"
            it('passes', async () => {
                await Promise.allSettled([
                    somePromise().then(data => expect(data).toBe('foo'))
                ]);
            });
        "#,
        r#"
            it('passes', async () => {
                await Promise.any([
                    somePromise().then(data => expect(data).toBe('foo'))
                ]);
            });
        "#,
        r#"
            it('passes', () => {
                somePromise().then(data => {
                    console.log(data);
                });
                expect(true).toBe(true);
            });
        "#,
        r#"
            it('passes', async () => {
                await somePromise()
                    .then(data => data.json())
                    .then(json => {
                        expect(json).toHaveProperty('foo');
                    });
            });
        "#,
        r#"
            it('passes', () => {
                return somePromise()
                    .then(data => {
                        expect(data).toBe('foo');
                    })
                    .catch(err => {
                        expect(err).toBeInstanceOf(Error);
                    });
            });
        "#,
        r#"
            it('passes', async () => {
                await Promise.resolve().then(() => {
                    expect(true).toBe(true);
                });
            });
        "#,
        r#"
            it('passes', () => {
                expect(true).toBe(true);
            });
        "#,
        r#"
            it('passes', async () => {
                await expect(somePromise()).resolves.toBe('foo');
            });
        "#,
        r#"
            it('passes', async () => {
                await expect(somePromise()).rejects.toThrow();
            });
        "#,
        r#"
            it('passes', done => {
                somePromise().then(data => {
                    expect(data).toBe('foo');
                    done();
                });
            });
        "#,
        r#"
            it('passes', function(done) {
                somePromise().then(data => {
                    expect(data).toBe('foo');
                    done();
                });
            });
        "#,
        r#"
            beforeEach(done => {
                somePromise().then(data => {
                    expect(data).toBe('foo');
                    done();
                });
            });
        "#,
    ];

    let fail = vec![
        r#"
            it('fails', () => {
                somePromise().then(data => {
                    expect(data).toBe('foo');
                });
            });
        "#,
        r#"
            it('fails', () => {
                somePromise().catch(err => {
                    expect(err).toBeInstanceOf(Error);
                });
            });
        "#,
        r#"
            it('fails', () => {
                somePromise().finally(() => {
                    expect(cleanup).toHaveBeenCalled();
                });
            });
        "#,
        r#"
            it('fails', () => {
                Promise.all([
                    somePromise().then(data => expect(data).toBe('foo'))
                ]);
            });
        "#,
        r#"
            it('fails', () => {
                Promise.race([
                    somePromise().then(data => expect(data).toBe('foo'))
                ]);
            });
        "#,
        r#"
            it('fails', () => {
                const promise = somePromise().then(data => {
                    expect(data).toBe('foo');
                });
            });
        "#,
        r#"
            it('fails', () => {
                somePromise().then(data => {
                    expect(data).toBe('foo');
                });
                otherPromise().then(data => {
                    expect(data).toBe('bar');
                });
            });
        "#,
        r#"
            it('fails', () => {
                somePromise()
                    .then(data => data.json())
                    .then(json => {
                        expect(json).toHaveProperty('foo');
                    });
            });
        "#,
        r#"
            it('fails', async () => {
                somePromise().then(data => {
                    expect(data).toBe('foo');
                });
            });
        "#,
        r#"
            it('fails', () => {
                Promise.resolve().then(() => {
                    expect(true).toBe(true);
                });
            });
        "#,
        r#"
            it('fails', () => {
                const promise = somePromise().then(data => {
                    expect(data).toBe('foo');
                });
                promise.then(() => {
                    expect(true).toBe(true);
                });
            });
        "#,
    ];

    Tester::new(ValidExpectInPromise::NAME, ValidExpectInPromise::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
