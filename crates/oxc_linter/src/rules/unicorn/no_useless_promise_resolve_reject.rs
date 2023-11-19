use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::outermost_paren_parent, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum NoUselessPromiseResolveRejectDiagnostic {
    #[error("eslint-plugin-unicorn(no-useless-promise-resolve-reject): Prefer `{1} value` over `{1} Promise.resolve(value)`.")]
    #[diagnostic(severity(warning), help("Wrapping the return value in `Promise.Resolve` is needlessly verbose. All return values in async functions are already wrapped in a `Promise`."))]
    Resolve(#[label] Span, &'static str),
    #[error("eslint-plugin-unicorn(no-useless-promise-resolve-reject): Prefer `throw error` over `{1} Promise.reject(error)`.")]
    #[diagnostic(severity(warning), help("Wrapping the error in `Promise.reject` is needlessly verbose. All errors thrown in async functions are already wrapped in a `Promise`."))]
    Reject(#[label] Span, &'static str),
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessPromiseResolveReject;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows returning values wrapped in `Promise.resolve` or `Promise.reject` in an async function or a `Promise#then`/`catch`/`finally` callback.
    ///
    /// ### Why is this bad?
    ///
    /// Wrapping a return value in `Promise.resolve` in an async function or a `Promise#then`/`catch`/`finally` callback is unnecessary as all return values in async functions and promise callback functions are already wrapped in a `Promise`. Similarly, returning an error wrapped in `Promise.reject` is equivalent to simply `throw`ing the error. This is the same for `yield`ing in async generators as well.
    ///
    /// ### Example
    /// ```javascript
    /// // bad
    /// async () => Promise.resolve(bar);
    ///
    /// // good
    /// async () => bar;
    /// ```
    NoUselessPromiseResolveReject,
    pedantic
);

impl Rule for NoUselessPromiseResolveReject {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        let Some(member_expr) = call_expr.callee.get_member_expr() else { return };

        if !member_expr.object().is_specific_id("Promise") {
            return;
        }

        let MemberExpression::StaticMemberExpression(static_member_expr) = member_expr else {
            return;
        };

        if !matches!(static_member_expr.property.name.as_str(), "resolve" | "reject") {
            return;
        }

        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        let mut is_yield = false;

        if let AstKind::YieldExpression(yield_expr) = parent.kind() {
            is_yield = true;
            if yield_expr.delegate {
                return;
            }
        }

        let Some((is_async, function_node)) = get_function_like_node(node, ctx) else { return };

        if !(is_async || is_promise_callback(function_node, ctx)) {
            return;
        }

        match static_member_expr.property.name.as_str() {
            "resolve" => {
                ctx.diagnostic(NoUselessPromiseResolveRejectDiagnostic::Resolve(
                    node.kind().span(),
                    if is_yield { "yield" } else { "return" },
                ));
            }
            "reject" => {
                ctx.diagnostic(NoUselessPromiseResolveRejectDiagnostic::Reject(
                    node.kind().span(),
                    if is_yield { "yield" } else { "return" },
                ));
            }
            _ => unreachable!(),
        }
    }
}

fn get_function_like_node<'a, 'b>(
    node: &'a AstNode<'b>,
    ctx: &'a LintContext<'b>,
) -> Option<(bool, &'a AstNode<'b>)> {
    let mut parent = node;

    let fnx = loop {
        if let Some(grand_parent) = ctx.nodes().parent_node(parent.id()) {
            parent = grand_parent;
            if parent.kind().is_function_like() {
                break parent;
            }
        } else {
            return None;
        }
    };

    match fnx.kind() {
        AstKind::ArrowExpression(arrow_expr) => Some((arrow_expr.r#async, parent)),
        AstKind::Function(func) => Some((func.r#async, parent)),
        _ => None,
    }
}

fn is_promise_callback<'a, 'b>(node: &'a AstNode<'b>, ctx: &'a LintContext<'b>) -> bool {
    let Some(parent) = outermost_paren_parent(node, ctx) else {
        return false;
    };

    let Some(parent) = outermost_paren_parent(parent, ctx) else {
        return false;
    };

    let AstKind::CallExpression(call_expr) = parent.kind() else { return false };

    let Some(member_expr) = call_expr.callee.get_member_expr() else { return false };

    if member_expr.is_computed() {
        return false;
    }
    let Some(static_prop_name) = member_expr.static_property_name() else { return false };

    if call_expr.arguments.len() == 1 && matches!(static_prop_name, "then" | "catch" | "finally") {
        return true;
    }

    if call_expr.arguments.len() == 2 && matches!(static_prop_name, "then") {
        if call_expr.arguments[0].is_spread() {
            return false;
        }
        return true;
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Async functions returning normal values/throwing values
        r"async () => bar;",
        r"
            async () => {
                return bar;
            };
        ",
        r"
            async function foo() {
                return bar;
            }
        ",
        r"
            (async function() {
                return bar;
            });
        ",
        r"
            async () => {
                throw bar;
            };
        ",
        r"
            async function foo() {
                throw bar;
            }
        ",
        r"
            (async function() {
                throw bar;
            });
        ",
        // Async functions returning normal values/throwing values
        // Sync function returning Promise.resolve/reject
        r"() => Promise.resolve(bar);",
        r"
            () => {
                return Promise.resolve(bar);
            };
        ",
        r"
            function foo() {
                return Promise.resolve(bar);
            };
        ",
        r"
            (function() {
                return Promise.resolve(bar);
            });
            ",
        r"() => Promise.reject(bar);",
        r"
            () => {
                return Promise.reject(bar);
            };
        ",
        r"
            function foo() {
                return Promise.reject(bar);
            };
        ",
        r"
            (function() {
                return Promise.reject(bar);
            });
        ",
        // Sync generator yielding Promise.resolve/reject
        r"
            function * foo() {
                yield Promise.$resolve(bar);
            }
        ",
        r"
            (function * () {
                yield Promise.$resolve(bar);
            })
        ",
        r"
            function * foo() {
                yield Promise.reject(bar);
            }
        ",
        r"
            (function * () {
                yield Promise.reject(bar);
            })
        ",
        // Sync function nested in async function
        r"
            async function foo() {
                function bar() {
                    return Promise.resolve(baz);
                }
            }
        ",
        // Delegate yield expressions
        r"
            async function * foo() {
				yield* Promise.resolve(bar);
			}
        ",
        r"
            async function * foo() {
				yield* Promise.reject(bar);
			}
        ",
        // Promise#then/catch/finally
        r"promise.then(() => foo).catch(() => bar).finally(() => baz)",
        r"promise.then(() => foo, () => bar).finally(() => baz)",
        r"promise.then(x, y, () => Promise.resolve(foo))",
        r"promise.catch(x, () => Promise.resolve(foo))",
        r"promise.finally(x, () => Promise.resolve(foo))",
        r"promise[then](() => Promise.resolve(foo))",
    ];

    let fail = vec![
        r"
            const main = async foo => {
                if (foo > 4) {
                    return Promise.reject(new Error('🤪'));
                }
                return Promise.resolve(result);
            };
        ",
        // Async function returning Promise.resolve
        r"async () => Promise.resolve(bar);",
        r"
            async () => {
                return Promise.resolve(bar);
            };
        ",
        r"
            async function foo() {
                return Promise.resolve(bar);
            }
        ",
        r"
			(async function() {
				return Promise.resolve(bar);
			});
		",
        r"
			async function * foo() {
				return Promise.resolve(bar);
			}
        ",
        r"
		    (async function*() {
		    	return Promise.resolve(bar);
		    });
        ",
        // Async function returning Promise.reject
        r"async () => Promise.reject(bar);",
        r"
			async () => {
				return Promise.reject(bar);
			};
		",
        r"
			async function foo() {
				return Promise.reject(bar);
		    }   
		",
        r"
			(async function() {
				return Promise.reject(bar);
			});
		",
        r"
			async function * foo() {
				return Promise.reject(bar);
			}
		",
        r"
			(async function*() {
				return Promise.reject(bar);
			});
		",
        // Async generator yielding Promise.resolve
        r"
				async function * foo() {
					yield Promise.resolve(bar);
				}
			",
        r"
				(async function * () {
					yield Promise.resolve(bar);
				});
				",
        // Async generator yielding Promise.reject
        r"
				async function * foo() {
					yield Promise.reject(bar);
				}
				",
        r"
				(async function * () {
					yield Promise.reject(bar);
				});
				",
        r"async () => Promise.resolve();",
        r"
				async function foo() {
					return Promise.resolve();
				}
				",
        r"async () => Promise.reject();",
        r"
				async function foo() {
					return Promise.reject();
				}
				",
        r"
				async function * foo() {
					yield Promise.resolve();
				}
				",
        // Multiple arguments
        r"async () => Promise.resolve(bar, baz);",
        r"async () => Promise.reject(bar, baz);",
        // Sequence expressions
        r"async
				async function * foo() {
					yield Promise.resolve((bar, baz));
				}
		",
        r"async () => Promise.resolve((bar, baz))",
        // Arrow function returning an object
        r"async () => Promise.resolve({})",
        // Try statements
        r"
				async function foo() {
					try {
						return Promise.resolve(1);
					} catch {}
				}
		",
        r"
				async function foo() {
					try {
						return Promise.reject(1);
					} catch {}
				}
				",
        // Spread arguments
        r"async () => Promise.resolve(...bar);",
        r"async () => Promise.reject(...bar);",
        // Yield not in an ExpressionStatement
        r"#
        async function * foo() {
            const baz = yield Promise.resolve(bar);
        }
        #",
        r"
        async function * foo() {
            const baz = yield Promise.reject(bar);
        }
        ",
        // Parenthesized Promise.resolve/reject
        r"async () => (Promise.resolve(bar));",
        r"async () => (Promise.reject(bar));",
        r"async () => ((Promise.reject(bar)));",
        r"
            async function * foo() {
     			(yield Promise.reject(bar));
     		}
        ",
        r"
            async function * foo() {
     			((yield Promise.reject(bar)));
     		}
        ",
        r"promise.then(() => Promise.resolve(bar))",
        r"promise.then(() => { return Promise.resolve(bar); })",
        r"promise.then(async () => Promise.reject(bar))",
        r"promise.then(async () => { return Promise.reject(bar); })",
        r"promise.catch(() => Promise.resolve(bar))",
        r"promise.catch(() => { return Promise.resolve(bar); })",
        r"promise.catch(async () => Promise.reject(bar))",
        r"promise.catch(async () => { return Promise.reject(bar); })",
        r"promise.finally(() => Promise.resolve(bar))",
        r"promise.finally(() => { return Promise.resolve(bar); })",
        r"promise.finally(async () => Promise.reject(bar))",
        r"promise.finally(async () => { return Promise.reject(bar); })",
        r"promise.then(() => {}, () => Promise.resolve(bar))",
        r"promise.then(() => Promise.resolve(bar), () => Promise.resolve(baz))",
        r"promise.then(() => {}, () => { return Promise.resolve(bar); })",
        r"promise.then(() => {}, async () => Promise.reject(bar))",
        r"promise.then(() => {}, async () => { return Promise.reject(bar); })",
    ];

    Tester::new_without_config(NoUselessPromiseResolveReject::NAME, pass, fail).test_and_snapshot();
}
