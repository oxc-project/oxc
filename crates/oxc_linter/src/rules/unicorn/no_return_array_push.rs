use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, ReturnStatement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util::is_method_call, context::LintContext, rule::Rule};

const IGNORED_PUSH_CALLEES: &[&[&str]] = &[
    &["stream", "push"],
    &["this", "push"],
    &["this", "stream", "push"],
    &["process", "stdin", "push"],
    &["process", "stdout", "push"],
    &["process", "stderr", "push"],
];

fn no_return_array_push_diagnostic(span: Span, method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not use the return value of `.{method}(…)`."))
        .with_help(format!("Separate the `{method}()` call from `return`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoReturnArrayPush;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using the return value of `Array#push()` and `Array#unshift()`.
    ///
    /// ### Why is this bad?
    ///
    /// `Array#push()` and `Array#unshift()` return the new length of the array, not the added
    /// value or the array. Returning or assigning that length is almost always a mistake.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function add(item) {
    ///     return items.push(item);
    /// }
    ///
    /// const add = item => items.push(item);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function add(item) {
    ///     items.push(item);
    ///     return;
    /// }
    ///
    /// const add = item => {
    ///     items.push(item);
    /// };
    /// ```
    NoReturnArrayPush,
    unicorn,
    pedantic,
    suggestion,
    version = "next",
    short_description = "Disallow using the return value of `Array#push()` and `Array#unshift()`.",
);

impl Rule for NoReturnArrayPush {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, None, Some(&["push", "unshift"]), Some(1), None) {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        if member_expr.is_computed() {
            return;
        }

        let Some(method) = member_expr.static_property_name() else {
            return;
        };

        if method == "push" && is_ignored_push_callee(&call_expr.callee) {
            return;
        }

        if is_bare_expression_statement(node, ctx) {
            return;
        }

        let Some((property_span, _)) = member_expr.static_property_info() else {
            return;
        };
        let diagnostic = no_return_array_push_diagnostic(property_span, method);

        if let Some(return_statement) = get_direct_return_statement(node, ctx)
            && can_apply_return_suggestion(return_statement, call_expr, node, ctx)
        {
            let src = ctx.source_text();
            ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                let call_text = call_expr.span().source_text(src);
                let semicolon =
                    needs_leading_semicolon(return_statement.span.start, src, call_text);
                fixer.replace(return_statement.span, format!("{semicolon}{call_text}; return;"))
            });
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}

fn is_ignored_push_callee(callee: &Expression) -> bool {
    IGNORED_PUSH_CALLEES.iter().any(|path| matches_static_member_path(callee, path))
}

fn matches_static_member_path(expression: &Expression, path: &[&str]) -> bool {
    if path.len() < 2 {
        return false;
    }

    let Some(member) = expression.get_member_expr() else {
        return false;
    };

    if member.is_computed() || member.static_property_name() != path.last().copied() {
        return false;
    }

    let mut current = member.object().get_inner_expression();

    for name in &path[1..path.len() - 1] {
        let Expression::StaticMemberExpression(static_member) = current else {
            return false;
        };
        if static_member.property.name.as_str() != *name {
            return false;
        }
        current = static_member.object.get_inner_expression();
    }

    match current {
        Expression::Identifier(ident) => ident.name.as_str() == path[0],
        Expression::ThisExpression(_) => path[0] == "this",
        _ => false,
    }
}

fn is_bare_expression_statement<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let outer = outer_result_parent(node, ctx);
    if !matches!(outer.kind(), AstKind::ExpressionStatement(_)) {
        return false;
    }

    !is_arrow_expression_body_statement(outer, ctx)
}

// NOTE: Oxc stores concise arrow bodies as a single ExpressionStatement inside FunctionBody, not as a direct expression child like ESTree.
fn is_arrow_expression_body_statement<'a>(
    expression_statement: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let nodes = ctx.nodes();
    let body_node = nodes.parent_node(expression_statement.id());
    let AstKind::FunctionBody(function_body) = body_node.kind() else {
        return false;
    };

    let arrow_node = nodes.parent_node(body_node.id());
    let AstKind::ArrowFunctionExpression(arrow) = arrow_node.kind() else {
        return false;
    };

    arrow.expression
        && function_body.statements.len() == 1
        && function_body.statements[0].span() == expression_statement.span()
}

fn outer_result_parent<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> &'b AstNode<'a> {
    let nodes = ctx.nodes();
    let mut current = node;

    loop {
        let parent = nodes.parent_node(current.id());
        if parent.id() == current.id() {
            return current;
        }

        let transparent = match parent.kind() {
            AstKind::ParenthesizedExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::TSInstantiationExpression(_) => true,
            AstKind::ChainExpression(chain) => {
                chain.expression.span().contains_inclusive(current.span())
            }
            _ => false,
        };

        if transparent {
            current = parent;
            continue;
        }

        return parent;
    }
}

fn get_direct_return_statement<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a ReturnStatement<'a>> {
    let parent = ctx.nodes().parent_node(node.id());
    let AstKind::ReturnStatement(return_statement) = parent.kind() else {
        return None;
    };

    let Expression::CallExpression(call_expression) =
        return_statement.argument.as_ref()?.without_parentheses()
    else {
        return None;
    };

    if call_expression.span == node.span() { Some(return_statement) } else { None }
}

