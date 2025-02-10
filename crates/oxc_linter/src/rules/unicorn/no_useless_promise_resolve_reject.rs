use oxc_ast::{
    ast::{Argument, CallExpression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::outermost_paren_parent,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

fn resolve(span: Span, preferred: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `{preferred} value` over `{preferred} Promise.resolve(value)`."))
        .with_help("Wrapping the return value in `Promise.Resolve` is needlessly verbose. All return values in async functions are already wrapped in a `Promise`.")
        .with_label(span)
}

fn reject(span: Span, preferred: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `throw error` over `{preferred} Promise.reject(error)`."))
        .with_help("Wrapping the error in `Promise.reject` is needlessly verbose. All errors thrown in async functions are already wrapped in a `Promise`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessPromiseResolveReject(Box<NoUselessPromiseResolveRejectOptions>);

#[derive(Debug, Default, Clone)]
pub struct NoUselessPromiseResolveRejectOptions {
    pub allow_reject: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows returning values wrapped in `Promise.resolve` or `Promise.reject` in an async function or a `Promise#then`/`catch`/`finally` callback.
    ///
    /// ### Why is this bad?
    ///
    /// Wrapping a return value in `Promise.resolve` in an async function or a `Promise#then`/`catch`/`finally` callback is unnecessary as all return values in async functions and promise callback functions are already wrapped in a `Promise`. Similarly, returning an error wrapped in `Promise.reject` is equivalent to simply `throw`ing the error. This is the same for `yield`ing in async generators as well.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// async () => Promise.resolve(bar);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// async () => bar;
    /// ```
    NoUselessPromiseResolveReject,
    unicorn,
    pedantic,
    fix
);

impl Rule for NoUselessPromiseResolveReject {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);

        let allow_reject = config
            .and_then(|c| c.get("allowReject"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default();

        Self(Box::new(NoUselessPromiseResolveRejectOptions { allow_reject }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        if !member_expr.object().is_specific_id("Promise") {
            return;
        }

        let MemberExpression::StaticMemberExpression(static_member_expr) = member_expr else {
            return;
        };

        if !matches!(static_member_expr.property.name.as_str(), "resolve" | "reject") {
            return;
        }

        let Some(parent) = outermost_paren_parent(node, ctx) else {
            return;
        };

        let mut is_yield = false;

        match parent.kind() {
            AstKind::ArrowFunctionExpression(_) | AstKind::ReturnStatement(_) => {}
            AstKind::ExpressionStatement(_) => {
                let Some(grand_parent) = outermost_paren_parent(parent, ctx) else {
                    return;
                };
                let AstKind::FunctionBody(function_body) = grand_parent.kind() else { return };

                if function_body.statements.len() != 1 {
                    return;
                }
            }
            AstKind::YieldExpression(yield_expr) => {
                is_yield = true;
                if yield_expr.delegate {
                    return;
                }
            }
            _ => return,
        }

        let Some((is_async, function_node, is_in_try_statement)) =
            get_function_like_node(node, ctx)
        else {
            return;
        };

        if !(is_async || is_promise_callback(function_node, ctx)) {
            return;
        }

        match static_member_expr.property.name.as_str() {
            "resolve" => {
                ctx.diagnostic_with_fix(
                    resolve(node.kind().span(), if is_yield { "yield" } else { "return" }),
                    |fixer| {
                        generate_fix(
                            call_expr,
                            false,
                            is_yield,
                            is_in_try_statement,
                            fixer,
                            ctx,
                            node,
                        )
                    },
                );
            }
            "reject" => {
                if self.0.allow_reject {
                    return;
                }
                ctx.diagnostic_with_fix(
                    reject(node.kind().span(), if is_yield { "yield" } else { "return" }),
                    |fixer| {
                        generate_fix(
                            call_expr,
                            true,
                            is_yield,
                            is_in_try_statement,
                            fixer,
                            ctx,
                            node,
                        )
                    },
                );
            }
            _ => unreachable!(),
        }
    }
}

fn get_function_like_node<'a, 'b>(
    node: &'a AstNode<'b>,
    ctx: &'a LintContext<'b>,
) -> Option<(bool, &'a AstNode<'b>, bool)> {
    let mut parent = node;
    let mut is_in_try_statement = false;

    let fnx = loop {
        if let Some(grand_parent) = ctx.nodes().parent_node(parent.id()) {
            parent = grand_parent;
            if parent.kind().is_function_like() {
                break parent;
            }
            if matches!(parent.kind(), AstKind::TryStatement(_)) {
                is_in_try_statement = true;
            }
        } else {
            return None;
        }
    };

    match fnx.kind() {
        AstKind::ArrowFunctionExpression(arrow_expr) => {
            Some((arrow_expr.r#async, parent, is_in_try_statement))
        }
        AstKind::Function(func) => Some((func.r#async, parent, is_in_try_statement)),
        _ => None,
    }
}

fn is_promise_callback<'a, 'b>(node: &'a AstNode<'b>, ctx: &'a LintContext<'b>) -> bool {
    let function_node = traverse_bind_calls(node, ctx);
    let Some(parent) = outermost_paren_parent(function_node, ctx) else {
        return false;
    };
    let Some(parent) = outermost_paren_parent(parent, ctx) else {
        return false;
    };

    let AstKind::CallExpression(call_expr) = parent.kind() else {
        return false;
    };

    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };

    if member_expr.is_computed() {
        return false;
    }
    let Some(static_prop_name) = member_expr.static_property_name() else {
        return false;
    };

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

// Traverse bind functions and return outer call expression
fn traverse_bind_calls<'a, 'b>(node: &'a AstNode<'b>, ctx: &'a LintContext<'b>) -> &'a AstNode<'b> {
    let mut current_node = node;
    loop {
        let Some(parent) = outermost_paren_parent(current_node, ctx) else {
            return current_node;
        };
        if !is_bind_member_expression(parent) {
            return current_node;
        }

        let Some(grand_parent) = outermost_paren_parent(parent, ctx) else {
            return current_node;
        };
        let AstKind::CallExpression(_) = grand_parent.kind() else {
            return current_node;
        };

        current_node = grand_parent;
    }
}

fn is_bind_member_expression(node: &AstNode) -> bool {
    let AstKind::MemberExpression(member_expr) = node.kind() else {
        return false;
    };

    member_expr.static_property_name() == Some("bind")
}

fn match_arrow_function_body<'a>(ctx: &LintContext<'a>, parent: &AstNode<'a>) -> bool {
    match ctx.nodes().parent_node(parent.id()) {
        Some(arrow_function_body) => match arrow_function_body.kind() {
            AstKind::FunctionBody(_) => match ctx.nodes().parent_node(arrow_function_body.id()) {
                Some(arrow_function) => {
                    matches!(arrow_function.kind(), AstKind::ArrowFunctionExpression(_))
                }
                None => false,
            },
            _ => false,
        },
        None => false,
    }
}

fn generate_fix<'a>(
    call_expr: &CallExpression,
    is_reject: bool,
    is_yield: bool,
    is_in_try_statement: bool,
    fixer: RuleFixer<'_, 'a>,
    ctx: &LintContext<'a>,
    node: &AstNode<'a>,
) -> RuleFix<'a> {
    if call_expr.arguments.len() > 1 {
        return fixer.noop();
    }

    let arg_text = if call_expr.arguments.is_empty() {
        &String::new()
    } else {
        let arg = &call_expr.arguments[0];
        if arg.is_spread() {
            return fixer.noop();
        }
        fixer.source_range(arg.span())
    };

    if is_reject {
        if is_in_try_statement {
            return fixer.noop();
        }
        if is_yield {
            if let Some(parent) = ctx.nodes().parent_node(node.id()) {
                if let Some(grand_parent) = ctx.nodes().parent_node(parent.id()) {
                    if !matches!(
                        grand_parent.kind(),
                        AstKind::ExpressionStatement(_) | AstKind::ParenthesizedExpression(_)
                    ) {
                        return fixer.noop();
                    }
                };
            };
        };
    }

    let node = get_parenthesized_node(node, ctx);

    let Some(parent) = ctx.nodes().parent_node(node.id()) else {
        return fixer.noop();
    };

    let is_arrow_function_body = match parent.kind() {
        AstKind::ExpressionStatement(_) => match_arrow_function_body(ctx, parent),
        _ => false,
    };

    let mut replace_range = if is_reject { parent.kind().span() } else { call_expr.span() };
    let replacement_text = if is_reject {
        let text = if arg_text.is_empty() { "undefined" } else { arg_text };
        let mut text = format!("throw {text}");

        if is_yield {
            replace_range = get_parenthesized_node(parent, ctx).kind().span();
            text
        } else {
            text = format!("{text};");
            // `=> Promise.reject(error)` -> `=> { throw error; }`
            if is_arrow_function_body {
                replace_range = get_parenthesized_node(parent, ctx).kind().span();
                text = format!("{{ {text} }}");
            }
            text
        }
    } else {
        let mut text = arg_text.to_string();
        if text.is_empty() {
            // `=> Promise.resolve()` -> `=> {}`
            if is_arrow_function_body {
                text = "{}".to_string();
                text
            } else {
                if matches!(parent.kind(), AstKind::ReturnStatement(_)) {
                    replace_range.start = parent.kind().span().start + 6;
                }
                if is_yield {
                    replace_range.start = parent.kind().span().start + 5;
                }
                text
            }
        } else {
            if matches!(&call_expr.arguments[0], Argument::ObjectExpression(_)) {
                text = format!("({text})");
            }
            text
        }
    };

    fixer.replace(replace_range, replacement_text)
}

fn get_parenthesized_node<'a, 'b>(
    node: &'a AstNode<'b>,
    ctx: &'a LintContext<'b>,
) -> &'a AstNode<'b> {
    let mut node = node;
    while let Some(parent_node) = ctx.nodes().parent_node(node.id()) {
        if let AstKind::ParenthesizedExpression(_) = parent_node.kind() {
            node = parent_node;
        } else {
            break;
        }
    }
    node
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Async functions returning normal values/throwing values
        (r"async () => bar;", None),
        (
            r"
            async () => {
                return bar;
            };
        ",
            None,
        ),
        (
            r"
            async function foo() {
                return bar;
            }
        ",
            None,
        ),
        (
            r"
            (async function() {
                return bar;
            });
        ",
            None,
        ),
        (
            r"
            async () => {
                throw bar;
            };
        ",
            None,
        ),
        (
            r"
            async function foo() {
                throw bar;
            }
        ",
            None,
        ),
        (
            r"
            (async function() {
                throw bar;
            });
        ",
            None,
        ),
        (
            // Async functions returning normal values/throwing values
            // Sync function returning Promise.resolve/reject
            r"() => Promise.resolve(bar);",
            None,
        ),
        (
            r"
            () => {
                return Promise.resolve(bar);
            };
        ",
            None,
        ),
        (
            r"
            function foo() {
                return Promise.resolve(bar);
            };
        ",
            None,
        ),
        (
            r"
            (function() {
                return Promise.resolve(bar);
            });
            ",
            None,
        ),
        (r"() => Promise.reject(bar);", None),
        (
            r"
            () => {
                return Promise.reject(bar);
            };
        ",
            None,
        ),
        (
            r"
            function foo() {
                return Promise.reject(bar);
            };
        ",
            None,
        ),
        (
            r"
            (function() {
                return Promise.reject(bar);
            });
        ",
            None,
        ),
        (
            // Sync generator yielding Promise.resolve/reject
            r"
            function * foo() {
                yield Promise.$resolve(bar);
            }
        ",
            None,
        ),
        (
            r"
            (function * () {
                yield Promise.$resolve(bar);
            })
        ",
            None,
        ),
        (
            r"
            function * foo() {
                yield Promise.reject(bar);
            }
        ",
            None,
        ),
        (
            r"
            (function * () {
                yield Promise.reject(bar);
            })
        ",
            None,
        ),
        (
            // Sync function nested in async function
            r"
            async function foo() {
                function bar() {
                    return Promise.resolve(baz);
                }
            }
        ",
            None,
        ),
        // Delegate yield expressions
        (
            r"
            async function * foo() {
				yield* Promise.resolve(bar);
			}
        ",
            None,
        ),
        (
            r"
            async function * foo() {
				yield* Promise.reject(bar);
			}
        ",
            None,
        ),
        // Promise#then/catch/finally
        (r"promise.then(() => foo).catch(() => bar).finally(() => baz)", None),
        (r"promise.then(() => foo, () => bar).finally(() => baz)", None),
        (r"promise.then(x, y, () => Promise.resolve(foo))", None),
        (r"promise.catch(x, () => Promise.resolve(foo))", None),
        (r"promise.finally(x, () => Promise.resolve(foo))", None),
        (r"promise[then](() => Promise.resolve(foo))", None),
        // additional cases:
        (r"(async () => { Promise.resolve().then(() => console.log('foo')); })();", None),
        // TODO: enhance to report this case?
        (
            r#"fs.promises.readFile("foo", 'utf8').then(undefined, err => err.code === 'ENOENT' ? Promise.resolve('{}') : Promise.reject(err))"#,
            None,
        ),
        (r"Promise.resolve(4).then(function(x) { return x })", None),
        (r"Promise.reject(4).then(function(x) { return x })", None),
        (r"Promise.resolve(4).then(function() {})", None),
        (r"Promise.reject(4).then(function() {})", None),
        (r"doThing().then(function() { return 4 })", None),
        (r"doThing().then(function() { throw 4 })", None),
        (r"doThing().then(null, function() { return 4 })", None),
        (r"doThing().then(null, function() { throw 4 })", None),
        (r"doThing().catch(null, function() { return 4 })", None),
        (r"doThing().catch(null, function() { throw 4 })", None),
        (r"doThing().then(function() { return Promise.all([a,b,c]) })", None),
        (r"doThing().then(() => 4)", None),
        (r"doThing().then(() => { throw 4 })", None),
        (r"doThing().then(()=>{}, () => 4)", None),
        (r"doThing().then(()=>{}, () => { throw 4 })", None),
        (r"doThing().catch(() => 4)", None),
        (r"doThing().catch(() => { throw 4 })", None),
        (r"var x = function() { return Promise.resolve(4) }", None),
        (r"function y() { return Promise.resolve(4) }", None),
        (r"function then() { return Promise.reject() }", None),
        (r"doThing(function(x) { return Promise.reject(x) })", None),
        (r"doThing().then(function() { return })", None),
        (
            "doThing().then(function() { return Promise.reject(4) })",
            Some(serde_json::json!([{ "allowReject": true }])),
        ),
        (r"doThing().then((function() { return Promise.resolve(4) }).toString())", None),
        (
            "doThing().then(() => Promise.reject(4))",
            Some(serde_json::json!([{ "allowReject": true }])),
        ),
        (r"doThing().then(function() { return a() })", None),
        (r"doThing().then(function() { return Promise.a() })", None),
        (r"doThing().then(() => { return a() })", None),
        (r"doThing().then(() => { return Promise.a() })", None),
        (r"doThing().then(() => a())", None),
        (r"doThing().then(() => Promise.a())", None),
    ];

    let fail = vec![
        (
            r"
            const main = async foo => {
                if (foo > 4) {
                    return Promise.reject(new Error('🤪'));
                }
                return Promise.resolve(result);
            };
        ",
            None,
        ),
        // Async function returning Promise.resolve
        (r"async () => Promise.resolve(bar);", None),
        (
            r"
            async () => {
                return Promise.resolve(bar);
            };
        ",
            None,
        ),
        (
            r"
            async function foo() {
                return Promise.resolve(bar);
            }
        ",
            None,
        ),
        (
            r"
			(async function() {
				return Promise.resolve(bar);
			});
		",
            None,
        ),
        (
            r"
			async function * foo() {
				return Promise.resolve(bar);
			}
        ",
            None,
        ),
        (
            r"
		    (async function*() {
		    	return Promise.resolve(bar);
		    });
        ",
            None,
        ),
        // Async function returning Promise.reject
        (r"async () => Promise.reject(bar);", None),
        (
            r"
			async () => {
				return Promise.reject(bar);
			};
		",
            None,
        ),
        (
            r"
			async function foo() {
				return Promise.reject(bar);
		    }
		",
            None,
        ),
        (
            r"
			(async function() {
				return Promise.reject(bar);
			});
		",
            None,
        ),
        (
            r"
			async function * foo() {
				return Promise.reject(bar);
			}
		",
            None,
        ),
        (
            r"
			(async function*() {
				return Promise.reject(bar);
			});
		",
            None,
        ),
        // Async generator yielding Promise.resolve
        (
            r"
				async function * foo() {
					yield Promise.resolve(bar);
				}
			",
            None,
        ),
        (
            r"
				(async function * () {
					yield Promise.resolve(bar);
				});
				",
            None,
        ),
        (
            // Async generator yielding Promise.reject
            r"
				async function * foo() {
					yield Promise.reject(bar);
				}
				",
            None,
        ),
        (
            r"
				(async function * () {
					yield Promise.reject(bar);
				});
				",
            None,
        ),
        (r"async () => Promise.resolve();", None),
        (
            r"
				async function foo() {
					return Promise.resolve();
				}
				",
            None,
        ),
        (r"async () => Promise.reject();", None),
        (
            r"
				async function foo() {
					return Promise.reject();
				}
				",
            None,
        ),
        (
            r"
				async function * foo() {
					yield Promise.resolve();
				}
				",
            None,
        ),
        // Multiple arguments
        (r"async () => Promise.resolve(bar, baz);", None),
        (r"async () => Promise.reject(bar, baz);", None),
        // Sequence expressions
        (
            r"async
				async function * foo() {
					yield Promise.resolve((bar, baz));
				}
		",
            None,
        ),
        (r"async () => Promise.resolve((bar, baz))", None),
        // Arrow function returning an object
        (r"async () => Promise.resolve({})", None),
        // Try statements
        (
            r"
				async function foo() {
					try {
						return Promise.resolve(1);
					} catch {}
				}
		",
            None,
        ),
        (
            r"
				async function foo() {
					try {
						return Promise.reject(1);
					} catch {}
				}
				",
            None,
        ),
        // Spread arguments
        (r"async () => Promise.resolve(...bar);", None),
        (r"async () => Promise.reject(...bar);", None),
        // Yield not in an ExpressionStatement
        (
            r"#
        async function * foo() {
            const baz = yield Promise.resolve(bar);
        }
        #",
            None,
        ),
        (
            r"
        async function * foo() {
            const baz = yield Promise.reject(bar);
        }
        ",
            None,
        ),
        // Parenthesized Promise.resolve/reject
        (r"async () => (Promise.resolve(bar));", None),
        (r"async () => (Promise.reject(bar));", None),
        (r"async () => ((Promise.reject(bar)));", None),
        (
            r"
            async function * foo() {
     			(yield Promise.reject(bar));
     		}
        ",
            None,
        ),
        (
            r"
            async function * foo() {
     			((yield Promise.reject(bar)));
     		}
        ",
            None,
        ),
        (r"promise.then(() => Promise.resolve(bar))", None),
        (r"promise.then(() => { return Promise.resolve(bar); })", None),
        (r"promise.then(async () => Promise.reject(bar))", None),
        (r"promise.then(async () => { return Promise.reject(bar); })", None),
        (r"promise.catch(() => Promise.resolve(bar))", None),
        (r"promise.catch(() => { return Promise.resolve(bar); })", None),
        (r"promise.catch(async () => Promise.reject(bar))", None),
        (r"promise.catch(async () => { return Promise.reject(bar); })", None),
        (r"promise.finally(() => Promise.resolve(bar))", None),
        (r"promise.finally(() => { return Promise.resolve(bar); })", None),
        (r"promise.finally(async () => Promise.reject(bar))", None),
        (r"promise.finally(async () => { return Promise.reject(bar); })", None),
        (r"promise.then(() => {}, () => Promise.resolve(bar))", None),
        (r"promise.then(() => Promise.resolve(bar), () => Promise.resolve(baz))", None),
        (r"promise.then(() => {}, () => { return Promise.resolve(bar); })", None),
        (r"promise.then(() => {}, async () => Promise.reject(bar))", None),
        (r"promise.then(() => {}, async () => { return Promise.reject(bar); })", None),
        (r"doThing().then(function() { return Promise.resolve(4) })", None),
        (r"doThing().then(null, function() { return Promise.resolve(4) })", None),
        (r"doThing().catch(function() { return Promise.resolve(4) })", None),
        (r"doThing().then(function() { return Promise.reject(4) })", None),
        (r"doThing().then(null, function() { return Promise.reject(4) })", None),
        (r"doThing().catch(function() { return Promise.reject(4) })", None),
        (
            r#"doThing().then(function(x) { if (x>1) { return Promise.resolve(4) } else { throw "bad" } })"#,
            None,
        ),
        (r"doThing().then(function(x) { if (x>1) { return Promise.reject(4) } })", None),
        (
            r"doThing().then(null, function() { if (true && false) { return Promise.resolve() } })",
            None,
        ),
        (
            r"doThing().catch(function(x) {if (x) { return Promise.resolve(4) } else { return Promise.reject() } })",
            None,
        ),
        (
            "
                          fn(function() {
                            doThing().then(function() {
                              return Promise.resolve(4)
                            })
                            return
                          })",
            None,
        ),
        (
            "
                          fn(function() {
                            doThing().then(function nm() {
                              return Promise.resolve(4)
                            })
                            return
                          })",
            None,
        ),
        (
            "
                          fn(function() {
                            fn2(function() {
                              doThing().then(function() {
                                return Promise.resolve(4)
                              })
                            })
                          })",
            None,
        ),
        (
            "
                          fn(function() {
                            fn2(function() {
                              doThing().then(function() {
                                fn3(function() {
                                  return Promise.resolve(4)
                                })
                                return Promise.resolve(4)
                              })
                            })
                          })",
            None,
        ),
        (
            "
                          const o = {
                            fn: function() {
                              return doThing().then(function() {
                                return Promise.resolve(5);
                              });
                            },
                          }
                          ",
            None,
        ),
        (
            "
                          fn(
                            doThing().then(function() {
                              return Promise.resolve(5);
                            })
                          );
                          ",
            None,
        ),
        (r"doThing().then((function() { return Promise.resolve(4) }).bind(this))", None),
        (r"doThing().then((function() { return Promise.resolve(4) }).bind(this).bind(this))", None),
        (r"doThing().then(() => { return Promise.resolve(4) })", None),
        (
            "
                          function a () {
                            return p.then(function(val) {
                              return Promise.resolve(val * 4)
                            })
                          }
                          ",
            None,
        ),
        (r"doThing().then(() => Promise.resolve(4))", None),
        (r"doThing().then(() => Promise.reject(4))", None),
    ];

    let fix = vec![
        (
            r"
				const main = async foo => {
					if (foo > 4) {
						return Promise.reject(new Error('🤪'));
					}

					return Promise.resolve(result);
				};
        ",
            r"
				const main = async foo => {
					if (foo > 4) {
						throw new Error('🤪');
					}

					return result;
				};
        ",
            None,
        ),
        // Async function returning Promise.resolve
        ("async () => Promise.resolve(bar);", "async () => bar;", None),
        (
            r"
				async () => {
					return Promise.resolve(bar);
				};
        ",
            r"
				async () => {
					return bar;
				};
        ",
            None,
        ),
        (
            r"
				async function foo() {
					return Promise.resolve(bar);
				}
        ",
            r"
				async function foo() {
					return bar;
				}
        ",
            None,
        ),
        (
            r"
				(async function() {
					return Promise.resolve(bar);
				});
        ",
            r"
				(async function() {
					return bar;
				});
        ",
            None,
        ),
        (
            r"
				async function * foo() {
					return Promise.resolve(bar);
				}
        ",
            r"
				async function * foo() {
					return bar;
				}
        ",
            None,
        ),
        (
            r"
				(async function*() {
					return Promise.resolve(bar);
				});
        ",
            r"
				(async function*() {
					return bar;
				});
        ",
            None,
        ),
        // Async function returning Promise.reject
        ("async () => Promise.reject(bar);", "async () => { throw bar; };", None),
        (
            r"
				async () => {
					return Promise.reject(bar);
				};
        ",
            r"
				async () => {
					throw bar;
				};
        ",
            None,
        ),
        (
            r"
				async function foo() {
					return Promise.reject(bar);
				}
        ",
            r"
				async function foo() {
					throw bar;
				}
        ",
            None,
        ),
        (
            r"
				(async function() {
					return Promise.reject(bar);
				});
        ",
            r"
				(async function() {
					throw bar;
				});
        ",
            None,
        ),
        (
            r"
				async function * foo() {
					return Promise.reject(bar);
				}
        ",
            r"
				async function * foo() {
					throw bar;
				}
        ",
            None,
        ),
        (
            r"
				(async function*() {
					return Promise.reject(bar);
				});
        ",
            r"
				(async function*() {
					throw bar;
				});
        ",
            None,
        ),
        // Async generator yielding Promise.resolve
        (
            r"
				async function * foo() {
					yield Promise.resolve(bar);
				}
        ",
            r"
				async function * foo() {
					yield bar;
				}
        ",
            None,
        ),
        (
            r"
				(async function * () {
					yield Promise.resolve(bar);
				});
        ",
            r"
				(async function * () {
					yield bar;
				});
        ",
            None,
        ),
        // Async generator yielding Promise.reject
        (
            r"
				async function * foo() {
					yield Promise.reject(bar);
				}
        ",
            r"
				async function * foo() {
					throw bar;
				}
        ",
            None,
        ),
        (
            r"
				(async function * () {
					yield Promise.reject(bar);
				});
        ",
            r"
				(async function * () {
					throw bar;
				});
        ",
            None,
        ),
        // No arguments
        (r"async () => Promise.resolve();", r"async () => {};", None),
        (
            r"
            async function foo() {
                return Promise.resolve();
            }
            ",
            r"
            async function foo() {
                return;
            }
            ",
            None,
        ),
        (r"async () => Promise.reject();", r"async () => { throw undefined; };", None),
        (
            r"
            async function foo() {
                return Promise.reject();
            }
            ",
            r"
            async function foo() {
                throw undefined;
            }
            ",
            None,
        ),
        (
            r"
            async function * foo() {
                yield Promise.resolve();
            }
            ",
            r"
            async function * foo() {
                yield;
            }
            ",
            None,
        ),
        // Sequence expressions
        (
            r"
            async function * foo() {
                yield Promise.resolve((bar, baz));
            }
            ",
            r"
            async function * foo() {
                yield (bar, baz);
            }
            ",
            None,
        ),
        (r"async () => Promise.resolve((bar, baz))", r"async () => (bar, baz)", None),
        // Arrow function returning an object
        (r"async () => Promise.resolve({})", r"async () => ({})", None),
        // Try statements
        (
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
                    return 1;
                } catch {}
            }
            ",
            None,
        ),
        // Try statements reject case: don't change
        (
            r"
            async function foo() {
                try {
                    return Promise.reject(1);
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
            None,
        ),
        // Spread arguments can not modify the original array
        (r"async () => Promise.resolve(...bar);", r"async () => Promise.resolve(...bar);", None),
        (r"async () => Promise.reject(...bar);", r"async () => Promise.reject(...bar);", None),
        // Yield not in an ExpressionStatement
        (
            r"
            async function * foo() {
                const baz = yield Promise.resolve(bar);
            }
            ",
            r"
            async function * foo() {
                const baz = yield bar;
            }
            ",
            None,
        ),
        // Yield reject not in an ExpressionStatement not modify
        (
            r"
            async function * foo() {
                const baz = yield Promise.reject(bar);
            }
            ",
            r"
            async function * foo() {
                const baz = yield Promise.reject(bar);
            }
            ",
            None,
        ),
        // Parenthesized Promise.resolve/reject
        (r"async () => (Promise.resolve(bar));", r"async () => (bar);", None),
        (r"async () => (Promise.reject(bar));", r"async () => { throw bar; };", None),
        (r"async () => ((Promise.reject(bar)));", r"async () => { throw bar; };", None),
        (
            r"
            async function * foo() {
                (yield Promise.reject(bar));
            }
            ",
            r"
            async function * foo() {
                throw bar;
            }
            ",
            None,
        ),
        (
            r"
            async function * foo() {
                ((yield Promise.reject(bar)));
            }
            ",
            r"
            async function * foo() {
                throw bar;
            }
            ",
            None,
        ),
        // Promise#then/catch/finally callbacks returning Promise.resolve/reject
        (
            r"promise.then(() => {}, () => Promise.resolve(bar))",
            r"promise.then(() => {}, () => bar)",
            None,
        ),
        (
            r"promise.then(() => Promise.resolve(bar), () => Promise.resolve(baz))",
            r"promise.then(() => bar, () => baz)",
            None,
        ),
        (
            r"promise.then(() => {}, () => { return Promise.resolve(bar); })",
            r"promise.then(() => {}, () => { return bar; })",
            None,
        ),
        (
            r"promise.then(() => {}, async () => Promise.reject(bar))",
            r"promise.then(() => {}, async () => { throw bar; })",
            None,
        ),
        (
            r"promise.then(() => {}, async () => { return Promise.reject(bar); })",
            r"promise.then(() => {}, async () => { throw bar; })",
            None,
        ),
        (
            r"promise.then(() => {}, async () => { return Promise.reject(bar); })",
            r"promise.then(() => {}, async () => { throw bar; })",
            None,
        ),
    ];

    Tester::new(
        NoUselessPromiseResolveReject::NAME,
        NoUselessPromiseResolveReject::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
