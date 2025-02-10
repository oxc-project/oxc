use oxc_ast::{
    ast::{ArrowFunctionExpression, AwaitExpression, ForOfStatement, Function, PropertyKey},
    visit::walk::walk_for_of_statement,
    AstKind, Visit,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct RequireAwait;

fn require_await_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Async function has no 'await' expression.").with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow async functions which have no `await` expression.
    ///
    /// ### Why is this bad?
    ///
    /// Asynchronous functions in JavaScript behave differently than other
    /// functions in two important ways:
    ///
    /// 1. The return value is always a `Promise`.
    /// 2. You can use the `await` operator inside of them.
    ///
    /// The primary reason to use asynchronous functions is typically to use the
    /// await operator, such as this:
    ///
    /// ```js
    /// async function fetchData(processDataItem) {
    ///     const response = await fetch(DATA_URL);
    ///     const data = await response.json();
    ///
    ///     return data.map(processDataItem);
    /// }
    /// ```
    /// Asynchronous functions that don’t use await might not need to be
    /// asynchronous functions and could be the unintentional result of
    /// refactoring.
    ///
    /// Note: this rule ignores async generator functions. This is because
    /// generators yield rather than return a value and async generators might
    /// yield all the values of another async generator without ever actually
    /// needing to use await.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```js
    /// async function foo() {
    ///     doSomething();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```js
    /// async function foo() {
    ///    await doSomething();
    /// }
    /// ```
    RequireAwait,
    eslint,
    pedantic,
);

impl Rule for RequireAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::FunctionBody(body) = node.kind() else {
            return;
        };
        if body.is_empty() {
            return;
        }
        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        match parent.kind() {
            AstKind::Function(func) => {
                if func.r#async && !func.generator {
                    let mut finder = AwaitFinder { found: false };
                    finder.visit_function_body(body);
                    if !finder.found {
                        if let Some(AstKind::ObjectProperty(p)) =
                            ctx.nodes().parent_kind(parent.id())
                        {
                            if let PropertyKey::StaticIdentifier(iden) = &p.key {
                                ctx.diagnostic(require_await_diagnostic(iden.span));
                            } else {
                                ctx.diagnostic(require_await_diagnostic(func.span));
                            }
                        } else {
                            ctx.diagnostic(require_await_diagnostic(
                                func.id.as_ref().map_or(func.span, |ident| ident.span),
                            ));
                        }
                    }
                }
            }
            AstKind::ArrowFunctionExpression(func) => {
                if func.r#async {
                    let mut finder = AwaitFinder { found: false };
                    finder.visit_function_body(body);
                    if !finder.found {
                        ctx.diagnostic(require_await_diagnostic(func.span));
                    }
                }
            }
            _ => {}
        }
    }
}

struct AwaitFinder {
    found: bool,
}

impl<'a> Visit<'a> for AwaitFinder {
    fn visit_await_expression(&mut self, _expr: &AwaitExpression) {
        if self.found {
            return;
        }
        self.found = true;
    }

    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement) {
        if stmt.r#await {
            self.found = true;
        } else {
            walk_for_of_statement(self, stmt);
        }
    }

    fn visit_arrow_function_expression(&mut self, _expr: &ArrowFunctionExpression<'a>) {}

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "async function foo() { await doSomething() }",
        "(async function() { await doSomething() })",
        "async () => { await doSomething() }",
        "async () => await doSomething()",
        "({ async foo() { await doSomething() } })",
        "class A { async foo() { await doSomething() } }",
        "(class { async foo() { await doSomething() } })",
        "async function foo() { await (async () => { await doSomething() }) }",
        "async function foo() {}",
        "async () => {}",
        "function foo() { doSomething() }",
        "async function foo() { for await (x of xs); }",
        "await foo()",
        "
        	                for await (let num of asyncIterable) {
        	                    console.log(num);
        	                }
        	            ",
        "async function* run() { yield * anotherAsyncGenerator() }",
        "async function* run() {
        	                await new Promise(resolve => setTimeout(resolve, 100));
        	                yield 'Hello';
        	                console.log('World');
        	            }
        	            ",
        "
        async function foo() {
            {
                await doSomething()
            }
        }
        	            ",
        "async function* run() { }",
        "const foo = async function *(){}",
        r#"const foo = async function *(){ console.log("bar") }"#,
        r#"async function* run() { console.log("bar") }"#,
        "
        const foo = async (
         ): Promise<string> => {
           for (const bar of baz) {
             x(await y(z));
           }
        };
        ",
    ];

    let fail = vec![
        "async function foo() { doSomething() }",
        "(async function() { doSomething() })",
        "async () => { doSomething() }",
        "async () => doSomething()",
        "({ async foo() { doSomething() } })",
        "class A { async foo() { doSomething() } }",
        "(class { async foo() { doSomething() } })",
        "(class { async ''() { doSomething() } })",
        "async function foo() { async () => { await doSomething() } }",
        "async function foo() { await (async () => { doSomething() }) }",
    ];

    Tester::new(RequireAwait::NAME, RequireAwait::PLUGIN, pass, fail).test_and_snapshot();
}
