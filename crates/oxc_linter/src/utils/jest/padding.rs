use oxc_ast::{
    AstKind,
    ast::{Directive, Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    identifier::{is_identifier_part, is_identifier_start, is_white_space},
    line_terminator::is_line_terminator,
};

use crate::context::LintContext;

fn missing_padding_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected blank line before this statement")
        .with_help("Insert a blank line before this statement")
        .with_label(span)
}

#[derive(Clone, Copy)]
pub enum PaddingStatement {
    Any,
    Named(&'static [&'static str]),
}

impl PaddingStatement {
    fn matches(self, node: PaddingNode<'_>) -> bool {
        match self {
            Self::Any => true,
            Self::Named(names) => node.name.is_some_and(|name| names.contains(&name)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PaddingType {
    Any,
    Always,
}

#[derive(Clone, Copy)]
pub struct PaddingConfig {
    padding: PaddingType,
    prev: PaddingStatement,
    next: PaddingStatement,
}

impl PaddingConfig {
    pub const fn any(prev: PaddingStatement, next: PaddingStatement) -> Self {
        Self { padding: PaddingType::Any, prev, next }
    }

    pub const fn always(prev: PaddingStatement, next: PaddingStatement) -> Self {
        Self { padding: PaddingType::Always, prev, next }
    }
}

#[derive(Clone, Copy)]
struct PaddingNode<'a> {
    span: Span,
    name: Option<&'a str>,
}

impl<'a> PaddingNode<'a> {
    fn from_directive(directive: &Directive<'a>) -> Self {
        Self { span: directive.span, name: None }
    }

    fn from_statement(statement: &Statement<'a>, source_text: &'a str) -> Self {
        let mut inner = statement;
        while let Statement::LabeledStatement(labeled) = inner {
            inner = &labeled.body;
        }

        let name = match inner {
            Statement::ExpressionStatement(statement) => {
                let start = match &statement.expression {
                    Expression::AwaitExpression(await_expr) => {
                        await_expr.argument.without_parentheses().span().start
                    }
                    _ => statement.span.start,
                };
                leading_identifier(source_text, start)
            }
            _ => None,
        };

        Self { span: statement.span(), name }
    }
}

pub fn check_padding<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>, configs: &[&[PaddingConfig]]) {
    let source_text = ctx.source_text();
    match node.kind() {
        AstKind::Program(program) => check_nodes(
            program.directives.iter().map(PaddingNode::from_directive).chain(
                program
                    .body
                    .iter()
                    .map(|statement| PaddingNode::from_statement(statement, source_text)),
            ),
            ctx,
            configs,
        ),
        AstKind::FunctionBody(body) => check_nodes(
            body.directives.iter().map(PaddingNode::from_directive).chain(
                body.statements
                    .iter()
                    .map(|statement| PaddingNode::from_statement(statement, source_text)),
            ),
            ctx,
            configs,
        ),
        AstKind::BlockStatement(block) => check_nodes(
            block.body.iter().map(|statement| PaddingNode::from_statement(statement, source_text)),
            ctx,
            configs,
        ),
        AstKind::SwitchCase(case) => check_nodes(
            case.consequent
                .iter()
                .map(|statement| PaddingNode::from_statement(statement, source_text)),
            ctx,
            configs,
        ),
        _ => {}
    }
}

fn check_nodes<'a>(
    mut nodes: impl Iterator<Item = PaddingNode<'a>>,
    ctx: &LintContext<'a>,
    configs: &[&[PaddingConfig]],
) {
    let Some(mut prev) = nodes.next() else {
        return;
    };

    for next in nodes {
        if requires_padding(prev, next, configs) {
            report_missing_padding(prev.span, next.span, ctx);
        }
        prev = next;
    }
}

fn requires_padding(
    prev: PaddingNode<'_>,
    next: PaddingNode<'_>,
    configs: &[&[PaddingConfig]],
) -> bool {
    configs
        .iter()
        .rev()
        .flat_map(|configs| configs.iter().rev())
        .find(|config| config.prev.matches(prev) && config.next.matches(next))
        .is_some_and(|config| config.padding == PaddingType::Always)
}

fn report_missing_padding(prev: Span, next: Span, ctx: &LintContext<'_>) {
    let prev_end = actual_last_token_end(prev, ctx);
    if has_padding_line(prev_end, next.start, ctx) {
        return;
    }

    ctx.diagnostic_with_fix(missing_padding_diagnostic(Span::empty(next.start)), |fixer| {
        let (insert_at, next_start) = fix_insert_position(prev_end, next.start, ctx);
        if has_line_break(insert_at, next_start, ctx) {
            fixer.insert_text_after_range(Span::empty(insert_at), "\n")
        } else {
            fixer.replace(Span::new(insert_at, next_start), "\n\n")
        }
    });
}

fn leading_identifier(source_text: &str, start: u32) -> Option<&str> {
    let text = &source_text[start as usize..];
    let mut chars = text.char_indices();
    let (_, first) = chars.next()?;
    if !is_identifier_start(first) {
        return None;
    }
    let end = chars.find(|(_, c)| !is_identifier_part(*c)).map_or(text.len(), |(index, _)| index);
    Some(&text[..end])
}

fn actual_last_token_end(span: Span, ctx: &LintContext<'_>) -> u32 {
    if span.is_empty() || !ctx.source_range(span).ends_with(';') {
        return span.end;
    }

    let semicolon_start = span.end - 1;
    let Some(prev_token_end) = previous_token_end(semicolon_start, span.start, ctx) else {
        return span.end;
    };
    let Some(next_token_start) = next_token_start(span.end, ctx) else {
        return span.end;
    };

    if has_line_break(prev_token_end, semicolon_start, ctx)
        && !has_line_break(span.end, next_token_start, ctx)
    {
        prev_token_end
    } else {
        span.end
    }
}

fn has_padding_line(prev_end: u32, next_start: u32, ctx: &LintContext<'_>) -> bool {
    let mut gap_start = prev_end;
    loop {
        let item = next_item(gap_start, next_start, ctx);
        let gap_end = item.map_or(next_start, |item| item.start);
        if line_break_count(gap_start, gap_end, ctx) >= 2 {
            return true;
        }
        let Some(item) = item else {
            return false;
        };
        gap_start = item.end;
    }
}

fn fix_insert_position(prev_end: u32, next_start: u32, ctx: &LintContext<'_>) -> (u32, u32) {
    let mut insert_at = prev_end;
    while let Some(item) = next_item(insert_at, next_start, ctx) {
        if has_line_break(insert_at, item.start, ctx) {
            return (insert_at, item.start);
        }
        insert_at = item.end;
    }
    (insert_at, next_start)
}

fn next_item(start: u32, limit: u32, ctx: &LintContext<'_>) -> Option<Span> {
    let start = skip_whitespace_forward(start, limit, ctx);
    if start >= limit {
        return None;
    }
    if let Some(comment) = comment_starting_at(start, ctx) {
        return Some(comment);
    }
    let len = ctx.source_text()[start as usize..].chars().next().map_or(1, char::len_utf8);
    Some(Span::sized(start, u32::try_from(len).unwrap()))
}

fn next_token_start(start: u32, ctx: &LintContext<'_>) -> Option<u32> {
    let limit = u32::try_from(ctx.source_text().len()).unwrap();
    let mut pos = start;
    loop {
        pos = skip_whitespace_forward(pos, limit, ctx);
        if pos >= limit {
            return None;
        }
        let Some(comment) = comment_starting_at(pos, ctx) else {
            return Some(pos);
        };
        pos = comment.end;
    }
}

fn previous_token_end(end: u32, min: u32, ctx: &LintContext<'_>) -> Option<u32> {
    let mut pos = end;
    loop {
        pos = skip_whitespace_backward(pos, min, ctx);
        if pos <= min {
            return None;
        }
        let Some(comment) = comment_ending_at(pos, ctx) else {
            return Some(pos);
        };
        pos = comment.start;
    }
}

fn skip_whitespace_forward(start: u32, limit: u32, ctx: &LintContext<'_>) -> u32 {
    let text = &ctx.source_text()[start as usize..limit as usize];
    let offset = text
        .char_indices()
        .find(|(_, c)| !is_white_space(*c) && !is_line_terminator(*c))
        .map_or(text.len(), |(index, _)| index);
    start + u32::try_from(offset).unwrap()
}

fn skip_whitespace_backward(end: u32, min: u32, ctx: &LintContext<'_>) -> u32 {
    let text = &ctx.source_text()[min as usize..end as usize];
    let offset = text
        .char_indices()
        .rev()
        .find(|(_, c)| !is_white_space(*c) && !is_line_terminator(*c))
        .map_or(0, |(index, c)| index + c.len_utf8());
    min + u32::try_from(offset).unwrap()
}

fn comment_starting_at(start: u32, ctx: &LintContext<'_>) -> Option<Span> {
    let comments = ctx.comments();
    comments
        .binary_search_by_key(&start, |comment| comment.span.start)
        .ok()
        .map(|index| comments[index].span)
}

fn comment_ending_at(end: u32, ctx: &LintContext<'_>) -> Option<Span> {
    let comments = ctx.comments();
    comments
        .binary_search_by_key(&end, |comment| comment.span.end)
        .ok()
        .map(|index| comments[index].span)
}

fn has_line_break(start: u32, end: u32, ctx: &LintContext<'_>) -> bool {
    ctx.source_range(Span::new(start, end)).chars().any(is_line_terminator)
}

fn line_break_count(start: u32, end: u32, ctx: &LintContext<'_>) -> usize {
    let mut count = 0;
    let mut chars = ctx.source_range(Span::new(start, end)).chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' && chars.peek() == Some(&'\n') {
            chars.next();
        }
        if is_line_terminator(c) {
            count += 1;
        }
    }
    count
}
