use oxc_ast::{AstKind, ast::Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::context::LintContext;

#[derive(Clone, Copy)]
enum PaddingSide {
    Before,
    After,
}

fn padding_diagnostic(side: PaddingSide, span: Span, name: &str) -> OxcDiagnostic {
    let where_word = match side {
        PaddingSide::Before => "before",
        PaddingSide::After => "after",
    };
    OxcDiagnostic::warn(format!("Missing padding {where_word} {name} block"))
        .with_help(format!("Make sure there is an empty new line {where_word} the {name} block"))
        .with_label(span)
}

pub fn report_missing_padding_before_jest_block<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
) {
    let Some(statements) = enclosing_statements(node, ctx) else {
        return;
    };
    report_padding_before(node, ctx, name, statements);
}

pub fn report_missing_padding_after_jest_block<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
) {
    let Some(statements) = enclosing_statements(node, ctx) else {
        return;
    };
    report_padding_after(node, ctx, name, statements);
}

pub fn report_missing_padding_around_jest_block<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
) {
    let Some(statements) = enclosing_statements(node, ctx) else {
        return;
    };
    report_padding_before(node, ctx, name, statements);
    report_padding_after(node, ctx, name, statements);
}

fn report_padding_before<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
    statements: &[Statement<'a>],
) {
    let Some(prev_statement_span) = get_statement_span_before_node(node, statements) else {
        return;
    };
    report_padding_in_gap(
        ctx,
        name,
        PaddingSide::Before,
        prev_statement_span.end,
        node.span().start,
        node.span().start,
    );
}

fn report_padding_after<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
    statements: &[Statement<'a>],
) {
    let Some((current_statement_end, next_statement_start)) =
        get_statement_spans_around_node(node, statements)
    else {
        return;
    };
    report_padding_in_gap(
        ctx,
        name,
        PaddingSide::After,
        current_statement_end,
        next_statement_start,
        current_statement_end,
    );
}

fn enclosing_statements<'a, 'b>(
    node: &AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b [Statement<'a>]> {
    let scope_node = ctx.nodes().get_node(ctx.scoping().get_node_id(node.scope_id()));
    match scope_node.kind() {
        AstKind::Program(program) => Some(program.body.as_slice()),
        AstKind::ArrowFunctionExpression(arrow_func_expr) => {
            Some(arrow_func_expr.body.statements.as_slice())
        }
        AstKind::Function(function) => Some(function.body.as_ref()?.statements.as_slice()),
        _ => None,
    }
}

fn report_padding_in_gap(
    ctx: &LintContext,
    name: &str,
    side: PaddingSide,
    gap_start: u32,
    gap_end: u32,
    diagnostic_anchor: u32,
) {
    let span_between = shrink_gap_past_attached_comments(ctx, side, gap_start, gap_end);
    let content = ctx.source_range(span_between);
    if content.matches('\n').count() >= 2 {
        return;
    }

    ctx.diagnostic_with_fix(
        padding_diagnostic(side, Span::new(diagnostic_anchor, diagnostic_anchor), name),
        |fixer| {
            let preserved_whitespace =
                content.rfind('\n').map_or("", |index| content.split_at(index + 1).1);
            fixer.replace(span_between, format!("\n\n{preserved_whitespace}"))
        },
    );
}

fn shrink_gap_past_attached_comments(
    ctx: &LintContext,
    side: PaddingSide,
    gap_start: u32,
    gap_end: u32,
) -> Span {
    let mut span_between_start = gap_start;
    let mut span_between_end = gap_end;
    let comments_range = ctx.comments_range(gap_start..gap_end);

    match side {
        PaddingSide::Before => {
            let mut next_attached_start = gap_end;
            for comment in comments_range.rev() {
                let comment_span = comment.span;
                let space_after =
                    ctx.source_range(Span::new(comment_span.end, next_attached_start));
                if space_after.matches('\n').count() > 1 {
                    break;
                }
                let space_before = ctx.source_range(Span::new(gap_start, comment_span.start));
                if space_before.matches('\n').count() == 0 {
                    span_between_start = comment_span.end;
                    break;
                }
                span_between_end = comment_span.start;
                next_attached_start = comment_span.start;
            }
        }
        PaddingSide::After => {
            let mut prev_attached_end = gap_start;
            for comment in comments_range {
                let comment_span = comment.span;
                let space_before =
                    ctx.source_range(Span::new(prev_attached_end, comment_span.start));
                if space_before.matches('\n').count() > 1 {
                    break;
                }
                let space_after = ctx.source_range(Span::new(comment_span.end, gap_end));
                if space_after.matches('\n').count() == 0 {
                    span_between_end = comment_span.start;
                    break;
                }
                span_between_start = comment_span.end;
                prev_attached_end = comment_span.end;
            }
        }
    }

    Span::new(span_between_start, span_between_end)
}

fn get_statement_span_before_node(node: &AstNode, statements: &[Statement]) -> Option<Span> {
    statements
        .iter()
        .filter_map(|statement| {
            if statement.span().end <= node.span().start { Some(statement.span()) } else { None }
        })
        .next_back()
}

fn get_statement_spans_around_node(node: &AstNode, statements: &[Statement]) -> Option<(u32, u32)> {
    let node_span = node.span();
    let mut current_end = None;
    for statement in statements {
        let statement_span = statement.span();
        if statement_span.start <= node_span.start && statement_span.end >= node_span.end {
            current_end = Some(statement_span.end);
        } else if let Some(end) = current_end
            && statement_span.start >= end
        {
            return Some((end, statement_span.start));
        }
    }
    None
}
