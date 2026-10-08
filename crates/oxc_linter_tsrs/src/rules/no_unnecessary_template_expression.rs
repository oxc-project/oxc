// Port of internal/rules/no_unnecessary_template_expression/no_unnecessary_template_expression.go.

use tsrs_ast::{self as ast, Kind, Node, NodeList};
use tsrs_checker::{Type, TypeFlags};
use tsrs_core::{P, TextRange, jsnum};

use crate::rule::{Ctx, Listener, Rule, RuleDiagnostic, RuleFix, RuleMessage, RuleVisitor};
use crate::utils;

fn build_no_unnecessary_template_expression_message() -> RuleMessage {
    RuleMessage::new(
        "noUnnecessaryTemplateExpression",
        "Template literal expression is unnecessary and can be simplified.",
    )
}

fn is_underlying_type_string(t: P<Type>) -> bool {
    utils::union_type_parts(t).into_iter().all(|t| {
        utils::intersection_type_parts(t)
            .into_iter()
            .any(|t| utils::is_type_flag_set(t, TypeFlags::StringLike))
    })
}

fn is_any_literal(node: P<Node>) -> bool {
    ast::is_literal_expression(node)
        || ast::is_boolean_literal(node)
        || node.kind() == Kind::NullKeyword
}

fn is_fixable_identifier(node: P<Node>) -> bool {
    if ast::is_identifier(node) {
        let name = node.as_identifier().text();
        return name == "undefined" || name == "Infinity" || name == "NaN";
    }
    node.kind() == Kind::UndefinedKeyword
}

fn starts_with_newline(s: &str) -> bool {
    s.starts_with('\n') || s.starts_with("\r\n")
}

fn is_whitespace(s: &str) -> bool {
    // allow empty string too
    s.chars().all(utils::is_str_white_space)
}

fn ends_with_unescaped_dollar_sign(s: &str) -> bool {
    if !s.ends_with('$') {
        return false;
    }
    let b = s.as_bytes();
    let mut backslashes = 0;
    let mut i = b.len() as isize - 2;
    while i >= 0 && b[i as usize] == b'\\' {
        backslashes += 1;
        i -= 1;
    }
    backslashes % 2 == 0
}

fn escape_template_raw_text(text: &str) -> String {
    let b = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    for i in 0..b.len() {
        let needs_escape = b[i] == b'\x60' || (b[i] == b'$' && i + 1 < b.len() && b[i + 1] == b'{');
        if needs_escape {
            let mut backslashes = 0;
            let mut j = i as isize - 1;
            while j >= 0 && b[j as usize] == b'\\' {
                backslashes += 1;
                j -= 1;
            }
            if backslashes % 2 == 0 {
                out.push(b'\\');
            }
        }
        out.push(b[i]);
    }
    String::from_utf8(out).unwrap()
}

fn canonical_numeric_literal_text(text: &str) -> String {
    let text = text.replace('_', "");
    jsnum::from_string(&text).to_string()
}

fn canonical_big_int_literal_text(text: &str) -> String {
    jsnum::parse_pseudo_big_int(&text.replace('_', ""))
}

pub struct NoUnnecessaryTemplateExpression;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnnecessaryTemplateExpression))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::TemplateExpression), Listener::Enter(Kind::TemplateLiteralType)];

impl Rule for NoUnnecessaryTemplateExpression {
    fn name(&self) -> &'static str {
        "no-unnecessary-template-expression"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn node_text(ctx: &Ctx, node: P<Node>) -> &'static str {
    let (pos, end) = ctx.trim(node);
    &ctx.text()[pos as usize..end as usize]
}

fn report_single_interpolation(
    ctx: &mut Ctx,
    template: P<Node>,
    interpolation: P<Node>,
    span_literal: P<Node>,
) {
    ctx.report_diagnostic_with_fixes(
        RuleDiagnostic {
            pos: interpolation.pos() - 2,
            end: span_literal.pos() + 1,
            message: build_no_unnecessary_template_expression_message(),
            labeled_ranges: Vec::new(),
        },
        |ctx| {
            let mut text = node_text(ctx, interpolation).to_string();
            let parent = template.parent().unwrap();
            let parent_is_nullish_coalescing = ast::is_binary_expression(parent)
                && parent.as_binary_expression().operator_token.kind()
                    == Kind::QuestionQuestionToken;
            let interpolation_is_logical = ast::is_binary_expression(interpolation)
                && matches!(
                    interpolation.as_binary_expression().operator_token.kind(),
                    Kind::BarBarToken | Kind::AmpersandAmpersandToken
                );
            let needs_parentheses = parent_is_nullish_coalescing && interpolation_is_logical
                || ast::is_expression(parent)
                    && (ast::get_expression_precedence(interpolation) as i32)
                        <= (ast::get_expression_precedence(parent) as i32);
            if needs_parentheses {
                text = format!("({text})");
            }
            vec![ctx.fix_replace(template, text)]
        },
    );
}

