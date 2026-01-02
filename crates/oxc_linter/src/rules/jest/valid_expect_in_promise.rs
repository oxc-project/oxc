use oxc_ast::{
    AstKind,
    ast::{
        Argument, ArrayExpressionElement, AssignmentTarget, BindingPattern, CallExpression,
        Expression, Statement, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

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
    correctness
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

/// Check if a variable storing a promise is later awaited or returned in the same block.
fn is_variable_awaited_or_returned<'a>(
    decl: &VariableDeclarator<'a>,
    decl_node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    // Bail out on destructuring patterns (return true = no error)
    let BindingPattern::BindingIdentifier(binding) = &decl.id else {
        return true;
    };
    let var_name = binding.name.as_str();

    let mut current = decl_node;
    let statements: &[Statement<'_>] = loop {
        let parent = ctx.nodes().parent_node(current.id());
        match parent.kind() {
            AstKind::FunctionBody(body) => break body.statements.as_slice(),
            AstKind::Program(_) => return false,
            _ => current = parent,
        }
    };

    let decl_span = decl.span;
    let mut found_decl = false;

    for stmt in statements {
        if !found_decl {
            if stmt.span().start <= decl_span.start && stmt.span().end >= decl_span.end {
                found_decl = true;
            }
            continue;
        }

        match stmt {
            Statement::ExpressionStatement(expr_stmt) => {
                if let Expression::AwaitExpression(await_expr) = &expr_stmt.expression {
                    if await_expr.argument.is_specific_id(var_name) {
                        return true;
                    }
                    if is_variable_in_promise_all_or_all_settled(var_name, &await_expr.argument) {
                        return true;
                    }
                }
                if let Expression::AssignmentExpression(assign) = &expr_stmt.expression
                    && let AssignmentTarget::AssignmentTargetIdentifier(target) = &assign.left
                    && target.name.as_str() == var_name
                {
                    if is_chain_reassignment(&assign.right, var_name) {
                        continue;
                    }
                    return false;
                }
            }
            Statement::ReturnStatement(ret) => {
                if let Some(arg) = &ret.argument {
                    if arg.is_specific_id(var_name) {
                        return true;
                    }
                    if is_variable_in_promise_all_or_all_settled(var_name, arg) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }

    false
}

fn is_chain_reassignment(expr: &Expression<'_>, var_name: &str) -> bool {
    if let Expression::CallExpression(call) = expr
        && let Some(member) = call.callee.get_member_expr()
        && let Some(prop) = member.static_property_name()
        && matches!(prop, "then" | "catch" | "finally")
    {
        return member.object().is_specific_id(var_name)
            || is_chain_reassignment(member.object(), var_name);
    }
    false
}

/// Check if an expression is a `Promise.all` or `Promise.allSettled` call containing a variable.
fn is_variable_in_promise_all_or_all_settled(var_name: &str, expr: &Expression<'_>) -> bool {
    if let Expression::CallExpression(call) = expr
        && let Some(member) = call.callee.get_member_expr()
        && member.object().is_specific_id("Promise")
        && let Some(prop) = member.static_property_name()
        && matches!(prop, "all" | "allSettled")
        && let Some(Argument::ArrayExpression(arr)) = call.arguments.first()
    {
        return arr.elements.iter().any(|el| {
            matches!(el, ArrayExpressionElement::Identifier(id) if id.name.as_str() == var_name)
        });
    }
    false
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
            AstKind::VariableDeclarator(decl) => {
                return is_variable_awaited_or_returned(decl, parent, ctx);
            }
            AstKind::Program(_) | AstKind::FunctionBody(_) => {
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

    let pass_jest = vec![
        ("test('something', () => Promise.resolve().then(() => expect(1).toBe(2)));", None, None),
        ("Promise.resolve().then(() => expect(1).toBe(2))", None, None),
        ("const x = Promise.resolve().then(() => expect(1).toBe(2))", None, None),
        (r#"it.todo("something")"#, None, None),
        (
            "it('is valid', () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(promise).resolves.toBe(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(promise).resolves.not.toBe(2);
			});",
            None,
            None,
        ),
        (
            "it('is valid', () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(promise).rejects.toBe(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(promise).rejects.not.toBe(2);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(await promise).toBeGreaterThan(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(await promise).resolves.toBeGreaterThan(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(1).toBeGreaterThan(await promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect.this.that.is(await promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  expect(await loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  })).toBeGreaterThan(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect([await promise]).toHaveLength(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect([,,await promise,,]).toHaveLength(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect([[await promise]]).toHaveLength(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  logValue(await promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return 1;
			  });
			  expect.assertions(await promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  await loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => new Promise((done) => {
			  test()
			    .then(() => {
			      expect(someThing).toEqual(true);
			      done();
			    });
			}));",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return new Promise(done => {
			    test().then(() => {
			      expect(someThing).toEqual(true);
			      done();
			    });
			  });
			});",
            None,
            None,
        ),
        (
            "it('passes', () => {
			  Promise.resolve().then(() => {
			    grabber.grabSomething();
			  });
			});",
            None,
            None,
        ),
        (
            "it('passes', async () => {
			  const grabbing = Promise.resolve().then(() => {
			    grabber.grabSomething();
			  });
			  await grabbing;
			  expect(grabber.grabbedItems).toHaveLength(1);
			});",
            None,
            None,
        ),
        (
            "const myFn = () => {
			  Promise.resolve().then(() => {
			    expect(true).toBe(false);
			  });
			};",
            None,
            None,
        ),
        (
            "const myFn = () => {
			  Promise.resolve().then(() => {
			    subject.invokeMethod();
			  });
			};",
            None,
            None,
        ),
        (
            "const myFn = () => {
			  Promise.resolve().then(() => {
			    expect(true).toBe(false);
			  });
			};
			it('it1', () => {
			  return somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => new Promise((done) => {
			  test()
			    .finally(() => {
			      expect(someThing).toEqual(true);
			      done();
			    });
			}));",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.finally(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  return somePromise.catch(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "xtest('it1', function() {
			  return somePromise.catch(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  return somePromise.then(function() {
			    doSomeThingButNotExpect();
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  return getSomeThing().getPromise().then(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  return Promise.resolve().then(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  return Promise.resolve().then(function () {
			    /*fulfillment*/
			    expect(someThing).toEqual(true);
			  }, function () {
			    /*rejection*/
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  Promise.resolve().then(/*fulfillment*/ function () {
			  }, undefined, /*rejection*/ function () {
			    expect(someThing).toEqual(true)
			  })
			});",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  return Promise.resolve().then(function () {
			    /*fulfillment*/
			  }, function () {
			    /*rejection*/
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  return somePromise.then()
			});",
            None,
            None,
        ),
        (
            "it('it1', async () => {
			  await Promise.resolve().then(function () {
			    expect(someThing).toEqual(true)
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', async () => {
			  await somePromise.then(() => {
			    expect(someThing).toEqual(true)
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', async () => {
			  await getSomeThing().getPromise().then(function () {
			    expect(someThing).toEqual(true)
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  })
			  .then(() => {
			    expect(someThing).toEqual(true);
			  })
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.then(() => {
			    return value;
			  })
			  .then(value => {
			    expect(someThing).toEqual(value);
			  })
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  })
			  .then(() => {
			    console.log('this is silly');
			  })
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  })
			  .catch(() => {
			    expect(someThing).toEqual(false);
			  })
			});",
            None,
            None,
        ),
        (
            "test('later return', () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test.only('later return', () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return promise;
			});",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', () => {
			  const [promise] = something().then(value => {
			    expect(value).toBe('red');
			  });
			});",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', async () => {
			  const [promise] = await something().then(value => {
			    expect(value).toBe('red');
			  });
			});",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', () => {
			  const [promise] = [
			    something().then(value => {
			      expect(value).toBe('red');
			    })
			  ];
			});",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', () => {
			  const {promise} = {
			    promise: something().then(value => {
			      expect(value).toBe('red');
			    })
			  };
			});",
            None,
            None,
        ),
        (
            "test('that we bailout in complex cases', () => {
			  promiseSomething({
			    timeout: 500,
			    promise: something().then(value => {
			      expect(value).toBe('red');
			    })
			  });
			});",
            None,
            None,
        ),
        (
            "it('shorthand arrow', () =>
			  something().then(value => {
			    expect(() => {
			      value();
			    }).toThrow();
			  })
			);",
            None,
            None,
        ),
        (
            "it('crawls for files based on patterns', () => {
			  const promise = nodeCrawl({}).then(data => {
			    expect(childProcess.spawn).lastCalledWith('find');
			  });
			  return promise;
			});",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
			  const value = await somePromise().then(response => {
			    expect(response).toHaveProperty('data');
			    return response.data;
			  });
			  expect(value).toBe('hello world');
			});",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
			  return await somePromise().then(response => {
			    expect(response).toHaveProperty('data');
			    return response.data;
			  });
			});",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
			  return somePromise().then(response => {
			    expect(response).toHaveProperty('data');
			    return response.data;
			  });
			});",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
			  await somePromise().then(response => {
			    expect(response).toHaveProperty('data');
			    return response.data;
			  });
			});",
            None,
            None,
        ),
        (
            "it(
			  'test function',
			  () => {
			    return Builder
			      .getPromiseBuilder()
			      .get().build()
			      .then((data) => {
			        expect(data).toEqual('Hi');
			      });
			  }
			);",
            None,
            None,
        ),
        (
            "notATestFunction(
			  'not a test function',
			  () => {
			    Builder
			      .getPromiseBuilder()
			      .get()
			      .build()
			      .then((data) => {
			        expect(data).toEqual('Hi');
			      });
			  }
			);",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promiseOne = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			  });
			  const promiseTwo = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			  });
			  await promiseTwo;
			  await promiseOne;
			});",
            None,
            None,
        ),
        (
            r#"it("it1", () => somePromise.then(() => {
			  expect(someThing).toEqual(true)
			}))"#,
            None,
            None,
        ),
        (r#"it("it1", () => somePromise.then(() => expect(someThing).toEqual(true)))"#, None, None),
        (
            "it('promise test with done', (done) => {
			  const promise = getPromise();
			  promise.then(() => expect(someThing).toEqual(true));
			});",
            None,
            None,
        ),
        (
            "it('name of done param does not matter', (nameDoesNotMatter) => {
			  const promise = getPromise();
			  promise.then(() => expect(someThing).toEqual(true));
			});",
            None,
            None,
        ),
        (
            "it.each([])('name of done param does not matter', (nameDoesNotMatter) => {
			  const promise = getPromise();
			  promise.then(() => expect(someThing).toEqual(true));
			});",
            None,
            None,
        ),
        (
            r"it.each``('name of done param does not matter', ({}, nameDoesNotMatter) => {
			  const promise = getPromise();
			  promise.then(() => expect(someThing).toEqual(true));
			});",
            None,
            None,
        ),
        (
            "test('valid-expect-in-promise', async () => {
			  const text = await fetch('url')
			      .then(res => res.text())
			      .then(text => text);
			  expect(text).toBe('text');
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  }), x = 1;
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let x = 1, somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			  somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			  somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  return somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  {}
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  {
			    await somePromise;
			  }
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  {
			    await somePromise;
			    somePromise = getPromise().then((data) => {
			      expect(data).toEqual('foo');
			    });
			    await somePromise;
			  }
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			  {
			    somePromise = getPromise().then((data) => {
			      expect(data).toEqual('foo');
			    });
			    await somePromise;
			  }
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  somePromise = somePromise.then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  somePromise = somePromise
			    .then((data) => data)
			    .then((data) => data)
			    .then((data) => {
			      expect(data).toEqual('foo');
			    });
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  somePromise = somePromise
			    .then((data) => data)
			    .then((data) => data)
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			  {
			    somePromise = getPromise().then((data) => {
			      expect(data).toEqual('foo');
			    });
			    {
			      await somePromise;
			    }
			  }
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await Promise.all([somePromise]);
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  return Promise.all([somePromise]);
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  return Promise.resolve(somePromise);
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  return Promise.reject(somePromise);
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await Promise.resolve(somePromise);
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await Promise.reject(somePromise);
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const onePromise = something().then(value => {
			    console.log(value);
			  });
			  const twoPromise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return Promise.all([onePromise, twoPromise]);
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const onePromise = something().then(value => {
			    console.log(value);
			  });
			  const twoPromise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return Promise.allSettled([onePromise, twoPromise]);
			});",
            None,
            None,
        ),
    ];

    let pass_vitest = vec![
        ("test('something', () => Promise.resolve().then(() => expect(1).toBe(2)));", None, None),
        ("Promise.resolve().then(() => expect(1).toBe(2))", None, None),
        ("const x = Promise.resolve().then(() => expect(1).toBe(2))", None, None),
        (
            "
			      it('is valid', () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        expect(promise).resolves.toBe(1);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        expect(promise).resolves.not.toBe(2);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        expect(promise).rejects.toBe(1);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        expect(promise).rejects.not.toBe(2);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        expect(await promise).toBeGreaterThan(1);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        expect(await promise).resolves.toBeGreaterThan(1);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        expect(1).toBeGreaterThan(await promise);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        expect.this.that.is(await promise);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        expect(await loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        })).toBeGreaterThan(1);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        expect([await promise]).toHaveLength(1);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        expect([,,await promise,,]).toHaveLength(1);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        expect([[await promise]]).toHaveLength(1);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return number + 1;
			        });

			        logValue(await promise);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        const promise = loadNumber().then(number => {
			          expect(typeof number).toBe('number');

			          return 1;
			        });

			        expect.assertions(await promise);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        await loadNumber().then(number => {
			          expect(typeof number).toBe('number');
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', () => new Promise((done) => {
			        test()
			          .then(() => {
			            expect(someThing).toEqual(true);
			            done();
			          });
			      }));
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', () => {
			        return new Promise(done => {
			          test().then(() => {
			            expect(someThing).toEqual(true);
			            done();
			          });
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('passes', () => {
			        Promise.resolve().then(() => {
			          grabber.grabSomething();
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('passes', async () => {
			        const grabbing = Promise.resolve().then(() => {
			          grabber.grabSomething();
			        });

			        await grabbing;

			        expect(grabber.grabbedItems).toHaveLength(1);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      const myFn = () => {
			        Promise.resolve().then(() => {
			          expect(true).toBe(false);
			        });
			      };
			    ",
            None,
            None,
        ),
        (
            "
			      const myFn = () => {
			        Promise.resolve().then(() => {
			          subject.invokeMethod();
			        });
			      };
			    ",
            None,
            None,
        ),
        (
            "
			      const myFn = () => {
			        Promise.resolve().then(() => {
			          expect(true).toBe(false);
			        });
			      };

			      it('it1', () => {
			        return somePromise.then(() => {
			          expect(someThing).toEqual(true);
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', () => new Promise((done) => {
			        test()
			          .finally(() => {
			            expect(someThing).toEqual(true);
			            done();
			          });
			      }));
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', () => {
			        return somePromise.then(() => {
			          expect(someThing).toEqual(true);
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', () => {
			        return somePromise.finally(() => {
			          expect(someThing).toEqual(true);
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', function() {
			        return somePromise.catch(function() {
			          expect(someThing).toEqual(true);
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      xtest('it1', function() {
			        return somePromise.catch(function() {
			          expect(someThing).toEqual(true);
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', function() {
			        return somePromise.then(function() {
			          doSomeThingButNotExpect();
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', function() {
			        return getSomeThing().getPromise().then(function() {
			          expect(someThing).toEqual(true);
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', function() {
			        return Promise.resolve().then(function() {
			          expect(someThing).toEqual(true);
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', function () {
			        return Promise.resolve().then(function () {
			          /*fulfillment*/
			          expect(someThing).toEqual(true);
			        }, function () {
			          /*rejection*/
			          expect(someThing).toEqual(true);
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', function () {
			        Promise.resolve().then(/*fulfillment*/ function () {
			        }, undefined, /*rejection*/ function () {
			          expect(someThing).toEqual(true)
			        })
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', function () {
			        return Promise.resolve().then(function () {
			          /*fulfillment*/
			        }, function () {
			          /*rejection*/
			          expect(someThing).toEqual(true);
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', function () {
			        return somePromise.then()
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', async () => {
			        await Promise.resolve().then(function () {
			          expect(someThing).toEqual(true)
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', async () => {
			        await somePromise.then(() => {
			          expect(someThing).toEqual(true)
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', async () => {
			        await getSomeThing().getPromise().then(function () {
			          expect(someThing).toEqual(true)
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', () => {
			        return somePromise.then(() => {
			          expect(someThing).toEqual(true);
			        })
			        .then(() => {
			          expect(someThing).toEqual(true);
			        })
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', () => {
			        return somePromise.then(() => {
			          return value;
			        })
			        .then(value => {
			          expect(someThing).toEqual(value);
			        })
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', () => {
			        return somePromise.then(() => {
			          expect(someThing).toEqual(true);
			        })
			        .then(() => {
			          console.log('this is silly');
			        })
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('it1', () => {
			        return somePromise.then(() => {
			          expect(someThing).toEqual(true);
			        })
			        .catch(() => {
			          expect(someThing).toEqual(false);
			        })
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('later return', () => {
			        const promise = something().then(value => {
			          expect(value).toBe('red');
			        });

			        return promise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('later return', async () => {
			        const promise = something().then(value => {
			          expect(value).toBe('red');
			        });

			        await promise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test.only('later return', () => {
			        const promise = something().then(value => {
			          expect(value).toBe('red');
			        });

			        return promise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('that we bailout if destructuring is used', () => {
			        const [promise] = something().then(value => {
			          expect(value).toBe('red');
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('that we bailout if destructuring is used', async () => {
			        const [promise] = await something().then(value => {
			          expect(value).toBe('red');
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('that we bailout if destructuring is used', () => {
			        const [promise] = [
			          something().then(value => {
			            expect(value).toBe('red');
			          })
			        ];
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('that we bailout if destructuring is used', () => {
			        const {promise} = {
			          promise: something().then(value => {
			            expect(value).toBe('red');
			          })
			        };
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('that we bailout in complex cases', () => {
			        promiseSomething({
			          timeout: 500,
			          promise: something().then(value => {
			            expect(value).toBe('red');
			          })
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('shorthand arrow', () =>
			        something().then(value => {
			          expect(() => {
			            value();
			          }).toThrow();
			        })
			      );
			    ",
            None,
            None,
        ),
        (
            "
			      it('crawls for files based on patterns', () => {
			        const promise = nodeCrawl({}).then(data => {
			          expect(childProcess.spawn).lastCalledWith('find');
			        });
			        return promise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is a test', async () => {
			        const value = await somePromise().then(response => {
			          expect(response).toHaveProperty('data');

			          return response.data;
			        });

			        expect(value).toBe('hello world');
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is a test', async () => {
			        return await somePromise().then(response => {
			          expect(response).toHaveProperty('data');

			          return response.data;
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is a test', async () => {
			        return somePromise().then(response => {
			          expect(response).toHaveProperty('data');

			          return response.data;
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('is a test', async () => {
			        await somePromise().then(response => {
			          expect(response).toHaveProperty('data');

			          return response.data;
			        });
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it(
			        'test function',
			        () => {
			          return Builder
			            .getPromiseBuilder()
			            .get().build()
			            .then((data) => {
			              expect(data).toEqual('Hi');
			            });
			        }
			      );
			    ",
            None,
            None,
        ),
        (
            "
			      notATestFunction(
			        'not a test function',
			        () => {
			          Builder
			            .getPromiseBuilder()
			            .get()
			            .build()
			            .then((data) => {
			              expect(data).toEqual('Hi');
			            });
			        }
			      );
			    ",
            None,
            None,
        ),
        (
            "
			      it('is valid', async () => {
			        const promiseOne = loadNumber().then(number => {
			          expect(typeof number).toBe('number');
			        });
			        const promiseTwo = loadNumber().then(number => {
			          expect(typeof number).toBe('number');
			        });

			        await promiseTwo;
			        await promiseOne;
			      });
			    ",
            None,
            None,
        ),
        (
            r#"
			      it("it1", () => somePromise.then(() => {
			        expect(someThing).toEqual(true)
			      }))
			    "#,
            None,
            None,
        ),
        (r#"it("it1", () => somePromise.then(() => expect(someThing).toEqual(true)))"#, None, None),
        (
            "
			      it('promise test with done', (done) => {
			        const promise = getPromise();
			        promise.then(() => expect(someThing).toEqual(true));
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it('name of done param does not matter', (nameDoesNotMatter) => {
			        const promise = getPromise();
			        promise.then(() => expect(someThing).toEqual(true));
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it.each([])('name of done param does not matter', (nameDoesNotMatter) => {
			        const promise = getPromise();
			        promise.then(() => expect(someThing).toEqual(true));
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      it.each``('name of done param does not matter', ({}, nameDoesNotMatter) => {
			        const promise = getPromise();
			        promise.then(() => expect(someThing).toEqual(true));
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('valid-expect-in-promise', async () => {
			        const text = await fetch('url')
			            .then(res => res.text())
			            .then(text => text);

			        expect(text).toBe('text');
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        let somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        }), x = 1;

			        await somePromise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        let x = 1, somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        await somePromise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        let somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        await somePromise;

			        somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        await somePromise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        let somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        await somePromise;

			        somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        return somePromise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        let somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        {}

			        await somePromise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        const somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        {
			          await somePromise;
			        }
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        let somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        {
			          await somePromise;

			          somePromise = getPromise().then((data) => {
			            expect(data).toEqual('foo');
			          });

			          await somePromise;
			        }
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        let somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        await somePromise;

			        {
			          somePromise = getPromise().then((data) => {
			            expect(data).toEqual('foo');
			          });

			          await somePromise;
			        }
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        let somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        somePromise = somePromise.then((data) => {
			          expect(data).toEqual('foo');
			        });

			        await somePromise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        let somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        somePromise = somePromise
			          .then((data) => data)
			          .then((data) => data)
			          .then((data) => {
			            expect(data).toEqual('foo');
			          });

			        await somePromise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        let somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        somePromise = somePromise
			          .then((data) => data)
			          .then((data) => data)

			        await somePromise;
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        let somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        await somePromise;

			        {
			          somePromise = getPromise().then((data) => {
			            expect(data).toEqual('foo');
			          });

			          {
			            await somePromise;
			          }
			        }
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        const somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        await Promise.all([somePromise]);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        const somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        return Promise.all([somePromise]);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        const somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        return Promise.resolve(somePromise);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        const somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        return Promise.reject(somePromise);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        const somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        await Promise.resolve(somePromise);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('promise test', async function () {
			        const somePromise = getPromise().then((data) => {
			          expect(data).toEqual('foo');
			        });

			        await Promise.reject(somePromise);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('later return', async () => {
			        const onePromise = something().then(value => {
			          console.log(value);
			        });
			        const twoPromise = something().then(value => {
			          expect(value).toBe('red');
			        });

			        return Promise.all([onePromise, twoPromise]);
			      });
			    ",
            None,
            None,
        ),
        (
            "
			      test('later return', async () => {
			        const onePromise = something().then(value => {
			          console.log(value);
			        });
			        const twoPromise = something().then(value => {
			          expect(value).toBe('red');
			        });

			        return Promise.allSettled([onePromise, twoPromise]);
			      });
			    ",
            None,
            None,
        ),
    ];

    let fail_jest = vec![
        (
            "const myFn = () => {
			  Promise.resolve().then(() => {
			    expect(true).toBe(false);
			  });
			};
			it('it1', () => {
			  somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  somePromise.finally(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "
			       it('it1', () => {
			         somePromise['then'](() => {
			           expect(someThing).toEqual(true);
			         });
			       });
			      ",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  getSomeThing().getPromise().then(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  Promise.resolve().then(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  somePromise.catch(function() {
			    expect(someThing).toEqual(true)
			  })
			})",
            None,
            None,
        ),
        (
            "xtest('it1', function() {
			  somePromise.catch(function() {
			    expect(someThing).toEqual(true)
			  })
			})",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  somePromise.then(function() {
			    expect(someThing).toEqual(true)
			  })
			})",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  Promise.resolve().then(/*fulfillment*/ function () {
			    expect(someThing).toEqual(true);
			  }, /*rejection*/ function () {
			    expect(someThing).toEqual(true);
			  })
			})",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  Promise.resolve().then(/*fulfillment*/ function () {
			  }, /*rejection*/ function () {
			    expect(someThing).toEqual(true)
			  })
			});",
            None,
            None,
        ),
        (
            "it('test function', () => {
			  Builder.getPromiseBuilder()
			    .get()
			    .build()
			    .then(data => expect(data).toEqual('Hi'));
			});",
            None,
            None,
        ),
        (
            "
			        it('test function', async () => {
			          Builder.getPromiseBuilder()
			            .get()
			            .build()
			            .then(data => expect(data).toEqual('Hi'));
			        });
			      ",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  somePromise.then(() => {
			    doSomeOperation();
			    expect(someThing).toEqual(true);
			  })
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise
			    .then(() => {})
			    .then(() => expect(someThing).toEqual(value))
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise
			    .then(() => expect(someThing).toEqual(value))
			    .then(() => {})
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise.then(() => {
			    return value;
			  })
			  .then(value => {
			    expect(someThing).toEqual(value);
			  })
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  })
			  .then(() => {
			    console.log('this is silly');
			  })
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise.then(() => {
			    // return value;
			  })
			  .then(value => {
			    expect(someThing).toEqual(value);
			  })
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise.then(() => {
			    return value;
			  })
			  .then(value => {
			    expect(someThing).toEqual(value);
			  })
			  return anotherPromise.then(() => expect(x).toBe(y));
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise
			    .then(() => 1)
			    .then(x => x + 1)
			    .catch(() => -1)
			    .then(v => expect(v).toBe(2));
			  return anotherPromise.then(() => expect(x).toBe(y));
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise
			    .then(() => 1)
			    .then(v => expect(v).toBe(2))
			    .then(x => x + 1)
			    .catch(() => -1);
			  return anotherPromise.then(() => expect(x).toBe(y));
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  somePromise.finally(() => {
			    doSomeOperation();
			    expect(someThing).toEqual(true);
			  })
			});",
            None,
            None,
        ),
        (
            r#"test('invalid return', () => {
			  const promise = something().then(value => {
			    const foo = "foo";
			    return expect(value).toBe('red');
			  });
			});"#,
            None,
            None,
        ),
        (
            "fit('it1', () => {
			  somePromise.then(() => {
			    doSomeOperation();
			    expect(someThing).toEqual(true);
			  })
			});",
            None,
            None,
        ),
        (
            "it.skip('it1', () => {
			  somePromise.then(() => {
			    doSomeOperation();
			    expect(someThing).toEqual(true);
			  })
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return;
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return 1;
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return [];
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return Promise.all([anotherPromise]);
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return {};
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return Promise.all([]);
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await 1;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await [];
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await Promise.all([anotherPromise]);
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await {};
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await Promise.all([]);
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  }), x = 1;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const x = 1, promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			});",
            None,
            None,
        ),
        (
            "import { test } from '@jest/globals';
			test('later return', async () => {
			  const x = 1, promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			});",
            None,
            None,
        ),
        (
            "it('promise test', () => {
			  const somePromise = getThatPromise();
			  somePromise.then((data) => {
			    expect(data).toEqual('foo');
			  });
			  expect(somePromise).toBeDefined();
			  return somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', function () {
			  let somePromise = getThatPromise();
			  somePromise.then((data) => {
			    expect(data).toEqual('foo');
			  });
			  expect(somePromise).toBeDefined();
			  return somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  somePromise = null;
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  ({ somePromise } = {})
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  {
			    somePromise = getPromise().then((data) => {
			      expect(data).toEqual('foo');
			    });
			    await somePromise;
			  }
			});",
            None,
            None,
        ),
        (
            "test('that we error on this destructuring', async () => {
			  [promise] = something().then(value => {
			    expect(value).toBe('red');
			  });
			});",
            None,
            None,
        ),
        (
            "test('that we error on this', () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  log(promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(promise).toBeInstanceOf(Promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(anotherPromise).resolves.toBe(1);
			});",
            None,
            None,
        ),
        (
            "import { it as promiseThatThis } from '@jest/globals';
			promiseThatThis('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(anotherPromise).resolves.toBe(1);
			});",
            None,
            None,
        ),
        (
            "promiseThatThis('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(anotherPromise).resolves.toBe(1);
			});",
            None,
            Some(
                serde_json::json!({ "settings": { "jest": { "globalAliases": { "xit": ["promiseThatThis"] } } } }),
            ),
        ),
    ];

    let fail_vitest = vec![
        (
            "
			        const myFn = () => {
			          Promise.resolve().then(() => {
			            expect(true).toBe(false);
			          });
			        };

			        it('it1', () => {
			          somePromise.then(() => {
			            expect(someThing).toEqual(true);
			          });
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('it1', () => {
			          somePromise.then(() => {
			            expect(someThing).toEqual(true);
			          });
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('it1', () => {
			          somePromise.finally(() => {
			            expect(someThing).toEqual(true);
			          });
			        });
			      ",
            None,
            None,
        ),
        (
            "
			       it('it1', () => {
			         somePromise['then'](() => {
			           expect(someThing).toEqual(true);
			         });
			       });
			      ",
            None,
            None,
        ),
        (
            "
			        it('it1', function() {
			          getSomeThing().getPromise().then(function() {
			            expect(someThing).toEqual(true);
			          });
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('it1', function() {
			          Promise.resolve().then(function() {
			            expect(someThing).toEqual(true);
			          });
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('it1', function() {
			          somePromise.catch(function() {
			            expect(someThing).toEqual(true)
			          })
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        xtest('it1', function() {
			          somePromise.catch(function() {
			            expect(someThing).toEqual(true)
			          })
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        it('it1', function() {
			          somePromise.then(function() {
			            expect(someThing).toEqual(true)
			          })
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        it('it1', function () {
			          Promise.resolve().then(/*fulfillment*/ function () {
			            expect(someThing).toEqual(true);
			          }, /*rejection*/ function () {
			            expect(someThing).toEqual(true);
			          })
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        it('it1', function () {
			          Promise.resolve().then(/*fulfillment*/ function () {
			          }, /*rejection*/ function () {
			            expect(someThing).toEqual(true)
			          })
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('test function', () => {
			          Builder.getPromiseBuilder()
			            .get()
			            .build()
			            .then(data => expect(data).toEqual('Hi'));
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('test function', async () => {
			          Builder.getPromiseBuilder()
			            .get()
			            .build()
			            .then(data => expect(data).toEqual('Hi'));
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('it1', () => {
			          somePromise.then(() => {
			            doSomeOperation();
			            expect(someThing).toEqual(true);
			          })
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('is a test', () => {
			          somePromise
			            .then(() => {})
			            .then(() => expect(someThing).toEqual(value))
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('is a test', () => {
			          somePromise
			            .then(() => expect(someThing).toEqual(value))
			            .then(() => {})
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('is a test', () => {
			          somePromise.then(() => {
			            return value;
			          })
			          .then(value => {
			            expect(someThing).toEqual(value);
			          })
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('is a test', () => {
			          somePromise.then(() => {
			            expect(someThing).toEqual(true);
			          })
			          .then(() => {
			            console.log('this is silly');
			          })
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('is a test', () => {
			          somePromise.then(() => {
			            // return value;
			          })
			          .then(value => {
			            expect(someThing).toEqual(value);
			          })
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('is a test', () => {
			          somePromise.then(() => {
			            return value;
			          })
			          .then(value => {
			            expect(someThing).toEqual(value);
			          })

			          return anotherPromise.then(() => expect(x).toBe(y));
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('is a test', () => {
			          somePromise
			            .then(() => 1)
			            .then(x => x + 1)
			            .catch(() => -1)
			            .then(v => expect(v).toBe(2));

			          return anotherPromise.then(() => expect(x).toBe(y));
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('is a test', () => {
			          somePromise
			            .then(() => 1)
			            .then(v => expect(v).toBe(2))
			            .then(x => x + 1)
			            .catch(() => -1);

			          return anotherPromise.then(() => expect(x).toBe(y));
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('it1', () => {
			          somePromise.finally(() => {
			            doSomeOperation();
			            expect(someThing).toEqual(true);
			          })
			        });
			      ",
            None,
            None,
        ),
        (
            r#"
			        test('invalid return', () => {
			          const promise = something().then(value => {
			            const foo = "foo";
			            return expect(value).toBe('red');
			          });
			        });
			      "#,
            None,
            None,
        ),
        (
            "
			        fit('it1', () => {
			          somePromise.then(() => {
			            doSomeOperation();
			            expect(someThing).toEqual(true);
			          })
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it.skip('it1', () => {
			          somePromise.then(() => {
			            doSomeOperation();
			            expect(someThing).toEqual(true);
			          })
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          promise;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          return;

			          await promise;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          return 1;

			          await promise;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          return [];

			          await promise;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          return Promise.all([anotherPromise]);

			          await promise;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          return {};

			          await promise;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          return Promise.all([]);

			          await promise;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          await 1;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          await [];
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          await Promise.all([anotherPromise]);
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          await {};
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          await Promise.all([]);
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          }), x = 1;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('later return', async () => {
			          const x = 1, promise = something().then(value => {
			            expect(value).toBe('red');
			          });
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        import { test } from 'vitest';

			        test('later return', async () => {
			          const x = 1, promise = something().then(value => {
			            expect(value).toBe('red');
			          });
			        });
			      ",
            None,
            None,
        ), // {  "parserOptions": { "sourceType": "module" },  },
        (
            "
			        it('promise test', () => {
			          const somePromise = getThatPromise();
			          somePromise.then((data) => {
			            expect(data).toEqual('foo');
			          });
			          expect(somePromise).toBeDefined();
			          return somePromise;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('promise test', function () {
			          let somePromise = getThatPromise();
			          somePromise.then((data) => {
			            expect(data).toEqual('foo');
			          });
			          expect(somePromise).toBeDefined();
			          return somePromise;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('promise test', async function () {
			          let somePromise = getPromise().then((data) => {
			            expect(data).toEqual('foo');
			          });

			          somePromise = null;

			          await somePromise;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('promise test', async function () {
			          let somePromise = getPromise().then((data) => {
			            expect(data).toEqual('foo');
			          });

			          somePromise = getPromise().then((data) => {
			            expect(data).toEqual('foo');
			          });

			          await somePromise;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('promise test', async function () {
			          let somePromise = getPromise().then((data) => {
			            expect(data).toEqual('foo');
			          });

			          ({ somePromise } = {})
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('promise test', async function () {
			          let somePromise = getPromise().then((data) => {
			            expect(data).toEqual('foo');
			          });

			          {
			            somePromise = getPromise().then((data) => {
			              expect(data).toEqual('foo');
			            });

			            await somePromise;
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('that we error on this destructuring', async () => {
			          [promise] = something().then(value => {
			            expect(value).toBe('red');
			          });
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        test('that we error on this', () => {
			          const promise = something().then(value => {
			            expect(value).toBe('red');
			          });

			          log(promise);
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('is valid', async () => {
			          const promise = loadNumber().then(number => {
			            expect(typeof number).toBe('number');

			            return number + 1;
			          });

			          expect(promise).toBeInstanceOf(Promise);
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        it('is valid', async () => {
			          const promise = loadNumber().then(number => {
			            expect(typeof number).toBe('number');

			            return number + 1;
			          });

			          expect(anotherPromise).resolves.toBe(1);
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        import { it as promiseThatThis } from 'vitest';

			        promiseThatThis('is valid', async () => {
			          const promise = loadNumber().then(number => {
			            expect(typeof number).toBe('number');

			            return number + 1;
			          });

			          expect(anotherPromise).resolves.toBe(1);
			        });
			      ",
            None,
            None,
        ), // { "parserOptions": { "sourceType": "module" } },
        (
            "
			        promiseThatThis('is valid', async () => {
			          const promise = loadNumber().then(number => {
			            expect(typeof number).toBe('number');

			            return number + 1;
			          });

			          expect(anotherPromise).resolves.toBe(1);
			        });
			      ",
            None,
            Some(
                serde_json::json!({ "settings": { "vitest": { "globalAliases": { "xit": ["promiseThatThis"] } } } }),
            ),
        ),
    ];

    // concat the two
    let pass = [&pass_jest[..], &pass_vitest[..]].concat();
    let fail = [&fail_jest[..], &fail_vitest[..]].concat();

    Tester::new(ValidExpectInPromise::NAME, ValidExpectInPromise::PLUGIN, pass, fail)
        .test_and_snapshot();
}
