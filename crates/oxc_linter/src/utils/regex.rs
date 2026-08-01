use oxc_allocator::Allocator;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, RegExpFlags},
};
use oxc_regular_expression::{ConstructorParser, LiteralParser, Options, ast::Pattern};
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
            // We need to make sure that this regex literal is not part of a `new RegExp()` or `RegExp()` call, as those are handled separately.
            // the flags of a regex literal will be overridden by the flags argument of the `new RegExp()` or `RegExp()` call.
            for ancestor_kind in ctx.nodes().ancestor_kinds(node.id()) {
                match ancestor_kind {
                    AstKind::ParenthesizedExpression(_)
                    | AstKind::TSAsExpression(_)
                    | AstKind::TSSatisfiesExpression(_)
                    | AstKind::TSInstantiationExpression(_)
                    | AstKind::TSNonNullExpression(_)
                    | AstKind::TSTypeAssertion(_) => { /* continue */ }
                    AstKind::NewExpression(expr) if is_regexp_callee(&expr.callee, ctx) => return,
                    AstKind::CallExpression(expr) if is_regexp_callee(&expr.callee, ctx) => return,
                    _ => break,
                }
            }

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

pub enum RegexFlagsParseResult {
    // The flags argument is either missing, or successfully parsed, with the span of the flags if present.
    Valid(Option<Span>),
    // If the flags argument is a template literal, but impossible to parse (e.g. with substitutions)
    TemplateLiteralNotResolvable,
    // The flags argument is present but not a string literal or template literal expression.
    NoValidArgument,
}

pub fn get_regex_flags_span(arg: Option<&Argument>) -> RegexFlagsParseResult {
    let Some(arg) = arg else {
        return RegexFlagsParseResult::Valid(None);
    };
    let Some(arg) = Argument::as_expression(arg).map(Expression::get_inner_expression) else {
        return RegexFlagsParseResult::NoValidArgument;
    };

    match arg {
        Expression::StringLiteral(flags) => RegexFlagsParseResult::Valid(Some(flags.span)),
        Expression::TemplateLiteral(flags) => {
            if flags.is_no_substitution_template() {
                RegexFlagsParseResult::Valid(Some(flags.span))
            } else {
                RegexFlagsParseResult::TemplateLiteralNotResolvable
            }
        }
        _ => RegexFlagsParseResult::NoValidArgument,
    }
}

pub fn get_regex_pattern_span(arg: Option<&Argument>) -> Option<Span> {
    // note: improvements required for strings used via identifier references
    // Missing or non-string arguments will be runtime errors, but are not covered by this rule.
    let arg = arg?.as_expression()?.get_inner_expression();
    match arg {
        Expression::RegExpLiteral(reg) => {
            reg.regex.pattern.pattern.as_ref().map(|pattern| pattern.span)
        }
        Expression::StringLiteral(pattern) => Some(pattern.span),
        Expression::TemplateLiteral(pattern) if pattern.is_no_substitution_template() => {
            Some(pattern.span)
        }
        _ => None,
    }
}

pub fn run_on_arguments<M>(
    arg1: Option<&Argument>,
    arg2: Option<&Argument>,
    ctx: &LintContext,
    cb: M,
) where
    M: FnOnce(&Pattern<'_>, Span),
{
    let Some(pattern_span) = get_regex_pattern_span(arg1) else {
        return;
    };

    let flag_span = match get_regex_flags_span(arg2) {
        RegexFlagsParseResult::Valid(span) => span,
        RegexFlagsParseResult::NoValidArgument => None,
        // we should not attempt to parse the pattern, as the flags may affect the validity of the pattern.
        RegexFlagsParseResult::TemplateLiteralNotResolvable => return,
    };

    // the argument is a regex literal
    let mut is_regex_literal = false;
    if let Some(Argument::RegExpLiteral(reg)) = arg1 {
        let Some(pat) = &reg.regex.pattern.pattern else {
            return;
        };

        // If new flags are added that can affect the pattern (e.g. "u" or "v"), we need to re-parse the pattern.
        let needs_reparse = flag_span.is_some_and(|flag_span| {
            let flag_text = ctx.source_range(flag_span);
            let same_unicode_mode =
                reg.regex.flags.contains(RegExpFlags::U) == flag_text.contains('u');
            let same_unicode_set_mode =
                reg.regex.flags.contains(RegExpFlags::V) == flag_text.contains('v');

            !same_unicode_mode || !same_unicode_set_mode
        });

        // we can directly use the pattern produced by the parser, without re-parsing the pattern text
        if !needs_reparse {
            cb(pat, reg.span);
            return;
        }

        is_regex_literal = true;
    }

    let allocator = Allocator::default();
    if is_regex_literal {
        if let Some(pat) =
            parse_regex_literal(&allocator, pattern_span, flag_span.map(|span| span.shrink(1)), ctx)
        {
            cb(&pat, pattern_span);
        }
    } else if let Some(pat) = parse_regex_string(&allocator, pattern_span, flag_span, ctx) {
        cb(&pat, pattern_span);
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

fn parse_regex_string<'a>(
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

fn parse_regex_literal<'a>(
    allocator: &'a Allocator,
    pattern_span: Span,
    flags_span: Option<Span>,
    ctx: &'a LintContext<'_>,
) -> Option<Pattern<'a>> {
    let flags_text = flags_span.map(|span| span.source_text(ctx.source_text()));
    let parser = LiteralParser::new(
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
        assert!(parser_ret.diagnostics.is_empty(), "Parse error in: {source}");

        let program = allocator.alloc(parser_ret.program);
        let semantic = SemanticBuilder::new_linter().build(program).semantic;
        let ctx = Rc::new(ContextHost::new(
            "test.js",
            vec![ContextSubHost::new(
                semantic,
                Arc::new(ModuleRecord::default()),
                0,
                ContextSubHostOptions::default(),
            )],
            &allocator,
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