fn is_unnecessary_value_interpolation(
    ctx: &Ctx,
    mut expression: P<Node>,
    prev_quasi_end: i32,
    next_quasi_literal: P<Node>,
) -> bool {
    if utils::has_comments_in_range(
        ctx.file,
        TextRange::new(prev_quasi_end, next_quasi_literal.pos()),
    ) || utils::has_comments_in_range(
        ctx.file,
        TextRange::new(next_quasi_literal.pos(), ctx.trim(next_quasi_literal).0),
    ) {
        return false;
    }
    if ast::is_literal_type_node(expression) {
        expression = expression.as_literal_type_node().literal;
    }
    if is_fixable_identifier(expression) {
        return true;
    }
    if ast::is_string_literal_like(expression) {
        let raw = next_quasi_literal.raw_text();
        // allow trailing whitespace literal
        return !starts_with_newline(raw) || !is_whitespace(expression.text());
    }
    is_any_literal(expression) || ast::is_template_expression(expression)
}

fn get_literal(mut node: P<Node>) -> Option<P<Node>> {
    if ast::is_literal_type_node(node) {
        node = node.as_literal_type_node().literal;
    }
    if ast::is_literal_expression(node) {
        return Some(node);
    }
    None
}

fn get_template_literal(mut node: P<Node>) -> Option<P<Node>> {
    if ast::is_literal_type_node(node) {
        node = node.as_literal_type_node().literal;
    }
    if ast::is_template_expression(node) {
        return Some(node);
    }
    None
}

fn build_literal_replacement_text(
    ctx: &Ctx,
    literal: P<Node>,
    next_character_is_opening_curly_brace: bool,
) -> (String, bool, bool) {
    let (pos, end) = ctx.trim(literal);
    let source_text = ctx.text();
    let mut text = match literal.kind() {
        Kind::StringLiteral | Kind::NoSubstitutionTemplateLiteral => {
            source_text[(pos + 1) as usize..(end - 1) as usize].to_string()
        }
        Kind::NumericLiteral => canonical_numeric_literal_text(literal.text()),
        Kind::BigIntLiteral => canonical_big_int_literal_text(literal.text()),
        Kind::RegularExpressionLiteral => literal.text().replace('\\', "\\\\"),
        _ => node_text(ctx, literal).to_string(),
    };
    text = escape_template_raw_text(&text);
    if next_character_is_opening_curly_brace && ends_with_unescaped_dollar_sign(&text) {
        text = format!("{}\\$", &text[..text.len() - 1]);
    }
    let starts = text.starts_with('{');
    let non_empty = !text.is_empty();
    (text, starts, non_empty)
}

fn is_trivial_interpolation(
    ctx: &Ctx,
    template_spans: P<NodeList>,
    head: P<Node>,
    first_span_literal: P<Node>,
) -> bool {
    template_spans.nodes().len() == 1
        && head.text().is_empty()
        && first_span_literal.text().is_empty()
        && !utils::has_comments_in_range(
            ctx.file,
            TextRange::new(head.end(), first_span_literal.pos()),
        )
        && !utils::has_comments_in_range(
            ctx.file,
            TextRange::new(first_span_literal.pos(), ctx.trim(first_span_literal).0),
        )
}

fn is_enum_member_type(t: P<Type>) -> bool {
    utils::type_recurser(t, &mut |t| {
        t.symbol().and_then(|s| s.value_declaration()).is_some_and(ast::is_enum_member)
    })
}

fn span_parts(span: P<Node>) -> (P<Node>, P<Node>) {
    if span.kind() == Kind::TemplateSpan {
        let s = span.as_template_span();
        (s.expression, s.literal)
    } else {
        let s = span.as_template_literal_type_span();
        (s.type_, s.literal)
    }
}

