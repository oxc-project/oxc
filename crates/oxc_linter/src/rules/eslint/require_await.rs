use oxc_ast::{
    ast::{ArrowFunctionExpression, AwaitExpression, ForOfStatement, Function, PropertyKey},
    AstKind, Visit,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct RequireAwait;

fn require_await_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(require-await): Async function has no 'await' expression.")
        .with_labels([span0.into()])
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow async functions which have no await expression.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// async function foo() {
    ///   doSomething();
    /// }
    /// ```
    RequireAwait,
    pedantic,
);

impl Rule for RequireAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::FunctionBody(body) = node.kind() {
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
        }
    }

    fn visit_arrow_expression(&mut self, _expr: &ArrowFunctionExpression<'a>) {}
    fn visit_function(&mut self, _func: &Function<'a>, _flags: Option<ScopeFlags>) {}
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

    Tester::new(RequireAwait::NAME, pass, fail).test_and_snapshot();
}
