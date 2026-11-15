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

pub fn report_missing_padding_before_jest_block<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
) {
    let mut statement_node = ctx.nodes().parent_node(node.id());
    let statement = loop {
        match statement_node.kind() {
            AstKind::ExpressionStatement(statement) => break statement,
            AstKind::AwaitExpression(_) | AstKind::ParenthesizedExpression(_) => {}
            _ if statement_node.span().start == node.span().start => {}
            _ => return,
        }
        statement_node = ctx.nodes().parent_node(statement_node.id());
    };
    let leading_expression = match &statement.expression {
        Expression::AwaitExpression(await_expr) => await_expr.argument.without_parentheses(),
        expression => expression,
    };
    if leading_expression.span().start != node.span().start {
        return;
    }

    let scope_node = ctx.nodes().get_node(ctx.scoping().get_node_id(node.scope_id()));
    let prev_statement_span = match scope_node.kind() {
        AstKind::Program(program) => {
            get_statement_span_before_node(statement_node, program.body.as_slice())
        }
        AstKind::ArrowFunctionExpression(arrow_func_expr) => {
            let Some(body) = arrow_func_expr.get_function_body() else { return };
            get_statement_span_before_node(statement_node, body.statements.as_slice())
        }
        AstKind::Function(function) => {
            let Some(body) = &function.body else {
                return;
            };
            get_statement_span_before_node(statement_node, body.statements.as_slice())
        }
        _ => None,
    };
    let Some(prev_statement_span) = prev_statement_span else {
        return;
    };

    let statement_start = statement_node.span().start;
    let comments_range = ctx.comments_range(prev_statement_span.end..statement_start);
    let mut span_between_start = prev_statement_span.end;
    let mut span_between_end = statement_start;
    let mut next_attached_start = statement_start;
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

fn get_statement_span_before_node(node: &AstNode, statements: &[Statement]) -> Option<Span> {
    statements.windows(2).find_map(|pair| (pair[1].span() == node.span()).then(|| pair[0].span()))
}
