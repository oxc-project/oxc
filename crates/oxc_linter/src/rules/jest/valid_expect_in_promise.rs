use oxc_allocator::Vec as OxcVec;
use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, AssignmentTarget, BindingPatternKind, CallExpression,
        Expression, IdentifierReference, Statement, VariableDeclarator,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{Atom, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        get_node_name, is_type_of_jest_fn_call, parse_jest_fn_call, JestFnKind, JestGeneralFnKind,
        ParsedGeneralJestFnCall, ParsedJestFnCallNew, PossibleJestNode,
    },
};

fn expect_in_floating_promise(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require promises that have expectations in their chain to be valid")
        .with_help("This promise should either be returned or awaited to ensure the expects in its chain are called")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct ValidExpectInPromise;

declare_oxc_lint!(
    /// ### What it does
    /// This rule flags any promises within the body of a test that include expectations
    /// that have either not been returned or awaited.
    ///
    /// ### Example
    /// ```javascript
    /// // invalid
    /// it('promises a person', () => {
    ///     api.getPersonByName('bob').then(person => {
    ///         expect(person).toHaveProperty('name', 'Bob');
    ///     });
    /// });
    ///
    /// it('promises a counted person', () => {
    ///     const promise = api.getPersonByName('bob').then(person => {
    ///         expect(person).toHaveProperty('name', 'Bob');
    ///     });
    ///     promise.then(() => {
    ///         expect(analytics.gottenPeopleCount).toBe(1);
    ///     });
    /// });
    ///
    /// it('promises multiple people', () => {
    ///     const firstPromise = api.getPersonByName('bob').then(person => {
    ///         expect(person).toHaveProperty('name', 'Bob');
    ///     });
    ///     const secondPromise = api.getPersonByName('alice').then(person => {
    ///         expect(person).toHaveProperty('name', 'Alice');
    ///     });
    ///     return Promise.any([firstPromise, secondPromise]);
    /// });
    ///
    /// // valid
    /// it('promises a person', async () => {
    ///     await api.getPersonByName('bob').then(person => {
    ///         expect(person).toHaveProperty('name', 'Bob');
    ///     });
    /// });
    ///
    /// it('promises a counted person', () => {
    ///     let promise = api.getPersonByName('bob').then(person => {
    ///         expect(person).toHaveProperty('name', 'Bob');
    ///     });
    ///
    ///     promise = promise.then(() => {
    ///         expect(analytics.gottenPeopleCount).toBe(1);
    ///     });
    ///
    ///     return promise;
    /// });
    ///
    /// it('promises multiple people', () => {
    ///     const firstPromise = api.getPersonByName('bob').then(person => {
    ///         expect(person).toHaveProperty('name', 'Bob');
    ///     });
    ///     const secondPromise = api.getPersonByName('alice').then(person => {
    ///         expect(person).toHaveProperty('name', 'Alice');
    ///     });
    ///
    ///     return Promise.allSettled([firstPromise, secondPromise]);
    /// });
    /// ```
    ///
    ValidExpectInPromise,
    correctness
);

impl Rule for ValidExpectInPromise {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if Self::is_test_case_call_with_callback_arg(
            call_expr,
            &PossibleJestNode { node, original: None },
            ctx,
        ) {
            return;
        }

        if Self::is_promise_chain_call(call_expr)
            || !is_type_of_jest_fn_call(
                call_expr,
                &PossibleJestNode { node, original: None },
                ctx,
                &[JestFnKind::Expect],
            )
        {
            return;
        }

        let Some(parent) = Self::find_top_most_call_expr(node, ctx) else {
            return;
        };

        if !Self::is_directly_within_test_case_call(parent, ctx) {
            return;
        }

        let should_report = match parent.kind() {
            AstKind::VariableDeclarator(var_decl) => !Self::is_variable_awaited_or_returned(
                var_decl,
                &PossibleJestNode { node, original: None },
                parent,
                ctx,
            ),
            AstKind::AssignmentExpression(assign_expr) => {
                let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign_expr.left else {
                    return;
                };
                let Some(statements) = Self::find_first_block_body_up(node, ctx) else {
                    return;
                };
                !Self::is_value_awaited_or_returned(
                    ident,
                    &PossibleJestNode { node, original: None },
                    statements,
                    ctx,
                )
            }
            _ => true,
        };

        if should_report {
            ctx.diagnostic(expect_in_floating_promise(call_expr.span));
        }
    }
}

