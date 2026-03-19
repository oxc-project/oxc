use oxc_allocator::Allocator;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, ExpressionKind},
};
use oxc_regular_expression::{ConstructorParser, Options, ast::Pattern};
use oxc_semantic::IsGlobalReference;
use oxc_span::{
    Span,
    ident::{GLOBAL_THIS, REG_EXP},
};

use crate::{AstNode, context::LintContext};

pub fn run_on_regex_node<'a, 'b, M>(node: &'a AstNode<'b>, ctx: &'a LintContext<'b>, cb: M)
where
    M: FnOnce(&Pattern<'_>, Span),
{
    match node.kind() {
        AstKind::RegExpLiteral(reg) => {
            if let Some(pat) = &reg.regex.pattern.pattern {
                cb(pat, reg.span);
            }
        }
        AstKind::NewExpression(expr) if is_regexp_callee(&expr.callee, ctx) => {
            run_on_arguments(expr.arguments.first(), expr.arguments.get(1), ctx, cb);
        }

        // RegExp()
        AstKind::CallExpression(expr) if is_regexp_callee(&expr.callee, ctx) => {
            run_on_arguments(expr.arguments.first(), expr.arguments.get(1), ctx, cb);
        }
        _ => {}
    }
}

fn run_on_arguments<M>(arg1: Option<&Argument>, arg2: Option<&Argument>, ctx: &LintContext, cb: M)
where
    M: FnOnce(&Pattern<'_>, Span),
{
    let arg1 =
        arg1.and_then(Argument::as_expression).as_ref().map(Expression::get_inner_expression);
    let arg2 =
        arg2.and_then(Argument::as_expression).as_ref().map(Expression::get_inner_expression);
    // note: improvements required for strings used via identifier references
    // Missing or non-string arguments will be runtime errors, but are not covered by this rule.
    match (arg1.map(|e| e.kind()), arg2.map(|e| e.kind())) {
        (
            Some(ExpressionKind::StringLiteral(pattern)),
            Some(ExpressionKind::StringLiteral(flags)),
        ) => {
            let allocator = Allocator::default();
            if let Some(pat) = parse_regex(&allocator, pattern.span, Some(flags.span), ctx) {
                cb(&pat, pattern.span);
            }
        }
        (
            Some(ExpressionKind::StringLiteral(pattern)),
            Some(ExpressionKind::TemplateLiteral(flags)),
        ) => {
            if !flags.is_no_substitution_template() {
                return;
            }
            let allocator = Allocator::default();
            if let Some(pat) = parse_regex(&allocator, pattern.span, Some(flags.span), ctx) {
                cb(&pat, pattern.span);
            }
        }
        (Some(ExpressionKind::StringLiteral(pattern)), _) => {
            let allocator = Allocator::default();
            if let Some(pat) = parse_regex(&allocator, pattern.span, None, ctx) {
                cb(&pat, pattern.span);
            }
        }
        (
            Some(ExpressionKind::TemplateLiteral(pattern)),
            Some(ExpressionKind::TemplateLiteral(flags)),
        ) => {
            if !pattern.is_no_substitution_template() || !flags.is_no_substitution_template() {
                return;
            }
            let allocator = Allocator::default();
            if let Some(pat) = parse_regex(&allocator, pattern.span, Some(flags.span), ctx) {
                cb(&pat, pattern.span);
            }
        }
        (
            Some(ExpressionKind::TemplateLiteral(pattern)),
            Some(ExpressionKind::StringLiteral(flags)),
        ) => {
            if !pattern.is_no_substitution_template() {
                return;
            }
            let allocator = Allocator::default();
            if let Some(pat) = parse_regex(&allocator, pattern.span, Some(flags.span), ctx) {
                cb(&pat, pattern.span);
            }
        }
        (Some(ExpressionKind::TemplateLiteral(pattern)), _) => {
            if !pattern.is_no_substitution_template() {
                return;
            }
            let allocator = Allocator::default();
            if let Some(pat) = parse_regex(&allocator, pattern.span, None, ctx) {
                cb(&pat, pattern.span);
            }
        }
        _ => {}
    }
}

// Accepts both RegExp and globalThis.RegExp
fn is_regexp_callee<'a>(callee: &'a Expression<'a>, ctx: &'a LintContext<'_>) -> bool {
    if callee.is_global_reference_name(REG_EXP, ctx.semantic().scoping()) {
        return true;
    }
    // Check for globalThis.RegExp (StaticMemberExpression)
    if let Some(member) = callee.as_static_member_expression()
        && let Some(obj) = member.object.as_identifier()
        && obj.is_global_reference_name(GLOBAL_THIS, ctx.semantic().scoping())
        && member.property.name == "RegExp"
    {
        return true;
    }
    false
}

fn parse_regex<'a>(
    allocator: &'a Allocator,
    pattern_span: Span,
    flags_span: Option<Span>,
    ctx: &'a LintContext<'_>,
) -> Option<Pattern<'a>> {
    let flags_text = flags_span.map(|span| span.source_text(ctx.source_text()));
    let parser = ConstructorParser::new(
        allocator,
        pattern_span.source_text(ctx.source_text()),
        flags_text,
        Options {
            pattern_span_offset: pattern_span.start,
            flags_span_offset: flags_span.map_or(0, |span| span.start),
        },
    );
    let Ok(pattern) = parser.parse() else { return None };
    Some(pattern)
}
