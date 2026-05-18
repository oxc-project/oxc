use oxc_allocator::Allocator;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_regular_expression::{ConstructorParser, Options, ast::Pattern};
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;
use oxc_str::static_ident;

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
    let arg1 = arg1.and_then(Argument::as_expression).map(Expression::get_inner_expression);
    let arg2 = arg2.and_then(Argument::as_expression).map(Expression::get_inner_expression);
    // note: improvements required for strings used via identifier references
    // Missing or non-string arguments will be runtime errors, but are not covered by this rule.
    match (arg1, arg2) {
        (Some(Expression::StringLiteral(pattern)), Some(Expression::StringLiteral(flags))) => {
            let allocator = Allocator::default();
            if let Some(pat) = parse_regex(&allocator, pattern.span, Some(flags.span), ctx) {
                cb(&pat, pattern.span);
            }
        }
        (Some(Expression::StringLiteral(pattern)), Some(Expression::TemplateLiteral(flags))) => {
            if !flags.is_no_substitution_template() {
                return;
            }
            let allocator = Allocator::default();
            if let Some(pat) = parse_regex(&allocator, pattern.span, Some(flags.span), ctx) {
                cb(&pat, pattern.span);
            }
        }
        (Some(Expression::StringLiteral(pattern)), _) => {
            let allocator = Allocator::default();
            if let Some(pat) = parse_regex(&allocator, pattern.span, None, ctx) {
                cb(&pat, pattern.span);
            }
        }
        (Some(Expression::TemplateLiteral(pattern)), Some(Expression::TemplateLiteral(flags))) => {
            if !pattern.is_no_substitution_template() || !flags.is_no_substitution_template() {
                return;
            }
            let allocator = Allocator::default();
            if let Some(pat) = parse_regex(&allocator, pattern.span, Some(flags.span), ctx) {
                cb(&pat, pattern.span);
            }
        }
        (Some(Expression::TemplateLiteral(pattern)), Some(Expression::StringLiteral(flags))) => {
            if !pattern.is_no_substitution_template() {
                return;
            }
            let allocator = Allocator::default();
            if let Some(pat) = parse_regex(&allocator, pattern.span, Some(flags.span), ctx) {
                cb(&pat, pattern.span);
            }
        }
        (Some(Expression::TemplateLiteral(pattern)), _) => {
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

// Accepts global RegExp constructors like `RegExp`, `globalThis.RegExp`, `window.RegExp`, `window["RegExp"]("a")`,
// and `global.RegExp`.
pub fn is_regexp_callee<'a>(callee: &'a Expression<'a>, ctx: &'a LintContext<'_>) -> bool {
    if callee.is_global_reference_name(static_ident!("RegExp"), ctx.semantic().scoping()) {
        return true;
    }
    if let Some(member) = callee.get_member_expr()
        && let Expression::Identifier(obj) = &member.object().get_inner_expression()
        && member.static_property_name() == Some("RegExp")
        && (obj.is_global_reference_name(static_ident!("globalThis"), ctx.semantic().scoping())
            || obj.is_global_reference_name(static_ident!("window"), ctx.semantic().scoping())
            || obj.is_global_reference_name(static_ident!("global"), ctx.semantic().scoping()))
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

#[cfg(test)]
mod test {
    use std::{rc::Rc, sync::Arc};

    use oxc_allocator::Allocator;
    use oxc_ast::AstKind;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    use super::is_regexp_callee;
    use crate::{
        ModuleRecord,
        context::{ContextHost, ContextSubHost, ContextSubHostOptions},
        options::LintOptions,
    };

    #[test]
    fn test_is_regexp_callee() {
        let pass = [
            r#"RegExp("a")"#,
            r#"new RegExp("a")"#,
            r#"globalThis.RegExp("a")"#,
            r#"new globalThis.RegExp("a")"#,
            r#"window.RegExp("a")"#,
            r#"window["RegExp"]("a")"#,
            r#"new global["RegExp"]("a")"#,
        ];

        for source in pass {
            assert_eq!(is_first_call_or_new_regexp(source), Some(true), "{source}");
        }

        let fail = [
            r#"Regexp("a")"#,
            r#"let RegExp; RegExp("a")"#,
            r#"const window = {}; window.RegExp("a")"#,
            r#"const globalThis = {}; globalThis.RegExp("a")"#,
            r#"globalThis[RegExp]("a")"#,
            r#"class C { #RegExp; foo() { globalThis.#RegExp("a"); } }"#,
        ];

        for source in fail {
            assert_eq!(is_first_call_or_new_regexp(source), Some(false), "{source}");
        }
    }

    fn is_first_call_or_new_regexp(source: &str) -> Option<bool> {
        let allocator = Allocator::default();
        let parser_ret = Parser::new(&allocator, source, SourceType::default()).parse();
        assert!(parser_ret.errors.is_empty(), "Parse error in: {source}");

        let program = allocator.alloc(parser_ret.program);
        let semantic = SemanticBuilder::new().with_cfg(true).build(program).semantic;
        let ctx = Rc::new(ContextHost::new(
            "test.js",
            vec![ContextSubHost::new(
                semantic,
                Arc::new(ModuleRecord::default()),
                0,
                ContextSubHostOptions::default(),
            )],
            LintOptions::default(),
            Arc::default(),
        ))
        .spawn_for_test();

        ctx.nodes().iter().find_map(|node| match node.kind() {
            AstKind::CallExpression(call_expr) => Some(is_regexp_callee(&call_expr.callee, &ctx)),
            AstKind::NewExpression(new_expr) => Some(is_regexp_callee(&new_expr.callee, &ctx)),
            _ => None,
        })
    }
}
