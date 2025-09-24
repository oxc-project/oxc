use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, AwaitExpression, ForOfStatement, Function, FunctionType,
        MethodDefinition, ObjectProperty, PropertyKey,
    },
};
use oxc_ast_visit::{Visit, walk::walk_for_of_statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct RequireAwait;

fn require_await_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Async function has no 'await' expression.")
        .with_help("Consider removing the 'async' keyword.")
        .with_label(span)
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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// async function foo() {
    ///     doSomething();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// async function foo() {
    ///    await doSomething();
    /// }
    /// ```
    RequireAwait,
    eslint,
    pedantic,
    fix_dangerous
);

impl Rule for RequireAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::FunctionBody(body) = node.kind() else {
            return;
        };
        if body.is_empty() {
            return;
        }
        let parent = ctx.nodes().parent_node(node.id());

        match parent.kind() {
            AstKind::Function(func) => {
                if func.r#async && !func.generator {
                    let mut finder = AwaitFinder { found: false };
                    finder.visit_function_body(body);
                    if !finder.found {
                        if matches!(func.r#type, FunctionType::FunctionDeclaration) {
                            let need_delete_span = get_delete_span(ctx, func.span.start);
                            ctx.diagnostic_with_dangerous_fix(
                                require_await_diagnostic(
                                    func.id.as_ref().map_or(func.span, |ident| ident.span),
                                ),
                                |fixer| fixer.delete_range(need_delete_span),
                            );
                        } else {
                            let parent_parent_node = ctx.nodes().parent_kind(parent.id());
                            if let AstKind::ObjectProperty(ObjectProperty { span, key, .. })
                            | AstKind::MethodDefinition(MethodDefinition {
                                span, key, ..
                            }) = parent_parent_node
                            {
                                let need_delete_span = get_delete_span(
                                    ctx,
                                    if matches!(parent_parent_node, AstKind::ObjectProperty(x) if !x.method)
                                    {
                                        func.span.start
                                    } else {
                                        span.start
                                    },
                                );
                                let check_span = if matches!(key, PropertyKey::StaticIdentifier(_))
                                {
                                    key.span()
                                } else {
                                    func.span
                                };
                                ctx.diagnostic_with_dangerous_fix(
                                    require_await_diagnostic(check_span),
                                    |fixer| fixer.delete_range(need_delete_span),
                                );
                            } else {
                                let need_delete_span = get_delete_span(ctx, func.span.start);
                                ctx.diagnostic_with_dangerous_fix(
                                    require_await_diagnostic(
                                        func.id.as_ref().map_or(func.span, |ident| ident.span),
                                    ),
                                    |fixer| fixer.delete_range(need_delete_span),
                                );
                            }
                        }
                    }
                }
            }
            AstKind::ArrowFunctionExpression(func) => {
                if func.r#async {
                    let mut finder = AwaitFinder { found: false };
                    finder.visit_function_body(body);
                    if !finder.found {
                        let need_delete_span = get_delete_span(ctx, func.span.start);
                        ctx.diagnostic_with_dangerous_fix(
                            require_await_diagnostic(func.span),
                            |fixer| fixer.delete_range(need_delete_span),
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

#[expect(clippy::cast_possible_truncation)]
fn get_delete_span(ctx: &LintContext, start: u32) -> Span {
    let source_text = ctx.source_text();
    let source_from_start = &source_text[(start as usize)..];

    // Find the position of "async" keyword from the start position
    let async_pos = source_from_start.find("async").unwrap_or(0);
    let async_start = start + async_pos as u32;
    let async_end = async_start + 5;
    let async_key_span = Span::new(async_start, async_end);

    // debug assertions
    #[cfg(debug_assertions)]
    {
        assert!(
            async_key_span.source_text(ctx.source_text()) == "async",
            "Expected 'async' at span {async_key_span:?}, found: {:?}",
            async_key_span.source_text(ctx.source_text())
        );
    }

    let mut offset: u32 = 0;
    for c in ctx.source_text()[(async_end as usize)..].chars() {
        if !c.is_whitespace() {
            break;
        }
        offset += c.len_utf8() as u32;
    }
    async_key_span.expand_right(offset)
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

    let fix = vec![
        ("const a =async() => { let v = 3 ;}", "const a =() => { let v = 3 ;}"),
        ("const a = async () => { let v = 3 }", "const a = () => { let v = 3 }"),
        ("async function foo() { doSomething() }", "function foo() { doSomething() }"),
        ("(async function() { doSomething() })", "(function() { doSomething() })"),
        ("async () => { doSomething() }", "() => { doSomething() }"),
        ("async () => doSomething()", "() => doSomething()"),
        ("({ async foo() { doSomething() } })", "({ foo() { doSomething() } })"),
        ("class A { async foo() { doSomething() } }", "class A { foo() { doSomething() } }"),
        ("(class { async ''() { doSomething() } })", "(class { ''() { doSomething() } })"),
        (
            "async function foo() { async () => { await doSomething() } }",
            "function foo() { async () => { await doSomething() } }",
        ),
        (
            "async function foo() { await (async () => { doSomething() }) }",
            "async function foo() { await (() => { doSomething() }) }",
        ),
        (
            "async /** comments */ function name() { doSomething() }",
            "/** comments */ function name() { doSomething() }",
        ),
        ("async          function foo() { doSomething() }", "function foo() { doSomething() }"),
        (
            "async     /** cc */     function foo() { doSomething() }",
            "/** cc */     function foo() { doSomething() }",
        ),
        (
            "let a = { c: async () => { let c }, t:async()=>{ let r } }",
            "let a = { c: () => { let c }, t:()=>{ let r } }",
        ),
        ("async function O(){r}", "function O(){r}"),
        ("s={expoí:async function(){{}}}", "s={expoí:function(){{}}}"),
        ("class foo { private async bar() { x() } }", "class foo { private bar() { x() } }"),
    ];

    Tester::new(RequireAwait::NAME, RequireAwait::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
