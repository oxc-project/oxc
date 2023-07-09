use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Span, GetSpan};

use crate::{context::LintContext, rule::Rule, AstNode};

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
    correctness
);

impl Rule for NoReturnAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::AwaitExpression(await_expr) = node.kind() {
            if is_in_tail_call_position(node, ctx) && !has_error_handler(node, ctx) {
                let start = await_expr.span.start;
                let end = start + 5;
                ctx.diagnostic(NoReturnAwaitDiagnostic(Span::new(start, end)));
            }
        }
    }
}

fn is_in_tail_call_position<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if let Some(parent) = ctx.nodes().parent_node(node.id()) {
        let parent_kind = parent.kind();
        match parent_kind {
            AstKind::ArrowExpression(_) => {
                return true;
            }
            AstKind::ReturnStatement(_) => {
                return !has_error_handler(node, ctx);
            }
            AstKind::ConditionalExpression(cond_expr) => {
                if (cond_expr.consequent.span() == node.kind().span())
                    || (cond_expr.alternate.span() == node.kind().span()) {
                    return is_in_tail_call_position(parent, ctx);
                }
            },
            AstKind::LogicalExpression(logic_expr) => {
                if logic_expr.right.span() == node.kind().span() {
                    return is_in_tail_call_position(parent, ctx);
                }
            },
            AstKind::SequenceExpression(seq_expr) => {
                if let Some(seq_expr_last) = seq_expr.expressions.last() 
                    && seq_expr_last.span() == node.kind().span(){
                        return is_in_tail_call_position(parent, ctx);
                }
            },
            // `return (await b())`
            AstKind::ParenthesizedExpression(paren_expr) => {                    
                if paren_expr.expression.span() == node.kind().span() {
                    return is_in_tail_call_position(parent, ctx);
                }
            },
            // async () => await bar()
            AstKind::ExpressionStatement(expr_stat) => {
                // expression state in the last line of a `function_body`
                if expr_stat.expression.span() == node.kind().span() {
                    if let Some(grand_parent) = ctx.nodes().parent_node(parent.id()) {
                        if let AstKind::FunctionBody(func_body) = grand_parent.kind() {
                            if let Some(func_body_stat_last) = func_body.statements.last() {
                                return func_body_stat_last.span() == node.kind().span();
                            }
                        }
                    }
                }
            },
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
            if matches!(parent_node.kind(), AstKind::Program(_)) {
                break;
            }

            if parent_node.kind().is_function_like() {
                break;
            }

            if let AstKind::TryStatement(try_stat) = parent_node.kind() {
                if try_stat.block.span == node.kind().span()
                    && try_stat.finalizer.is_some() {
                    return true;
                }

                if let Some(catch_clause) = &try_stat.handler {
                    if catch_clause.span == node.kind().span() && try_stat.finalizer.is_some() {
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
            "\nasync function foo() {\ntry {\nreturn await bar();\n} catch (e) {\nbaz();\n}\n}\n",
            None,
        ),
        ("\nasync function foo() {\ntry {\nreturn await bar();\n} finally {\nbaz();\n}\n}\n", None),
        (
            "\nasync function foo() {\ntry {}\ncatch (e) {\nreturn await bar();\n} finally {\nbaz();\n}\n}\n",
            None,
        ),
        (
            "\nasync function foo() {\ntry {\ntry {}\nfinally {\nreturn await bar();\n}\n} finally {\nbaz();\n}\n}\n",
            None,
        ),
        (
            "\nasync function foo() {\ntry {\ntry {}\ncatch (e) {\nreturn await bar();\n}\n} finally {\nbaz();\n}\n}\n",
            None,
        ),
        (
            "\nasync function foo() {\ntry {\nreturn (a, await bar());\n} catch (e) {\nbaz();\n}\n}\n",
            None,
        ),
        (
            "\nasync function foo() {\ntry {\nreturn (qux() ? await bar() : b);\n} catch (e) {\nbaz();\n}\n}\n",
            None,
        ),
        (
            "\nasync function foo() {\ntry {\nreturn (a && await bar());\n} catch (e) {\nbaz();\n}\n}\n",
            None,
        ),
    ];

    let fail = vec![
        // ("\nasync function foo() {\n\treturn await bar();\n}\n", None),
        // ("\nasync function foo() {\n\treturn await(bar());\n}\n", None),
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
        (
            "\nasync function foo() {\nif (a) {\n\t\tif (b) {\n\t\t\treturn await bar();\n\t\t}\n\t}\n}\n",
            None,
        ),
        (
            "\nasync () => {\nif (a) {\n\t\tif (b) {\n\t\t\treturn await bar();\n\t\t}\n\t}\n}\n",
            None,
        ),
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
