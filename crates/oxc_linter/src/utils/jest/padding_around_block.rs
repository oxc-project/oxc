use oxc_ast::{
    AstKind,
    ast::{Expression, Statement, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::context::LintContext;

#[derive(Debug, Clone, Copy)]
pub enum PaddingDirection {
    Before,
    After,
}

fn missing_padding_diagnostic(
    span: Span,
    direction: PaddingDirection,
    name: &str,
) -> OxcDiagnostic {
    match direction {
        PaddingDirection::Before => {
            OxcDiagnostic::warn(format!("Missing padding before {name} block"))
                .with_help(format!("Make sure there is an empty new line before the {name} block"))
                .with_label(span)
        }
        PaddingDirection::After => {
            OxcDiagnostic::warn(format!("Missing padding after {name} block"))
                .with_help(format!("Make sure there is an empty new line after the {name} block"))
                .with_label(span)
        }
    }
}

pub fn enclosing_statement_list<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a [Statement<'a>]> {
    let scope_node = ctx.nodes().get_node(ctx.scoping().get_node_id(node.scope_id()));
    match scope_node.kind() {
        AstKind::Program(program) => Some(program.body.as_slice()),
        AstKind::ArrowFunctionExpression(arrow_func_expr) => {
            Some(arrow_func_expr.body.statements.as_slice())
        }
        AstKind::Function(function) => {
            function.body.as_ref().map(|body| body.statements.as_slice())
        }
        _ => None,
    }
}

pub fn enclosing_statement_index(node_span: Span, statements: &[Statement]) -> Option<usize> {
    statements.iter().position(|statement| statement.span().contains_inclusive(node_span))
}

pub struct LeadingStatementToken<'a> {
    pub name: &'a str,
    pub expr_start: u32,
}

/// Mirrors eslint-plugin-jest's token-based statement matching: labels and
/// `await` are unwrapped, then the leftmost identifier of the expression
/// chain is taken.
pub fn leading_token_of_statement<'a>(
    statement: &'a Statement<'a>,
) -> Option<LeadingStatementToken<'a>> {
    let mut statement = statement;
    while let Statement::LabeledStatement(labeled) = statement {
        statement = &labeled.body;
    }
    let Statement::ExpressionStatement(expr_stmt) = statement else {
        return None;
    };
    let mut expression = &expr_stmt.expression;
    while let Expression::AwaitExpression(await_expr) = expression {
        expression = &await_expr.argument;
    }
    let expr_start = expression.span().start;
    let mut current = expression;
    loop {
        match current {
            Expression::Identifier(ident) => {
                return Some(LeadingStatementToken { name: ident.name.as_str(), expr_start });
            }
            Expression::CallExpression(call_expr) => current = &call_expr.callee,
            Expression::TaggedTemplateExpression(tagged) => current = &tagged.tag,
            match_member_expression!(Expression) => {
                current = current.to_member_expression().object();
            }
            _ => return None,
        }
    }
}

/// Report and fix a missing blank line between two adjacent statements.
/// Comments attached to the next statement keep the padding before them.
pub fn check_padding_between(
    ctx: &LintContext<'_>,
    prev_end: u32,
    next_start: u32,
    direction: PaddingDirection,
    name: &str,
) {
    let comments_range = ctx.comments_range(prev_end..next_start);
    let mut span_between_start = prev_end;
    let mut span_between_end = next_start;
    let mut next_attached_start = next_start;
    for comment in comments_range.rev() {
        let comment_span = comment.span;
        let space_after = ctx.source_range(Span::new(comment_span.end, next_attached_start));
        if space_after.matches('\n').count() > 1 {
            break;
        }
        let space_before = ctx.source_range(Span::new(prev_end, comment_span.start));
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
            missing_padding_diagnostic(Span::new(next_start, next_start), direction, name),
            |fixer| {
                let whitespace_after_last_line =
                    content.rfind('\n').map_or("", |index| content.split_at(index + 1).1);
                fixer.replace(span_between, format!("\n\n{whitespace_after_last_line}"))
            },
        );
    }
}

pub fn report_missing_padding_before_jest_block<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
) {
    let Some(statements) = enclosing_statement_list(node, ctx) else {
        return;
    };
    let Some(index) = enclosing_statement_index(node.span(), statements) else {
        return;
    };
    let statement = &statements[index];
    // The call must lead its statement, e.g. `const x = it(...)` is not a block.
    let Some(token) = leading_token_of_statement(statement) else {
        return;
    };
    if token.expr_start != node.span().start {
        return;
    }
    let Some(prev) = index.checked_sub(1).map(|i| &statements[i]) else {
        return;
    };
    check_padding_between(
        ctx,
        prev.span().end,
        statement.span().start,
        PaddingDirection::Before,
        name,
    );
}
