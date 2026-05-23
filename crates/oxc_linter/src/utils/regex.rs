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
        Expression::StringLiteral(pattern) => Some(pattern.span),
        Expression::TemplateLiteral(pattern) if pattern.is_no_substitution_template() => {
            Some(pattern.span)
        }
        _ => None,
    }
}

fn run_on_arguments<M>(arg1: Option<&Argument>, arg2: Option<&Argument>, ctx: &LintContext, cb: M)
where
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

    let allocator = Allocator::default();
    if let Some(pat) = parse_regex(&allocator, pattern_span, flag_span, ctx) {
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
