use oxc_ast::{
    AstKind,
    ast::{Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::context::LintContext;

fn padding_around_jest_block_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Missing padding before {name} block"))
        .with_help(format!("Make sure there is an empty new line before the {name} block"))
        .with_label(span)
}

fn padding_after_jest_block_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
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
            padding_around_jest_block_diagnostic(
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

/// Counterpart of [`report_missing_padding_before_jest_block`] for the other side of the block.
///
/// The rules that use these enforce padding *around* a block, so a block that opens a scope — with
/// nothing before it but a statement after it — still needs a blank line separating it from what
/// follows.
pub fn report_missing_padding_after_jest_block<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
) {
    let scope_node = ctx.nodes().get_node(ctx.scoping().get_node_id(node.scope_id()));
    let spans = match scope_node.kind() {
        AstKind::Program(program) => get_statement_spans_after_node(node, program.body.as_slice()),
        AstKind::ArrowFunctionExpression(arrow_func_expr) => {
            let Some(body) = arrow_func_expr.get_function_body() else { return };
            get_statement_spans_after_node(node, body.statements.as_slice())
        }
        AstKind::Function(function) => {
            let Some(body) = &function.body else {
                return;
            };
            get_statement_spans_after_node(node, body.statements.as_slice())
        }
        _ => None,
    };
    let Some((own_statement_span, next_statement_span, next_statement)) = spans else {
        return;
    };

    // When the next statement is another block of the same kind, it reports this very gap through
    // its own "before" check. Reporting from both sides would flag one missing blank line twice.
    if is_call_to(next_statement, name) {
        return;
    }

    let comments_range = ctx.comments_range(own_statement_span.end..next_statement_span.start);
    let mut span_between_start = own_statement_span.end;
    let mut span_between_end = next_statement_span.start;
    let mut prev_attached_end = own_statement_span.end;
    for comment in comments_range {
        let comment_span = comment.span;
        // A comment on the same line as the block trails the block itself, so the blank line
        // belongs after it rather than before.
        let space_before = ctx.source_range(Span::new(prev_attached_end, comment_span.start));
        if space_before.matches('\n').count() == 0 {
            span_between_start = comment_span.end;
            prev_attached_end = comment_span.end;
            continue;
        }
        // Otherwise the comment leads the next statement, so the blank line belongs before it.
        let space_after = ctx.source_range(Span::new(comment_span.end, span_between_end));
        if space_after.matches('\n').count() > 1 {
            break;
        }
        span_between_end = comment_span.start;
        break;
    }

    let span_between = Span::new(span_between_start, span_between_end);
    let content = ctx.source_range(span_between);
    if content.matches('\n').count() < 2 {
        ctx.diagnostic_with_fix(
            padding_after_jest_block_diagnostic(
                Span::new(own_statement_span.end, own_statement_span.end),
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

/// The span of the statement containing `node`, paired with the span of the statement that follows
/// it in the same block. `None` when the block is the last statement — nothing to pad against.
fn get_statement_spans_after_node<'a, 'b>(
    node: &AstNode,
    statements: &'b [Statement<'a>],
) -> Option<(Span, Span, &'b Statement<'a>)> {
    let index = statements.iter().position(|statement| {
        let span = statement.span();
        span.start <= node.span().start && node.span().end <= span.end
    })?;
    let next = statements.get(index + 1)?;
    Some((statements[index].span(), next.span(), next))
}

/// Whether `statement` is a bare call to `name`, i.e. another block of the same kind.
fn is_call_to(statement: &Statement, name: &str) -> bool {
    let Statement::ExpressionStatement(expr_statement) = statement else {
        return false;
    };
    let Expression::CallExpression(call_expr) = &expr_statement.expression else {
        return false;
    };
    call_expr.callee.get_identifier_reference().is_some_and(|ident| ident.name == name)
}
