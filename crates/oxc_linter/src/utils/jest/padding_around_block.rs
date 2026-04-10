use oxc_ast::{AstKind, ast::Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::context::LintContext;

fn padding_around_jest_block_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Missing padding before {name} block"))
        .with_help(format!("Make sure there is an empty new line before the {name} block"))
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
            get_statement_span_before_node(node, arrow_func_expr.body.statements.as_slice())
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

    let mut comments_range = ctx.comments_range(prev_statement_span.end..node.span().start);
    let mut span_between_start = prev_statement_span.end;
    let mut span_between_end = node.span().start;
    if let Some(last_comment_span) = comments_range.next_back().map(|comment| comment.span) {
        let space_after_last_comment =
            ctx.source_range(Span::new(last_comment_span.end, node.span().start));
        let space_before_last_comment =
            ctx.source_range(Span::new(prev_statement_span.end, last_comment_span.start));
        if space_after_last_comment.matches('\n').count() > 1
            || space_before_last_comment.matches('\n').count() == 0
        {
            span_between_start = last_comment_span.end;
        } else {
            span_between_end = last_comment_span.start;
        }
    }

    let span_between = Span::new(span_between_start, span_between_end);
    let content = ctx.source_range(span_between);
    if content.matches('\n').count() < 2 {
        ctx.diagnostic_with_fix(
            padding_around_jest_block_diagnostic(
                Span::new(span_between_end, span_between_end),
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