fn can_apply_return_suggestion<'a>(
    return_statement: &ReturnStatement<'a>,
    call_expr: &CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let return_node = ctx.nodes().parent_node(node.id());
    let container = ctx.nodes().parent_node(return_node.id());
    if container.id() == return_node.id() {
        return false;
    }

    if !matches!(container.kind(), AstKind::BlockStatement(_) | AstKind::FunctionBody(_)) {
        return false;
    }

    let return_comments =
        ctx.comments_range(return_statement.span.start..return_statement.span.end).count();
    let call_comments = ctx.comments_range(call_expr.span.start..call_expr.span.end).count();
    return_comments == call_comments
}

fn needs_leading_semicolon(position: u32, src: &str, text: &str) -> &'static str {
    let before = src.get(..position as usize).unwrap_or("").trim_end();
    if before.is_empty() {
        return "";
    }

    let last = before.chars().last().unwrap();
    if matches!(last, ';' | '{' | '}') {
        return "";
    }

    let first = text.trim_start().chars().next().unwrap_or('\0');
    if matches!(last, ')' | ']')
        || matches!(first, '[' | '+' | '-' | '/' | '!' | '`')
        || first == '('
    {
        ";"
    } else {
        ""
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function foo() { array.push(value); return; }",
        "function foo() { array.unshift(value); return; }",
        "function foo() { return array.length + 1; }",
        "function foo() { return array.push(); }",
        "function foo() { return array.unshift(); }",
        "function foo() { return stream.push(chunk); }",
        "function foo() { return this.push(chunk); }",
        "function foo() { return this.stream.push(chunk); }",
        "function foo() { return process.stdin.push(chunk); }",
        "function foo() { return process.stdout.push(chunk); }",
        "function foo() { return process.stderr.push(chunk); }",
        "function foo() { return stream?.push(chunk); }",
        "function foo() { return this?.push(chunk); }",
        "function foo() { return process.stdin?.push(chunk); }",
        "function foo() { return array['push'](value); }",
        "function foo() { return array['unshift'](value); }",
        "function foo() { return array[push](value); }",
        "function foo() { return array[unshift](value); }",
        "function foo() { return push(value); }",
        "function foo() { return unshift(value); }",
        "array?.push(value);",
        "array?.unshift(value);",
        "array.push(value) as number;",
        "array.unshift(value) as number;",
        "array.push(value)!;",
        "array.unshift(value)!;",
        "array.push(value) satisfies number;",
        "array.unshift(value) satisfies number;",
    ];

    let fail = vec![
        "const length = array.push(value);",
        "const length = array.push(value, other);",
        "const length = array.unshift(value);",
        "const length = array.unshift(value, other);",
        "const length = stream.unshift(chunk);",
        "console.log(array.push(value));",
        "console.log(array.unshift(value));",
        "void array.push(value);",
        "void array.unshift(value);",
        "condition && array.push(value);",
        "condition && array.unshift(value);",
        "array.push(value) && sideEffect();",
        "array.unshift(value) && sideEffect();",
        "function foo() { return array.push(value); }",
        "function foo() { return /* keep */ array.push(value); }",
        "function foo() { return array.push(/* keep */ value); }",
        "function foo() { return array.unshift(value); }",
        "function foo() { return (array.push(value)); }",
        "function foo() { return (array.unshift(value)); }",
        "function foo() { return array?.push(value); }",
        "function foo() { return array?.unshift(value); }",
        "function foo() { return array.push?.(value); }",
        "function foo() { return array.unshift?.(value); }",
        "const foo = value => array.push(value);",
        "const foo = value => array.unshift(value);",
        "function foo() { if (condition) return array.push(value); }",
        "function foo() { return condition && array.push(value); }",
        "function foo() { return condition ? array.push(value) : value; }",
        "function foo() { return condition ? value : array.push(value); }",
        "function foo() { return (sideEffect(), array.push(value)); }",
        "function foo() { foo(); return [array].push(value); }",
        "function foo() { return array.push(value) as number; }",
        "function foo() { return array.unshift(value) as number; }",
        "function foo() { return array.push(value)!; }",
        "function foo() { return array.unshift(value)!; }",
        "function foo() { return array.push(value) satisfies number; }",
        "function foo() { return array.unshift(value) satisfies number; }",
        "const foo = (value: string) => array.push(value);",
        "const foo = (value: string) => array.unshift(value);",
    ];

    let fix = vec![
        (
            "function foo() { return array.push(value); }",
            "function foo() { array.push(value); return; }",
        ),
        (
            "function foo() { return array.unshift(value); }",
            "function foo() { array.unshift(value); return; }",
        ),
        (
            "function foo() { foo(); return [array].push(value); }",
            "function foo() { foo(); [array].push(value); return; }",
        ),
        (
            "function foo() { return array.push(value, other); }",
            "function foo() { array.push(value, other); return; }",
        ),
        (
            "function foo() { return array.unshift(value, other); }",
            "function foo() { array.unshift(value, other); return; }",
        ),
        (
            "function foo() { return array.push(/* keep */ value); }",
            "function foo() { array.push(/* keep */ value); return; }",
        ),
    ];

    Tester::new(NoReturnArrayPush::NAME, NoReturnArrayPush::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
