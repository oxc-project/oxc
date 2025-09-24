use rustc_hash::FxHashSet;

use oxc_ast::{AstKind, ast::*};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ReferenceId, ScopeFlags, ScopeId, SymbolId};
use oxc_span::{Atom, GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{get_function_like_declaration, nth_outermost_paren_parent, outermost_paren_parent},
    context::LintContext,
    rule::Rule,
    utils::is_react_hook,
};

fn consistent_function_scoping(
    fn_span: Span,
    parent_scope_span: Option<Span>,
    parent_scope_kind: Option<&'static str>,
    function_name: Option<Atom<'_>>,
) -> OxcDiagnostic {
    let function_label = if let Some(name) = function_name {
        format!("Function `{name}` does not capture any variables from its parent scope")
    } else {
        "This function does not use any variables from its parent scope".into()
    };

    let d = OxcDiagnostic::warn(function_label).with_help(match function_name {
        Some(name) => {
            format!("Move `{name}` to the outer scope to avoid recreating it on every call.")
        }
        None => {
            "Move this function to the outer scope to avoid recreating it on every call.".into()
        }
    });

    match parent_scope_span {
        Some(parent) => d.with_labels([
            parent.label("Outer scope where this function is defined"),
            fn_span.primary_label(if let Some(parent_scope_kind) = parent_scope_kind {
                format!(
                    "This function does not use any variables from the parent {parent_scope_kind}"
                )
            } else {
                "This function does not use any variables from here".into()
            }),
        ]),
        None => d.with_label(fn_span),
    }
}

#[derive(Debug, Clone)]
pub struct ConsistentFunctionScoping {
    check_arrow_functions: bool,
}

