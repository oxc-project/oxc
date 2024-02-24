use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, fixer::Fix, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-return-await): Redundant use of `await` on a return value.")]
#[diagnostic(severity(warning), help("Remove redundant `await`."))]
struct NoReturnAwaitDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoReturnAwait;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow unnecessary return await
    ///
    /// ### Why is this bad?
    /// This rule aims to prevent a likely common performance hazard due to a lack of understanding of the semantics of async function.
    /// https://eslint.org/docs/latest/rules/no-return-await
    ///
    /// ### Example
    /// ```javascript
    /// async function foo() {
    ///   return await bar();
    /// }
    /// ```
    NoReturnAwait,
    pedantic
);

impl Rule for NoReturnAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::AwaitExpression(await_expr) = node.kind() {
            if is_in_tail_call_position(node, ctx) && !has_error_handler(node, ctx) {
                let start = await_expr.span.start;
                let end = start + 5;
                let await_keyword_span = Span::new(start, end);
                ctx.diagnostic_with_fix(NoReturnAwaitDiagnostic(await_keyword_span), || {
                    Fix::new("", await_keyword_span)
                });
            }
        }
    }
}

fn is_in_tail_call_position<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if let Some(parent) = ctx.nodes().parent_node(node.id()) {
        let parent_kind = parent.kind();
        match parent_kind {
            AstKind::ArrowFunctionExpression(arrow_expr) => {
                // async () => { await b(); })
                // `epxression` property is false
                return arrow_expr.expression;
            }
            AstKind::ReturnStatement(_) => {
                return !has_error_handler(node, ctx);
            }
            AstKind::ConditionalExpression(cond_expr) => {
                if (cond_expr.consequent.span() == node.kind().span())
                    || (cond_expr.alternate.span() == node.kind().span())
                {
                    return is_in_tail_call_position(parent, ctx);
                }
            }
            AstKind::LogicalExpression(logic_expr) => {
                if logic_expr.right.span() == node.kind().span() {
                    return is_in_tail_call_position(parent, ctx);
                }
            }
            AstKind::SequenceExpression(seq_expr) => {
                if let Some(seq_expr_last) = seq_expr.expressions.last() {
                    if seq_expr_last.span() == node.kind().span() {
                        return is_in_tail_call_position(parent, ctx);
                    }
                }
            }
            // `return (await b())`
            AstKind::ParenthesizedExpression(paren_expr) => {
                if paren_expr.expression.span() == node.kind().span() {
                    return is_in_tail_call_position(parent, ctx);
                }
            }
            // async () => await bar()
            AstKind::ExpressionStatement(expr_stat) => {
                // expression state in the last line of a `function_body`
                if expr_stat.expression.span() == node.kind().span() {
                    return is_in_tail_call_position(parent, ctx);
                }
            }
            // last statement in `func_body`
            AstKind::FunctionBody(func_body) => {
                if let Some(func_body_stat_last) = func_body.statements.last() {
                    if func_body_stat_last.span() == node.kind().span() {
                        return is_in_tail_call_position(parent, ctx);
                    }
                }
            }
            _ => {
                return false;
            }
        }
    }
    false
}

