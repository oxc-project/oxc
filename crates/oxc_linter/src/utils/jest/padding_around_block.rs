use oxc_ast::{
    AstKind,
    ast::{Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::context::LintContext;

use super::get_node_name_vec;

fn missing_padding_before_jest_block_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Missing padding before {name} block"))
        .with_help(format!("Make sure there is an empty new line before the {name} block"))
        .with_label(span)
}

fn missing_padding_after_jest_block_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Missing padding after {name} block"))
        .with_help(format!("Make sure there is an empty new line after the {name} block"))
        .with_label(span)
}

pub fn report_missing_padding_before_jest_block<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
) {
    let scope_node = ctx.nodes().get_node(ctx.scoping().get_node_id(node.scope_id()));
    let prev_statement_span = match scope_node.kind() {
        AstKind::Program(program) => get_statement_span_before_node(node, program.body.as_slice()),
        AstKind::ArrowFunctionExpression(arrow_func_expr) => {
            let Some(body) = arrow_func_expr.get_function_body() else { return };
            get_statement_span_before_node(node, body.statements.as_slice())
        }
        AstKind::Function(function) => {
            let Some(body) = &function.body else {
                return;
            };
            get_statement_span_before_node(node, body.statements.as_slice())
        }
        _ => None,
    };
    let Some(prev_statement_span) = prev_statement_span else {
        return;
    };

    let comments_range = ctx.comments_range(prev_statement_span.end..node.span().start);
    let mut span_between_start = prev_statement_span.end;
    let mut span_between_end = node.span().start;
    let mut next_attached_start = node.span().start;
    for comment in comments_range.rev() {
        let comment_span = comment.span;
        let space_after = ctx.source_range(Span::new(comment_span.end, next_attached_start));
        if space_after.matches('\n').count() > 1 {
            break;
        }
        let space_before = ctx.source_range(Span::new(prev_statement_span.end, comment_span.start));
        if space_before.matches('\n').count() == 0 {
            span_between_start = comment_span.end;
            break;
        }
        span_between_end = comment_span.start;
        next_attached_start = comment_span.start;
    }

    let span_between = Span::new(span_between_start, span_between_end);
    let content = ctx.source_range(span_between);
    if content.matches('\n').count() < 2 {
        ctx.diagnostic_with_fix(
            missing_padding_before_jest_block_diagnostic(
                Span::new(node.span().start, node.span().start),
                name,
            ),
            |fixer| {
                let whitespace_after_last_line =
                    content.rfind('\n').map_or("", |index| content.split_at(index + 1).1);
                fixer.replace(span_between, format!("\n\n{whitespace_after_last_line}"))
            },
        );
    }
}

pub fn report_missing_padding_after_jest_block<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
) {
    let scope_node = ctx.nodes().get_node(ctx.scoping().get_node_id(node.scope_id()));
    let statement_spans = match scope_node.kind() {
        AstKind::Program(program) => {
            get_statement_spans_around_node(node, program.body.as_slice(), name)
        }
        AstKind::ArrowFunctionExpression(arrow_func_expr) => {
            let Some(body) = arrow_func_expr.get_function_body() else { return };
            get_statement_spans_around_node(node, body.statements.as_slice(), name)
        }
        AstKind::Function(function) => {
            let Some(body) = &function.body else {
                return;
            };
            get_statement_spans_around_node(node, body.statements.as_slice(), name)
        }
        _ => None,
    };
    let Some((current_statement_span, next_statement_span)) = statement_spans else {
        return;
    };

    let span_between = Span::new(current_statement_span.end, next_statement_span.start);
    let content = ctx.source_range(span_between);
    if content.matches('\n').count() < 2 {
        ctx.diagnostic_with_fix(
            missing_padding_after_jest_block_diagnostic(
                Span::new(next_statement_span.start, next_statement_span.start),
                name,
            ),
            |fixer| {
                let whitespace_after_last_line =
                    content.rfind('\n').map_or("", |index| content.split_at(index + 1).1);
                fixer.replace(span_between, format!("\n\n{whitespace_after_last_line}"))
            },
        );
    }
}

fn get_statement_span_before_node(node: &AstNode, statements: &[Statement]) -> Option<Span> {
    statements
        .iter()
        .filter_map(|statement| {
            if statement.span().end <= node.span().start { Some(statement.span()) } else { None }
        })
        .next_back()
}

fn get_statement_spans_around_node(
    node: &AstNode,
    statements: &[Statement],
    name: &str,
) -> Option<(Span, Span)> {
    let mut statements_after_node =
        statements.iter().skip_while(|statement| !statement.span().contains_inclusive(node.span()));
    let current_statement_span = statements_after_node.next()?.span();
    let next_statement = statements_after_node.next()?;
    if is_same_named_jest_call_statement(next_statement, name) {
        return None;
    }
    let next_statement_span = next_statement.span();
    Some((current_statement_span, next_statement_span))
}

fn is_same_named_jest_call_statement(statement: &Statement, name: &str) -> bool {
    let Statement::ExpressionStatement(expr_stmt) = statement else {
        return false;
    };
    let Expression::CallExpression(call_expr) = &expr_stmt.expression else {
        return false;
    };
    get_node_name_vec(&call_expr.callee).first().is_some_and(|callee_name| callee_name == name)
}