impl Default for ConsistentFunctionScoping {
    fn default() -> Self {
        Self { check_arrow_functions: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow functions that are declared in a scope which does not capture
    /// any variables from the outer scope.
    ///
    /// ### Why is this bad?
    ///
    /// Moving function declarations to the highest possible scope improves
    /// readability, directly [improves performance](https://stackoverflow.com/questions/80802/does-use-of-anonymous-functions-affect-performance/81329#81329)
    /// and allows JavaScript engines to better [optimize your performance](https://ponyfoo.com/articles/javascript-performance-pitfalls-v8#optimization-limit).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// export function doFoo(foo) {
    ///   // Does not capture anything from the scope, can be moved to the outer scope
    ///	  function doBar(bar) {
    ///	    return bar === 'bar';
    ///	  }
    ///	  return doBar;
    /// }
    ///
    /// function doFoo(foo) {
    ///   const doBar = bar => {
    ///     return bar === 'bar';
    ///   };
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function doBar(bar) {
    ///   return bar === 'bar';
    /// }
    ///
    /// export function doFoo(foo) {
    ///   return doBar;
    /// }
    ///
    /// export function doFoo(foo) {
    ///   function doBar(bar) {
    ///     return bar === 'bar' && foo.doBar(bar);
    ///   }
    ///   return doBar;
    /// }
    /// ```
    /// ### Options
    ///
    /// #### checkArrowFunctions
    ///
    /// `{ type: boolean, default: true }`
    ///
    /// Pass `"checkArrowFunctions": false` to disable linting of arrow functions.
    ///
    /// Example:
    /// ```json
    /// "unicorn/consistent-function-scoping": [
    ///   "error",
    ///   { "checkArrowFunctions": false }
    /// ]
    /// ```
    ///
    /// ### Limitations
    ///
    /// This rule does not detect or remove extraneous code blocks inside of functions:
    ///
    /// ```js
    /// function doFoo(foo) {
    /// 	{
    /// 		function doBar(bar) {
    /// 			return bar;
    /// 		}
    /// 	}
    ///
    /// 	return foo;
    /// }
    /// ```
    ///
    /// It also ignores functions that contain `JSXElement` references:
    ///
    /// ```jsx
    /// function doFoo(FooComponent) {
    /// 	function Bar() {
    /// 		return <FooComponent/>;
    /// 	}
    ///
    /// 	return Bar;
    /// };
    /// ```
    ///
    /// [Immediately invoked function expressions (IIFE)](https://en.wikipedia.org/wiki/Immediately_invoked_function_expression) are ignored:
    ///
    /// ```js
    /// (function () {
    /// 	function doFoo(bar) {
    /// 		return bar;
    /// 	}
    /// })();
    /// ```
    ConsistentFunctionScoping,
    unicorn,
    suspicious,
    pending
);

impl Rule for ConsistentFunctionScoping {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            check_arrow_functions: value
                .get(0)
                .and_then(|val| val.get("checkArrowFunctions"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (
            function_declaration_symbol_id,
            function_name,
            function_body,
            reporter_span,
            function_scope_id,
        ) =
            match node.kind() {
                AstKind::Function(function) => {
                    if function.is_typescript_syntax() {
                        return;
                    }

                    let func_scope_id = function.scope_id();
                    if let Some(parent_scope_id) = ctx.scoping().scope_parent_id(func_scope_id) {
                        // Example: const foo = function bar() {};
                        // The bar function scope id is 1. In order to ignore this rule,
                        // its parent's scope id (in this case `foo`'s scope id is 0 and is equal to root scope id)
                        // should be considered.
                        //
                        // We also allow functions declared in TS module/namespace blocks.
                        let flags = ctx.scoping().scope_flags(parent_scope_id);
                        if flags.intersects(ScopeFlags::Top | ScopeFlags::TsModuleBlock) {
                            return;
                        }
                    }

                    // NOTE: function.body will always be some here because of
                    // checks in `is_typescript_syntax`
                    let Some(function_body) = &function.body else { return };

                    if let Some(binding_ident) = get_function_like_declaration(node, ctx) {
                        (
                            binding_ident.symbol_id(),
                            Some(binding_ident.name),
                            function_body,
                            function.id.as_ref().map_or(
                                Span::sized(function.span.start, 8),
                                |func_binding_ident| func_binding_ident.span,
                            ),
                            func_scope_id,
                        )
                    } else if let Some(function_id) = &function.id {
                        (
                            function_id.symbol_id(),
                            Some(function_id.name),
                            function_body,
                            function_id.span(),
                            func_scope_id,
                        )
                    } else {
                        return;
                    }
                }
                AstKind::ArrowFunctionExpression(arrow_function) if self.check_arrow_functions => {
                    let Some(binding_ident) = get_function_like_declaration(node, ctx) else {
                        return;
                    };

                    (
                        binding_ident.symbol_id(),
                        Some(binding_ident.name),
                        &arrow_function.body,
                        binding_ident.span(),
                        arrow_function.scope_id(),
                    )
                }
                _ => return,
            };

        // if the function is declared at the root scope or in a TS
        // module/namespace block, we don't need to check anything
        let scope = ctx.scoping().symbol_scope_id(function_declaration_symbol_id);
        if ctx.scoping().scope_flags(scope).intersects(ScopeFlags::Top | ScopeFlags::TsModuleBlock)
        {
            return;
        }

        if matches!(
            outermost_paren_parent(node, ctx).map(AstNode::kind),
            Some(AstKind::ReturnStatement(_) | AstKind::Argument(_))
        ) {
            return;
        }

        if is_parent_scope_iife(node, ctx) || is_in_react_hook(node, ctx) {
            return;
        }

        // get all references in the function body
        let (function_body_var_references, is_parent_this_referenced) = {
            let mut rf = ReferencesFinder::default();
            rf.visit_function_body(function_body);
            (rf.references, rf.is_parent_this_referenced)
        };

        if is_parent_this_referenced && matches!(node.kind(), AstKind::ArrowFunctionExpression(_)) {
            return;
        }

        let parent_scope_ids = {
            let mut current_scope_id = function_scope_id;
            let mut parent_scope_ids = FxHashSet::default();
            while let Some(parent_scope_id) = ctx.scoping().scope_parent_id(current_scope_id) {
                parent_scope_ids.insert(parent_scope_id);
                current_scope_id = parent_scope_id;
            }
            parent_scope_ids
        };

        for reference_id in function_body_var_references {
            let reference = ctx.scoping().get_reference(reference_id);
            let Some(symbol_id) = reference.symbol_id() else { continue };
            let scope_id = ctx.scoping().symbol_scope_id(symbol_id);
            if parent_scope_ids.contains(&scope_id) && symbol_id != function_declaration_symbol_id {
                return;
            }
        }

        let (maybe_parent_scope_span, maybe_parent_scope_type) =
            get_short_span_for_fn_scope(ctx, function_declaration_symbol_id, scope)
                .map_or((None, None), |(span, rtype)| (Some(span), Some(rtype)));

        ctx.diagnostic(consistent_function_scoping(
            reporter_span,
            maybe_parent_scope_span,
            maybe_parent_scope_type,
            function_name,
        ));
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        // .d.ts files are never run, so there are no perf considerations for them.
        !ctx.source_type().is_typescript_definition()
    }
}

#[derive(Default)]
struct ReferencesFinder {
    is_parent_this_referenced: bool,
    references: Vec<ReferenceId>,
    in_function: usize,
}

impl<'a> Visit<'a> for ReferencesFinder {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        self.references.push(it.reference_id());
    }

    fn visit_jsx_element_name(&mut self, _it: &JSXElementName<'a>) {
        // Ignore references in JSX elements e.g. `Foo` in `<Foo>`.
        // No need to walk children as only references they may contain are also JSX identifiers.
    }

    fn visit_this_expression(&mut self, _: &ThisExpression) {
        if self.in_function == 0 {
            self.is_parent_this_referenced = true;
        }
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        self.in_function += 1;
        walk::walk_function(self, func, flags);
        self.in_function -= 1;
    }
}

fn is_parent_scope_iife<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if let Some(parent_node) = outermost_paren_parent(node, ctx)
        && let Some(parent_node) = outermost_paren_parent(parent_node, ctx)
        && matches!(parent_node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
        && let Some(parent_node) = outermost_paren_parent(parent_node, ctx)
    {
        return matches!(parent_node.kind(), AstKind::CallExpression(_));
    }

    false
}

fn is_in_react_hook<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // we want the 3rd outermost parent
    // parents are: function body -> function -> argument -> call expression
    if let Some(parent) = nth_outermost_paren_parent(node, ctx, 3)
        && let AstKind::CallExpression(call_expr) = parent.kind()
    {
        return is_react_hook(&call_expr.callee);
    }
    false
}

fn get_short_span_for_fn_scope(
    ctx: &LintContext<'_>,
    function_symbol_id: SymbolId,
    scope_id: ScopeId,
) -> Option<(Span, &'static str)> {
    let scoping = ctx.scoping();

    debug_assert!(!scoping.scope_flags(scope_id).contains(ScopeFlags::Top));

    let scope_id =
        match ctx.nodes().parent_kind(ctx.scoping().symbol_declaration(function_symbol_id)) {
            AstKind::AssignmentExpression(_) | AstKind::ObjectProperty(_) => {
                ctx.scoping().scope_parent_id(scope_id).unwrap_or(scope_id)
            }
            _ => scope_id,
        };

    let node_creating_parent_scope = ctx.nodes().get_node(scoping.get_node_id(scope_id));

    match node_creating_parent_scope.kind() {
        AstKind::Function(f) => f.id.as_ref().map(|id| (id.span(), "function")),
        AstKind::ArrowFunctionExpression(_) => {
            let parent = ctx.nodes().parent_kind(node_creating_parent_scope.id());
            match parent {
                AstKind::VariableDeclarator(v) => Some((v.id.span(), "arrow function")),
                AstKind::AssignmentExpression(a) => Some((a.left.span(), "arrow function")),
                _ => None,
            }
        }
        AstKind::Class(c) => c.id.as_ref().map(|id| (id.span(), "class")),
        // only cover keywords of control flow statements
        AstKind::ForInStatement(ForInStatement { span, .. })
        | AstKind::ForOfStatement(ForOfStatement { span, .. })
        | AstKind::ForStatement(ForStatement { span, .. }) => {
            Some((Span::sized(span.start, 3), "for loop"))
        }
        AstKind::TryStatement(TryStatement { span, .. }) => {
            Some((Span::sized(span.start, 3), "try statement"))
        }
        AstKind::IfStatement(IfStatement { span, .. }) => {
            Some((Span::sized(span.start, 2), "if statement"))
        }
        AstKind::DoWhileStatement(DoWhileStatement { span, .. }) => {
            Some((Span::sized(span.start, 2), "do while statement"))
        }
        AstKind::SwitchStatement(SwitchStatement { span, .. }) => {
            Some((Span::sized(span.start, 6), "switch statement"))
        }
        AstKind::WhileStatement(WhileStatement { span, .. }) => {
            Some((Span::sized(span.start, 5), "while statement"))
        }
        AstKind::CatchClause(CatchClause { span, .. }) => {
            Some((Span::sized(span.start, 5), "catch block"))
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function doFoo(foo) { return foo; }", None),
        ("function doFoo(foo) { return bar; }", None),
        ("const doFoo = function() {};", None),
        ("const doFoo = foo => foo;", None),
        ("foo => foo;", None),
        ("function doFoo(foo) { function doBar(bar) { return foo + bar; } return foo; }", None),
        (
            "const doFoo = function(foo) {
                function doBar(bar) {
                    return foo + bar;
                }
                return foo;
            };",
            None,
        ),
        (
            "const doFoo = function(foo) {
                const doBar = function(bar) {
                    return foo + bar;
                };
                return foo;
            };",
            None,
        ),
        (
            "function doFoo(foo) {
                const doBar = function(bar) {
                    return foo + bar;
                };
                return foo;
            }",
            None,
        ),
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    return foo + bar;
                }
            }",
            None,
        ),
        (
            "function doFoo(foo = 'foo') {
                function doBar(bar) {
                    return foo + bar;
                }
            }",
            None,
        ),
        (
            "function doFoo() {
                const foo = 'foo';
                function doBar(bar) {
                    return foo + bar;
                }
                return foo;
            }",
            None,
        ),
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    function doZaz(zaz) {
                        return foo + bar + zaz;
                    }
                    return bar;
                }
                return foo;
            }",
            None,
        ),
        ("for (let foo = 0; foo < 1; foo++) { function doBar(bar) { return bar + foo; } }", None),
        (
            "let foo = 0;
            function doFoo() {
                foo = 1;
                function doBar(bar) {
                    return foo + bar;
                }
                return foo;
            }",
            None,
        ),
        ("const doFoo = foo => { return foo; }", None),
        ("const doFoo = foo => bar => foo + bar;", None),
        ("const doFoo = () => { return bar => bar; } ", None),
        (
            "const doFoo = foo => {
                const doBar = bar => {
                    return foo + bar;
                }
                return foo;
            }",
            None,
        ),
        (
            "function doFoo() {
                {
                    const foo = 'foo';
                    function doBar(bar) {
                        return bar + foo;
                    }
                }
            }",
            None,
        ),
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    foo.bar = bar;
                }
                function doZaz(zaz) {
                    doBar(zaz);
                }

                doZaz('zaz');
            };",
            None,
        ),
        ("function doFoo() { return function doBar() {}; }", None),
        ("function doFoo(Foo) { function doBar() { return new Foo(); } return doBar; };", None),
        ("function doFoo(FooComponent) { return <FooComponent />; } ", None),
        ("const foo = <JSX/>;", None),
        ("function foo() { function bar() { return <JSX a={foo()}/>; } }", None),
        ("function doFoo(Foo) { const doBar = () => this; return doBar(); };", None),
        ("function doFoo(Foo) { const doBar = () => () => this; return doBar(); };", None),
        ("function doFoo(Foo) { const doBar = () => () => () => this; return doBar(); };", None),
        ("useEffect(() => { function foo() {} }, []) ", None),
        ("React.useEffect(() => { function foo() {} }, [])", None),
        ("(function() { function bar() {} })();", None),
        ("(function() { function bar() {} }());", None),
        ("!function() { function bar() {} }();", None),
        ("(() => { function bar() {} })();", None),
        ("(async function() { function bar() {} })();", None),
        (" (async function * () { function bar() {} })();", None),
        ("function doFoo() { const doBar = (function(bar) { return bar; })(); }", None),
        (
            "const enrichErrors = (packageName, cliArgs, f) => async (...args) => {
                try {
                    return await f(...args);
                } catch (error) {
                    error.packageName = packageName;
                    error.cliArgs = cliArgs;
                    throw error;
                }
            };",
            None,
        ),
        (
            "export const canStepForward = ([X, Y]) => ([x, y]) => direction => {
                switch (direction) {
                    case 0:
                        return y !== 0
                    case 1:
                        return x !== X - 1
                    case 2:
                        return y !== Y - 1
                    case 3:
                        return x !== 0
                    default:
                        throw new Error('unknown direction')
                }
            }",
            None,
        ),
        (
            "
            'use strict';
            module.exports = function recordErrors(eventEmitter, stateArgument) {
                const stateVariable = stateArgument;
                function onError(error) {
                    stateVariable.inputError = error;
                }
                eventEmitter.once('error', onError);
            };",
            None,
        ),
        (
            "module.exports = function recordErrors(eventEmitter, stateArgument) {
                function onError(error) {
                    stateArgument.inputError = error;
                }
                function onError2(error) {
                    onError(error);
                }

                eventEmitter.once('error', onError2);
            };",
            None,
        ),
        (
            "function outer(stream) {
                let content;

                function inner() {
                    process.stdout.write(content);
                }

                inner();
            }",
            None,
        ),
        (
            "function outer () { const inner = () => {} }",
            Some(serde_json::json!([{ "checkArrowFunctions": false }])),
        ),
        (
            "
                type Data<T> = T extends 'error' ? Error : Record<string, unknown> | unknown[]

                type Method = 'info' | 'error'

                export function createLogger(name: string) {
                    // Two lint errors are on the next line.
                    const log = <T extends Method>(method: T) => (data: Data<T>) => {
                            try {
                                    // eslint-disable-next-line no-console
                                    console[method](JSON.stringify({ name, data }))
                            } catch (error) {
                                    console.error(error)
                            }
                    }

                    return {
                            info: log('info'),
                            error: log('error'),
                    }
                }
            ",
            None,
        ),
        (
            "test('it works', async function(assert) {
                function assertHeader(assertions) {
                    for (const [key, value] of Object.entries(assertions)) {
                        assert.strictEqual(
                            native[key],
                            value
                        );
                    }
                }

                // ...
            });",
            None,
        ),
        (
            "export function a(x: number) {
                const b = (y: number) => (z: number): number => x + y + z;
                return b(1)(2);
            }",
            None,
        ),
        // https://github.com/oxc-project/oxc/pull/4948#issuecomment-2295819822
        ("t.throws(() => receiveString(function a() {}), {})", None),
        ("function test () { t.throws(() => receiveString(function a() {}), {}) }", None),
        ("function foo() { let x = new Bar(function b() {}) }", None),
        ("module.exports = function foo() {};", None),
        ("module.exports.foo = function foo() {};", None),
        ("foo.bar.func = function foo() {};", None),
        (
            "let inner;

            function foo1() {
                inner = function() {}
            }
            function foo2() {
                inner = function() {}
            }",
            None,
        ),
        ("if(f) function f(){}", None),
        (
            "
            export namespace Foo {
                export function somePublicFn() {
                    return somePrivateFn();
                }
                function somePrivateFn() {
                    return 'private';
                }
            }
        ",
            None,
        ),
        (
            "
            export namespace Foo {
                export function somePublicFn() {
                    return private1() + private2();
                }
                const private1 = function private1() {
                    return 'private1';
                }
                const private2 = () => {
                    return 'private2';
                }
            }
        ",
            None,
        ),
        (
            "
            declare namespace Foo {
                function foo(): void;
            }
            ",
            None,
        ),
        (
            "
            declare module 'some-package' {
                function foo(): void;
            }
            ",
            None,
        ),
        ("function outer() { { let x; var inner = () => x; } return inner; }", None),
    ];

    let fail = vec![
        // start of cases that eslint-plugin-unicorn passes, but we fail.

        // declared function is inside a block statement
        (
            "function doFoo(foo) {
                {
                    function doBar(bar) {
                        return bar;
                    }
                }
                return foo;
            }",
            None,
        ),
        (
            "function doFoo(FooComponent) {
                function Bar() {
                    return <FooComponent />;
                }
                return Bar;
            };",
            None,
        ),
        (
            "function Foo() {
                function Bar () {
                    return <div />
                }
                return <div>{ Bar() }</div>
            }",
            None,
        ),
        ("function foo() { function bar() { return <JSX/>; } }", None),
        ("function doFoo(Foo) { const doBar = () => arguments; return doBar(); };", None),
        (
            "let inner;

            function outer() {
                inner = function inner() {}
            }",
            None,
        ),
        // end of cases that eslint-plugin-unicorn passes, but we fail.
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    return bar;
                }
                return foo;
            }",
            None,
        ),
        (
            "function doFoo() {
                const foo = 'foo';
                function doBar(bar) {
                    return bar;
                }
                return foo;
            }",
            None,
        ),
        ("function doFoo() { function doBar(bar) { return bar; } }", None),
        ("const doFoo = function() { function doBar(bar) { return bar; } };", None),
        (
            "const doFoo = function() {
                const doBar = function(bar) {
                    return bar;
                };
            };",
            None,
        ),
        ("function doFoo() { const doBar = function(bar) { return bar; }; }", None),
        ("function doFoo() { const doBar = function(bar) { return bar; }; doBar(); }", None),
        ("const doFoo = () => { const doBar = bar => { return bar; } }", None),
        ("function doFoo(Foo) { function doBar() { return this; } return doBar(); };", None),
        (
            "function doFoo(Foo) { const doBar = () => (function() {return this})(); return doBar(); };",
            None,
        ),
        (
            "function doFoo(Foo) {
                const doBar = () => (function() {return () => this})();
                return doBar();
            };",
            None,
        ),
        (
            "function doFoo(Foo) {
                function doBar() {
                    return arguments;
                }
                return doBar();
            };",
            None,
        ),
        (
            "function doFoo(Foo) {
                const doBar = () => (function() {return arguments})();
                return doBar();
            };",
            None,
        ),
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    return doBar(bar);
                }
                return foo;
            }",
            None,
        ),
        (
            "function doFoo(foo) {
                function doBar(bar) {
                    return bar;
                }
                return doBar;
            }",
            None,
        ),
        ("function doFoo() { function doBar() {} }", None),
        ("function doFoo(foo) { { { function doBar(bar) { return bar; } } } return foo; }", None),
        ("{ { function doBar(bar) { return bar; } } }", None),
        ("for (let foo = 0; foo < 1; foo++) { function doBar(bar) { return bar; } }", None),
        ("function foo() { function bar() {} }", None),
        ("function foo() { async function bar() {} }", None),
        ("function foo() { function* bar() {} }", None),
        ("function foo() { async function* bar() {} }", None),
        ("function foo() { const bar = () => {} }", None),
        // ("const doFoo = () => bar => bar;", None),
        ("function foo() { const bar = async () => {} }", None),
        ("function foo() { async function* baz() {} }", None),
        (
            "useEffect(() => {
                function foo() {
                    function bar() {
                    }
                }
            }, [])",
            None,
        ),
        (
            "(function() {
                function foo() {
                    function bar() {
                    }
                }
            })();",
            None,
        ),
        (
            "process.nextTick(() => {
                function returnsZero() {
                    return true;
                }
                process.exitCode = returnsZero();
            });",
            None,
        ),
        (
            "foo(
                // This is not an IIFE
                function() {
                    function bar() {
                    }
                },
                // This is an IIFE
                (function() {
                    function baz() {
                    }
                })(),
            )",
            None,
        ),
        (
            "// This is an IIFE
            (function() {
                function bar() {
                }
            })(
                // This is not IIFE
                function() {
                    function baz() {
                    }
                },
            )",
            None,
        ),
        (
            "function Foo() {
                const Bar = <div />
                function doBaz() {
                    return 42
                }
                return <div>{ doBaz() }</div>
            }",
            None,
        ),
        (
            "function Foo() {
                function Bar () {
                    return <div />
                }
                function doBaz() {
                    return 42
                }
                return <div>{ doBaz() }</div>
            }",
            None,
        ),
        (
            "function fn1() {
                function a() {
                    return <JSX a={b()}/>;
                }
                function b() {}
                function c() {}
            }
            function fn2() {
                function foo() {}
            }",
            None,
        ),
        (
            "const outer = () => { function inner() {} }",
            Some(serde_json::json!([{ "checkArrowFunctions": false }])),
        ),
        ("function foo() { function bar() {} }", None),
        ("function foo() { async function bar() {} }", None),
        ("function foo() { function * bar() {} }", None),
        ("function foo() { async function * bar() {} }", None),
        ("function foo() { const bar = () => {} }", None),
        // ("const doFoo = () => bar => bar;", None),
        ("function foo() { const bar = async () => {} }", None),
        ("function doFoo() { const doBar = function(bar) { return bar; }; }", None),
        ("function outer() { const inner = function inner() {}; }", None),
        (
            "export namespace Foo { export function outer() { const inner = function inner() {}; } }",
            None,
        ),
        (
            "jest.mock('@kbn/i18n-react', () => { return { I18nProvider: function MockI18nProvider() { }, }; });",
            None,
        ),
    ];

    Tester::new(ConsistentFunctionScoping::NAME, ConsistentFunctionScoping::PLUGIN, pass, fail)
        .test_and_snapshot();
}