impl ValidExpectInPromise {
    fn is_identifier_with_name(expr: &Expression, name: &Atom) -> bool {
        let Some(ident) = expr.get_identifier_reference() else {
            return false;
        };
        ident.name == name
    }

    fn is_directly_within_test_case_call(node: &AstNode, ctx: &LintContext) -> bool {
        let mut parent = node;

        loop {
            if matches!(parent.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)) {
                let Some(grandparent) = ctx.nodes().parent_node(parent.id()) else {
                    break;
                };
                let AstKind::CallExpression(call_expr) = grandparent.kind() else {
                    break;
                };

                return is_type_of_jest_fn_call(
                    call_expr,
                    &PossibleJestNode { node: grandparent, original: None },
                    ctx,
                    &[JestFnKind::General(JestGeneralFnKind::Test)],
                );
            }

            let Some(grandparent) = ctx.nodes().parent_node(parent.id()) else {
                break;
            };

            parent = grandparent;
        }

        false
    }

    fn is_test_case_call_with_callback_arg<'a>(
        call_expr: &'a CallExpression<'a>,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        ctx: &LintContext<'a>,
    ) -> bool {
        let Some(ParsedJestFnCallNew::GeneralJestFnCall(jest_fn_call)) =
            parse_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return false;
        };
        let ParsedGeneralJestFnCall { kind, members, .. } = jest_fn_call;
        let is_jest_each = members.iter().any(|member| member.is_name_equal("each"));

        if !matches!(kind, JestFnKind::General(JestGeneralFnKind::Test)) {
            return false;
        }

        if is_jest_each && !matches!(call_expr.callee, Expression::TaggedTemplateExpression(_)) {
            return true;
        }

        let Some(first_arg) = call_expr.arguments.get(1) else {
            return false;
        };

        matches!(first_arg, Argument::ArrayExpression(_) | Argument::FunctionExpression(_))
            && call_expr.arguments.len() == 1 + usize::from(is_jest_each)
    }

    fn is_promise_chain_call(call_expr: &CallExpression) -> bool {
        if call_expr.arguments.len() == 0 {
            return false;
        }

        let Some(mem_expr) = call_expr.callee.as_member_expression() else {
            return false;
        };

        let Some(property_name) = mem_expr.static_property_name() else {
            return false;
        };

        if property_name.eq("then") {
            return call_expr.arguments.len() < 3;
        } else if property_name.eq("catch") || property_name.eq("finally") {
            return call_expr.arguments.len() < 2;
        }

        false
    }

    fn is_promise_method_that_uses_value(
        ident: &IdentifierReference,
        expr: Option<&Expression>,
    ) -> bool {
        let name = &ident.name;
        let Some(expr) = expr else {
            return false;
        };

        if let Expression::CallExpression(call_expr) = expr {
            if call_expr.arguments.len() > 0 {
                let node_name = get_node_name(expr);

                if ["Promise.all", "Promise.allSettled"].contains(&node_name.as_str()) {
                    if let Some(first_arg) = call_expr.arguments.first() {
                        if let Expression::ArrayExpression(arr_expr) = first_arg.to_expression() {
                            if arr_expr
                                .elements
                                .iter()
                                .any(|ele| Self::is_identifier_with_name(ele.to_expression(), name))
                            {
                                return true;
                            }
                        }
                    }
                }

                if ["Promise.resolve", "Promise.reject"].contains(&node_name.as_str())
                    && call_expr.arguments.len() == 1
                {
                    if let Some(first_arg) = call_expr.arguments.first() {
                        return Self::is_identifier_with_name(first_arg.to_expression(), name);
                    }
                }
            }
        }

        Self::is_identifier_with_name(expr, name)
    }

    fn is_variable_awaited_or_returned<'a>(
        var_decl: &VariableDeclarator,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        let BindingPatternKind::BindingIdentifier(ident) = &var_decl.id.kind else {
            return true;
        };
        let Some(statements) = Self::find_first_block_body_up(node, ctx) else {
            return false;
        };
        let ident_reference = IdentifierReference::new(ident.span, Atom::from(ident.name.as_str()));

        Self::is_value_awaited_or_returned(&ident_reference, possible_jest_node, statements, ctx)
    }

    fn is_value_awaited_or_returned<'a>(
        ident: &IdentifierReference,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        statements: &'a OxcVec<Statement<'a>>,
        ctx: &LintContext<'a>,
    ) -> bool {
        let name = &ident.name;

        for stmt in statements {
            if let Statement::ReturnStatement(return_stmt) = stmt {
                if let Some(argument) = &return_stmt.argument {
                    return Self::is_promise_method_that_uses_value(ident, Some(argument));
                }
            }

            if let Statement::ExpressionStatement(expr_stmt) = stmt {
                if let Expression::CallExpression(call_expr) = &expr_stmt.expression {
                    if Self::is_value_awaited_in_arguments(name, call_expr) {
                        return true;
                    }

                    let left_most_call = Self::get_left_most_call_expression(call_expr);
                    let Some(ParsedJestFnCallNew::GeneralJestFnCall(jest_fn_call)) =
                        parse_jest_fn_call(call_expr, possible_jest_node, ctx)
                    else {
                        return false;
                    };

                    if matches!(jest_fn_call.kind, JestFnKind::Expect)
                        && left_most_call.arguments.len() > 0
                    {
                        if let Some(Argument::Identifier(ident)) = left_most_call.arguments.first()
                        {
                            if ident.name == name
                                && jest_fn_call.members.iter().any(|m| m.is_name_equal("each"))
                            {
                                return true;
                            }
                        }
                    }
                }

                if let Expression::AwaitExpression(await_expr) = &expr_stmt.expression {
                    if Self::is_promise_method_that_uses_value(ident, Some(&await_expr.argument)) {
                        return true;
                    }
                }

                if let Expression::AssignmentExpression(assign_expr) = &expr_stmt.expression {
                    if matches!(assign_expr.left, AssignmentTarget::AssignmentTargetIdentifier(_))
                        && get_node_name(&assign_expr.right).starts_with(&(name.to_string() + "."))
                    {
                        if let Expression::CallExpression(call_expr) = &assign_expr.right {
                            if Self::is_promise_chain_call(call_expr) {
                                continue;
                            }
                        }
                    }

                    break;
                }
            }

            if let Statement::BlockStatement(block_stmt) = stmt {
                return Self::is_value_awaited_or_returned(
                    ident,
                    possible_jest_node,
                    &block_stmt.body,
                    ctx,
                );
            }
        }

        false
    }

    fn is_value_awaited_in_arguments(name: &Atom, call_expr: &CallExpression) -> bool {
        let mut node = call_expr;

        loop {
            if let Expression::CallExpression(call_expr) = &node.callee {
                if Self::is_value_awaited_in_argument_elements(name, &node.arguments) {
                    return true;
                }

                node = call_expr;
            }

            if matches!(
                node.callee,
                Expression::ComputedMemberExpression(_)
                    | Expression::StaticMemberExpression(_)
                    | Expression::PrivateFieldExpression(_)
            ) {
                break;
            }
        }

        false
    }

    fn is_value_awaited_in_argument_elements<'a>(
        name: &Atom,
        elements: &OxcVec<'a, Argument<'a>>,
    ) -> bool {
        for ele in elements {
            if let Argument::AwaitExpression(await_expr) = ele {
                if Self::is_identifier_with_name(&await_expr.argument, name) {
                    return true;
                }
            }

            if let Argument::ArrayExpression(arr_expr) = ele {
                if Self::is_value_awaited_in_array_elements(name, &arr_expr.elements) {
                    return true;
                }
            }
        }

        false
    }

    fn is_value_awaited_in_array_elements<'a>(
        name: &Atom,
        elements: &OxcVec<'a, ArrayExpressionElement<'a>>,
    ) -> bool {
        for ele in elements {
            if let ArrayExpressionElement::AwaitExpression(await_expr) = ele {
                if Self::is_identifier_with_name(&await_expr.argument, name) {
                    return true;
                }
            }

            if let ArrayExpressionElement::ArrayExpression(arr_expr) = ele {
                if Self::is_value_awaited_in_array_elements(name, &arr_expr.elements) {
                    return true;
                }
            }
        }

        false
    }

    fn find_top_most_call_expr<'a, 'b>(
        node: &'b AstNode<'a>,
        ctx: &'b LintContext<'a>,
    ) -> Option<&'b AstNode<'a>> {
        let mut current = Some(node);
        let mut parent = ctx.nodes().parent_node(node.id());

        loop {
            let Some(parent_inner) = parent else {
                break;
            };

            if matches!(parent_inner.kind(), AstKind::CallExpression(_)) {
                current = parent;
                parent = ctx.nodes().parent_node(parent_inner.id());
                continue;
            }
            if !matches!(parent_inner.kind(), AstKind::MemberExpression(_)) {
                break;
            }

            parent = ctx.nodes().parent_node(parent_inner.id());
        }

        current
    }

    fn find_first_block_body_up<'a>(
        node: &AstNode<'a>,
        ctx: &LintContext,
    ) -> Option<&'a OxcVec<'a, Statement<'a>>> {
        let mut current = node;
        loop {
            if let AstKind::BlockStatement(block_stmt) = node.kind() {
                return Some(&block_stmt.body);
            }
            let parent = ctx.nodes().parent_node(current.id())?;
            current = parent;
        }
    }

    fn get_left_most_call_expression<'a>(expr: &'a CallExpression) -> &'a CallExpression<'a> {
        let mut left_most_call_expr = expr;
        let mut node = &expr.callee;

        loop {
            if let Expression::CallExpression(call) = &node {
                left_most_call_expr = call;
                node = &call.callee;
            }
            if !matches!(
                node,
                Expression::ComputedMemberExpression(_)
                    | Expression::StaticMemberExpression(_)
                    | Expression::PrivateFieldExpression(_)
            ) {
                break;
            }
        }

        left_most_call_expr
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        ("test('something', () => Promise.resolve().then(() => expect(1).toBe(2)));", None),
        ("Promise.resolve().then(() => expect(1).toBe(2))", None),
        ("const x = Promise.resolve().then(() => expect(1).toBe(2))", None),
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
        ),
        (
            "
                it('it1', function () {
                    return somePromise.then()
                });
            ",
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
        ),
        (
            "
                it(\"it1\", () => somePromise.then(() => {
                    expect(someThing).toEqual(true)
                }))
            ",
            None,
        ),
        ("it(\"it1\", () => somePromise.then(() => expect(someThing).toEqual(true)))", None),
        (
            "
                it('promise test with done', (done) => {
                    const promise = getPromise();
                    promise.then(() => expect(someThing).toEqual(true));
                });
            ",
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
        ),
        (
            "
                it.each([])('name of done param does not matter', (nameDoesNotMatter) => {
                    const promise = getPromise();
                    promise.then(() => expect(someThing).toEqual(true));
                });
            ",
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
        ),
    ];

    let fail = vec![
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
        ),
        (
            "
                test('invalid return', () => {
                    const promise = something().then(value => {
                        const foo = \"foo\";
                        return expect(value).toBe('red');
                    });
                });
            ",
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
        ),
        (
            "
                import { test } from '@jest/globals';

                test('later return', async () => {
                    const x = 1, promise = something().then(value => {
                        expect(value).toBe('red');
                    });
                });
            ",
            None,
        ),
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
        ),
        (
            "
                import { it as promiseThatThis } from '@jest/globals';

                promiseThatThis('is valid', async () => {
                    const promise = loadNumber().then(number => {
                        expect(typeof number).toBe('number');

                        return number + 1;
                    });

                    expect(anotherPromise).resolves.toBe(1);
                });
            ",
            None,
        ),
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
        ),
    ];

    Tester::new(ValidExpectInPromise::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