fn check_template_spans(ctx: &mut Ctx, template_spans: P<NodeList>, head: P<Node>) {
    let mut next_character_is_opening_curly_brace = false;
    let spans = template_spans.nodes();
    for i in (0..spans.len()).rev() {
        let span = spans[i];
        let prev_quasi_end = if i == 0 { head.end() } else { spans[i - 1].end() };
        let (expr, literal) = span_parts(span);
        if !is_unnecessary_value_interpolation(ctx, expr, prev_quasi_end, literal) {
            continue;
        }
        let raw = literal.raw_text();
        if !raw.is_empty() {
            next_character_is_opening_curly_brace = raw.starts_with('{');
        }
        let expr_range = ctx.trim(expr);
        let literal_trim_pos = ctx.trim(literal).0;
        let mut fixes: Vec<RuleFix> = vec![
            ctx.fix_remove_range(prev_quasi_end - 2, expr_range.0),
            ctx.fix_remove_range(expr_range.1, literal_trim_pos + 1),
        ];
        if let Some(lit) = get_literal(expr) {
            let (replacement, starts_with_opening_curly_brace, should_update_next_character) =
                build_literal_replacement_text(ctx, lit, next_character_is_opening_curly_brace);
            if should_update_next_character {
                next_character_is_opening_curly_brace = starts_with_opening_curly_brace;
            }
            fixes.push(ctx.fix_replace(lit, replacement));
        } else if let Some(template_literal) = get_template_literal(expr) {
            let (tl_pos, tl_end) = ctx.trim(template_literal);
            let template_expr = template_literal.as_template_expression();
            let last_span = *template_expr.template_spans.nodes().last().unwrap();
            if next_character_is_opening_curly_brace
                && ends_with_unescaped_dollar_sign(last_span.as_template_span().literal.raw_text())
            {
                fixes.push(ctx.fix_replace_range(tl_end - 2, tl_end - 2, "\\"));
            }
            let head_raw = template_expr.head.raw_text();
            if !head_raw.is_empty() {
                next_character_is_opening_curly_brace = head_raw.starts_with('{');
            }
            fixes.push(ctx.fix_remove_range(tl_pos, tl_pos + 1));
            fixes.push(ctx.fix_remove_range(tl_end - 1, tl_end));
        } else {
            next_character_is_opening_curly_brace = false;
        }
        let prev_raw = if i == 0 { head.raw_text() } else { span_parts(spans[i - 1]).1.raw_text() };
        if next_character_is_opening_curly_brace && ends_with_unescaped_dollar_sign(prev_raw) {
            fixes.push(ctx.fix_replace_range(prev_quasi_end - 3, prev_quasi_end - 2, "\\$"));
        }
        ctx.report_diagnostic_with_fixes(
            RuleDiagnostic {
                pos: prev_quasi_end - 2,
                end: literal_trim_pos + 1,
                message: build_no_unnecessary_template_expression_message(),
                labeled_ranges: Vec::new(),
            },
            |_| fixes,
        );
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::TemplateExpression => {
                if ast::is_tagged_template_expression(node.parent().unwrap()) {
                    return;
                }
                let expr = node.as_template_expression();
                let first_span = expr.template_spans.nodes()[0].as_template_span();
                if is_trivial_interpolation(ctx, expr.template_spans, expr.head, first_span.literal)
                {
                    let t = ctx.checker.get_type_at_location(first_span.expression);
                    let (constraint_type, _) = utils::get_constraint_info(ctx.checker, t);
                    if constraint_type.is_some_and(is_underlying_type_string) {
                        report_single_interpolation(
                            ctx,
                            node,
                            first_span.expression,
                            first_span.literal,
                        );
                        return;
                    }
                }
                check_template_spans(ctx, expr.template_spans, expr.head);
            }
            Kind::TemplateLiteralType => {
                let expr = node.as_template_literal_type_node();
                let first_span = expr.template_spans.nodes()[0].as_template_literal_type_span();
                if is_trivial_interpolation(ctx, expr.template_spans, expr.head, first_span.literal)
                {
                    let t = ctx.checker.get_type_at_location(first_span.type_);
                    let (constraint_type, is_type_parameter) =
                        utils::get_constraint_info(ctx.checker, t);
                    if let Some(constraint_type) = constraint_type {
                        if !is_type_parameter
                            && is_underlying_type_string(constraint_type)
                            && !is_enum_member_type(constraint_type)
                        {
                            report_single_interpolation(
                                ctx,
                                node,
                                first_span.type_,
                                first_span.literal,
                            );
                            return;
                        }
                    }
                }
                check_template_spans(ctx, expr.template_spans, expr.head);
            }
            _ => {}
        }
    }
}
