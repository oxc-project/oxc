use oxc_ast::{
    AstKind,
    ast::{Expression, VariableDeclarationKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util::is_method_call, context::LintContext, rule::Rule};

fn prefer_top_level_await_over_promise_chain_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer top-level await over using a promise chain.").with_label(span)
}

fn prefer_top_level_await_over_async_iife_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer top-level await over using an async IIFE.").with_label(span)
}

fn prefer_top_level_await_over_async_function_call_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer top-level await over an async function call.")
        .with_help("Add `await` before the function call.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferTopLevelAwait;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer top-level await over top-level promises and async function calls.
    ///
    /// ### Why is this bad?
    ///
    /// Top-level await is more readable and can prevent unhandled rejections.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// (async () => {
    ///     await run();
    /// })();
    ///
    /// run().catch(error => {
    ///     console.error(error);
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// await run();
    ///
    /// try {
    ///     await run();
    /// } catch (error) {
    ///     console.error(error);
    /// }
    /// ```
    PreferTopLevelAwait,
    unicorn,
    pedantic
);

impl Rule for PreferTopLevelAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if node.scope_id() != ctx.scoping().root_scope_id()
            && ctx.nodes().ancestor_kinds(node.id()).any(|kind| {
                matches!(
                    kind,
                    AstKind::FunctionBody(_)
                        | AstKind::ArrowFunctionExpression(_)
                        | AstKind::ClassBody(_)
                )
            })
        {
            return;
        }

        let parent = ctx.nodes().parent_node(node.id());
        // TODO: remove this block once removing `AstKind::Argument` is complete
        let grand_parent = {
            let p = ctx.nodes().parent_node(parent.id());
            if let AstKind::Argument(_) = p.kind() { ctx.nodes().parent_node(p.id()) } else { p }
        };

        if let AstKind::StaticMemberExpression(member_expr) = parent.kind()
            && member_expr.object.span() == call_expr.span
            && matches!(member_expr.property.name.as_str(), "then" | "catch" | "finally")
            && let AstKind::CallExpression(grand_call_expr) = grand_parent.kind()
            && grand_call_expr.callee.span() == member_expr.span()
        {
            return;
        }

        if let Some(AstKind::AwaitExpression(_)) = ctx
            .nodes()
            .ancestors(node.id())
            .find(|ancestor| {
                !matches!(
                    ancestor.kind(),
                    AstKind::ParenthesizedExpression(_)
                        | AstKind::TSAsExpression(_)
                        | AstKind::TSSatisfiesExpression(_)
                        | AstKind::ChainExpression(_)
                        | AstKind::StaticMemberExpression(_)
                )
            })
            .map(AstNode::kind)
        {
            return;
        }

        if let AstKind::ArrayExpression(_) = parent.kind()
            && let AstKind::CallExpression(grand_call_expr) = grand_parent.kind()
            && is_method_call(
                grand_call_expr,
                Some(&["Promise"]),
                Some(&["all", "allSettled", "any", "race"]),
                Some(1),
                Some(1),
            )
        {
            return;
        }

        if let Expression::StaticMemberExpression(member_expr) = &call_expr.callee
            && matches!(member_expr.property.name.as_str(), "then" | "catch" | "finally")
        {
            ctx.diagnostic(prefer_top_level_await_over_promise_chain_diagnostic(call_expr.span));
            return;
        }

        if match call_expr.callee.get_inner_expression() {
            Expression::FunctionExpression(func) if func.r#async && !func.generator => true,
            Expression::ArrowFunctionExpression(func) if func.r#async => true,
            _ => false,
        } {
            ctx.diagnostic(prefer_top_level_await_over_async_iife_diagnostic(call_expr.span));
            return;
        }

        let Expression::Identifier(ident) = &call_expr.callee else {
            return;
        };

        let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id() else {
            return;
        };

        if ctx.scoping().get_resolved_references(symbol_id).count() > 1 {
            return;
        }

        let declaration = ctx.symbol_declaration(symbol_id);

        match declaration.kind() {
            AstKind::VariableDeclarator(var_decl)
                if var_decl.kind == VariableDeclarationKind::Const =>
            {
                let Some(init) = &var_decl.init else { return };

                if !matches!(init.get_inner_expression(), Expression::ArrowFunctionExpression(func) if func.r#async)
                    && !matches!(init.get_inner_expression(), Expression::FunctionExpression(func) if func.r#async && !func.generator)
                {
                    return;
                }

                ctx.diagnostic(prefer_top_level_await_over_async_function_call_diagnostic(
                    call_expr.span,
                ));
            }
            AstKind::Function(func) if func.r#async && !func.generator => {
                ctx.diagnostic(prefer_top_level_await_over_async_function_call_diagnostic(
                    call_expr.span,
                ));
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        ("a()", None, None, None),
        ("a = async () => {}", None, None, None),
        ("(async function *() {})()", None, None, None),
        (
            "function foo() {
				if (foo) {
					(async () => {})()
				}
			}",
            None,
            None,
            None,
        ),
        ("await (async () => {})()", None, None, None),
        ("foo.then", None, None, None),
        ("await foo.then(bar)", None, None, None),
        ("await foo.then(bar).catch(bar)", None, None, None),
        ("await foo.then?.(bar)", None, None, None),
        ("await foo.then(bar)?.catch(bar)", None, None, None),
        ("await foo.then(bar)?.catch?.(bar)", None, None, None),
        (
            "class Example {
				property = promise.then(bar)
			}",
            None,
            None,
            None,
        ),
        (
            "const Example = class Example {
				property = promise.then(bar)
			}",
            None,
            None,
            None,
        ),
        (
            "class Example {
				static {
					promise.then(bar)
				}
			}",
            None,
            None,
            None,
        ),
        (
            "const Example = class Example {
				static {
					promise.then(bar)
				}
			}",
            None,
            None,
            None,
        ),
        ("foo.then(bar)", None, None, Some(PathBuf::from("'foo.cjS'"))),
        ("foo()", None, None, None),
        ("foo.bar()", None, None, None),
        (
            "function foo() {
				return async () => {};
			}
			foo()();",
            None,
            None,
            None,
        ),
        (
            "const [foo] = [async () => {}];
			foo();",
            None,
            None,
            None,
        ),
        (
            "function foo() {}
			foo();",
            None,
            None,
            None,
        ),
        (
            "async function * foo() {}
			foo();",
            None,
            None,
            None,
        ),
        (
            "var foo = async () => {};
			foo();",
            None,
            None,
            None,
        ),
        (
            "let foo = async () => {};
			foo();",
            None,
            None,
            None,
        ),
        (
            "const foo = 1, bar = async () => {};
			foo();",
            None,
            None,
            None,
        ),
        (
            "async function foo() {}
			const bar = foo;
			bar();",
            None,
            None,
            None,
        ),
        (
            "const program = {async run () {}};
			program.run()",
            None,
            None,
            None,
        ),
        (
            "const program = {async run () {}};
			const {run} = program;
			run()",
            None,
            None,
            None,
        ),
        (
            "const foo = async () => {};
			await foo();",
            None,
            None,
            None,
        ),
        ("for (const statement of statements) { statement() };", None, None, None),
        (
            "const foo = async () => {};
			await Promise.all([
				(async () => {})(),
				/* hole */,
				foo(),
				foo.then(bar),
				foo.catch(bar),
			]);
			await Promise.allSettled([foo()]);
			await Promise?.any([foo()]);
			await Promise.race?.([foo()]);",
            None,
            None,
            None,
        ),
        (
            "const foo = async () => {};
			const promise = Promise.all([
				(async () => {})(),
				foo(),
				foo.then(bar),
				foo.catch(bar),
			]);
			await promise;",
            None,
            None,
            None,
        ),
        ("await foo", None, None, None),
        ("await foo()", None, None, None),
        (
            "try {
				await run()
			} catch {
				process.exit(1)
			}",
            None,
            None,
            None,
        ),
    ];

    let fail = vec![
        ("(async () => {})()", None, None, None),
        ("(async () => {})?.()", None, None, None),
        ("(async function() {})()", None, None, None),
        ("(async function() {}())", None, None, None),
        ("(async function run() {})()", None, None, None),
        ("(async function(c, d) {})(a, b)", None, None, None),
        ("if (foo) (async () => {})()", None, None, None),
        (
            "{
				(async () => {})();
			}",
            None,
            None,
            None,
        ),
        ("a = (async () => {})()", None, None, None),
        ("!async function() {}()", None, None, None),
        ("void async function() {}()", None, None, None),
        ("(async () => {})().catch(foo)", None, None, None),
        ("foo.then(bar)", None, None, None),
        ("foo.then?.(bar)", None, None, None),
        ("foo?.then(bar)", None, None, None),
        ("foo.catch(() => process.exit(1))", None, None, None),
        ("foo.finally(bar)", None, None, None),
        ("foo.then(bar, baz)", None, None, None),
        ("foo.then(bar, baz).finally(qux)", None, None, None),
        ("(foo.then(bar, baz)).finally(qux)", None, None, None),
        ("(async () => {})().catch(() => process.exit(1))", None, None, None),
        ("(async function() {}()).finally(() => {})", None, None, None),
        ("for (const foo of bar) foo.then(bar)", None, None, None),
        ("foo?.then(bar).finally(qux)", None, None, None),
        ("foo.then().toString()", None, None, None),
        ("!foo.then()", None, None, None),
        ("foo.then(bar).then(baz)?.then(qux)", None, None, None),
        ("foo.then(bar).then(baz).then?.(qux)", None, None, None),
        ("foo.then(bar).catch(bar).finally(bar)", None, None, None),
        (
            "const foo = async () => {};
			foo();",
            None,
            None,
            None,
        ),
        (
            "const foo = async () => {};
			foo?.();",
            None,
            None,
            None,
        ),
        (
            "const foo = async () => {};
			foo().then(foo);",
            None,
            None,
            None,
        ),
        (
            "const foo = async function () {}, bar = 1;
			foo(bar);",
            None,
            None,
            None,
        ),
        (
            "foo();
			async function foo() {}",
            None,
            None,
            None,
        ),
        (
            "const foo = async () => {};
			if (true) {
				alert();
			} else {
				foo();
			}",
            None,
            None,
            None,
        ),
    ];

    Tester::new(PreferTopLevelAwait::NAME, PreferTopLevelAwait::PLUGIN, pass, fail)
        .test_and_snapshot();
}