fn has_error_handler<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let mut current_node = node;
    loop {
        if let Some(parent_node) = ctx.nodes().parent_node(current_node.id()) {
            let parent_node_kind = parent_node.kind();
            if matches!(parent_node_kind, AstKind::Program(_)) {
                break;
            }

            if parent_node_kind.is_function_like() {
                break;
            }

            if let AstKind::TryStatement(try_stat) = parent_node_kind {
                let current_node_span = current_node.kind().span();
                // try statement must have a `catch` or `finally`
                if try_stat.block.span == current_node_span {
                    return true;
                }

                // return await in `catch clause` with `finally` would be passed
                if let Some(catch_clause) = &try_stat.handler {
                    if catch_clause.span == current_node_span && try_stat.finalizer.is_some() {
                        return true;
                    }
                }
            }

            current_node = parent_node;
        }
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("\nasync () => { await b(); }", None),
        ("\nasync function foo() {\n\tawait bar(); return;\n}\n", None),
        ("\nasync function foo() {\n\tconst x = await bar(); return x;\n}\n", None),
        ("\nasync () => { return bar(); }\n", None),
        ("\nasync () => bar()\n", None),
        (
            "\nasync function foo() {\nif (a) {\n\t\tif (b) {\n\t\t\treturn bar();\n\t\t}\n\t}\n}\n",
            None,
        ),
        ("\nasync () => {\nif (a) {\n\t\tif (b) {\n\t\t\treturn bar();\n\t\t}\n\t}\n}\n", None),
        ("\nasync function foo() {\n\treturn (await bar() && a);\n}\n", None),
        ("\nasync function foo() {\n\treturn (await bar() || a);\n}\n", None),
        ("\nasync function foo() {\n\treturn (a && await baz() && b);\n}\n", None),
        ("\nasync function foo() {\n\treturn (await bar(), a);\n}\n", None),
        ("\nasync function foo() {\n\treturn (await baz(), await bar(), a);\n}\n", None),
        ("\nasync function foo() {\n\treturn (a, b, (await bar(), c));\n}\n", None),
        ("\nasync function foo() {\n\treturn (await bar() ? a : b);\n}\n", None),
        ("\nasync function foo() {\n\treturn ((a && await bar()) ? b : c);\n}\n", None),
        ("\nasync function foo() {\n\treturn (baz() ? (await bar(), a) : b);\n}\n", None),
        ("\nasync function foo() {\n\treturn (baz() ? (await bar() && a) : b);\n}\n", None),
        ("\nasync function foo() {\n\treturn (baz() ? a : (await bar(), b));\n}\n", None),
        ("\nasync function foo() {\n\treturn (baz() ? a : (await bar() && b));\n}\n", None),
        ("\nasync () => (await bar(), a)\n", None),
        ("\nasync () => (await bar() && a)\n", None),
        ("\nasync () => (await bar() || a)\n", None),
        ("\nasync () => (a && await bar() && b)\n", None),
        ("\nasync () => (await baz(), await bar(), a)\n", None),
        ("\nasync () => (a, b, (await bar(), c))\n", None),
        ("\nasync () => (await bar() ? a : b)\n", None),
        ("\nasync () => ((a && await bar()) ? b : c)\n", None),
        ("\nasync () => (baz() ? (await bar(), a) : b)\n", None),
        ("\nasync () => (baz() ? (await bar() && a) : b)\n", None),
        ("\nasync () => (baz() ? a : (await bar(), b))\n", None),
        ("\nasync () => (baz() ? a : (await bar() && b))\n", None),
        (
            "\n          async function foo() {\n            try {\n              return await bar();\n            } catch (e) {\n              baz();\n            }\n          }\n        ",
            None,
        ),
        (
            "\n          async function foo() {\n            try {\n              return await bar();\n            } finally {\n              baz();\n            }\n          }\n        ",
            None,
        ),
        (
            "\n          async function foo() {\n            try {}\n            catch (e) {\n              return await bar();\n            } finally {\n              baz();\n            }\n          }\n        ",
            None,
        ),
        (
            "\n          async function foo() {\n            try {\n              try {}\n              finally {\n                return await bar();\n              }\n            } finally {\n              baz();\n            }\n          }\n        ",
            None,
        ),
        (
            "\n          async function foo() {\n            try {\n              try {}\n              catch (e) {\n                return await bar();\n              }\n            } finally {\n              baz();\n            }\n          }\n        ",
            None,
        ),
        (
            "\n          async function foo() {\n            try {\n              return (a, await bar());\n            } catch (e) {\n              baz();\n            }\n          }\n        ",
            None,
        ),
        (
            "\n          async function foo() {\n            try {\n              return (qux() ? await bar() : b);\n            } catch (e) {\n              baz();\n            }\n          }\n        ",
            None,
        ),
        (
            "\n          async function foo() {\n            try {\n              return (a && await bar());\n            } catch (e) {\n              baz();\n            }\n          }\n        ",
            None,
        ),
    ];

    let fail = vec![
        ("\nasync function foo() {\n\treturn await bar();\n}\n", None),
        ("\nasync function foo() {\n\treturn await(bar());\n}\n", None),
        ("\nasync function foo() {\n\treturn (a, await bar());\n}\n", None),
        ("\nasync function foo() {\n\treturn (a, b, await bar());\n}\n", None),
        ("\nasync function foo() {\n\treturn (a && await bar());\n}\n", None),
        ("\nasync function foo() {\n\treturn (a && b && await bar());\n}\n", None),
        ("\nasync function foo() {\n\treturn (a || await bar());\n}\n", None),
        ("\nasync function foo() {\n\treturn (a, b, (c, d, await bar()));\n}\n", None),
        ("\nasync function foo() {\n\treturn (a, b, (c && await bar()));\n}\n", None),
        ("\nasync function foo() {\n\treturn (await baz(), b, await bar());\n}\n", None),
        ("\nasync function foo() {\n\treturn (baz() ? await bar() : b);\n}\n", None),
        ("\nasync function foo() {\n\treturn (baz() ? a : await bar());\n}\n", None),
        ("\nasync function foo() {\n\treturn (baz() ? (a, await bar()) : b);\n}\n", None),
        ("\nasync function foo() {\n\treturn (baz() ? a : (b, await bar()));\n}\n", None),
        ("\nasync function foo() {\n\treturn (baz() ? (a && await bar()) : b);\n}\n", None),
        ("\nasync function foo() {\n\treturn (baz() ? a : (b && await bar()));\n}\n", None),
        ("\nasync () => { return await bar(); }\n", None),
        ("\nasync () => await bar()\n", None),
        ("\nasync () => (a, b, await bar())\n", None),
        ("\nasync () => (a && await bar())\n", None),
        ("\nasync () => (baz() ? await bar() : b)\n", None),
        ("\nasync () => (baz() ? a : (b, await bar()))\n", None),
        ("\nasync () => (baz() ? a : (b && await bar()))\n", None),
        ("\nasync function foo() {\nif (a) {\n\t\tif (b) {\n\t\t\treturn await bar();\n\t\t}\n\t}\n}\n", None),
        ("\nasync () => {\nif (a) {\n\t\tif (b) {\n\t\t\treturn await bar();\n\t\t}\n\t}\n}\n", None),
        ("\nasync function foo() { try {}\nfinally {\nreturn await bar();\n}\n}\n", None),
        ("\nasync function foo() {\ntry {}\ncatch (e) {\nreturn await bar();\n}\n}\n", None),
        ("\ntry {\nasync function foo() {\nreturn await bar();\n}\n} catch (e) {}\n", None),
        ("\ntry {\nasync () => await bar();\n} catch (e) {}\n", None),
        (
            "\nasync function foo() {\ntry {}\ncatch (e) {\ntry {}\ncatch (e) {\n return await bar();\n}\n}\n}\n",
            None,
        ),
        (
            "\nasync function foo() {\nreturn await new Promise(resolve => {\nresolve(5);\n});\n}\n",
            None,
        ),
        ("\nasync () => {\nreturn await (\nfoo()\n)\n};\n", None),
        ("\nasync function foo() {\nreturn await // Test\n5;\n}\n", None),
      ];

    Tester::new(NoReturnAwait::NAME, pass, fail).test_and_snapshot();
}
