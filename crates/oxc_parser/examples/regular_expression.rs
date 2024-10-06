#![allow(clippy::print_stdout, clippy::cast_possible_truncation)]
use std::{env, fs, path::Path, sync::Arc};

use oxc_allocator::Allocator;
use oxc_ast::{ast, AstKind, Visit};
use oxc_parser::{ParseOptions, Parser};
use oxc_regular_expression::{Parser as RegExpParser, ParserOptions as RegExpParserOptions};
use oxc_span::SourceType;

// `cargo run -p oxc_parser --example regular_expression`

fn main() {
    // 1. Get the file content and parse
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);

    let source_text: Arc<str> = Arc::from(fs::read_to_string(path).unwrap());
    let source_type = SourceType::from_path(path).unwrap();

    let allocator = Allocator::default();
    let options = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };

    let parser_ret =
        Parser::new(&allocator, source_text.as_ref(), source_type).with_options(options).parse();
    if !parser_ret.errors.is_empty() {
        println!("Parsing failed:");
        for error in parser_ret.errors {
            let error = error.with_source_code(Arc::clone(&source_text));
            println!("{error:?}");
        }
        return;
    }

    // Parse regular expressions
    // - RegExpLiteral
    // - new RegExp() with string or template literal if static
    RegularExpressionVisitor { source_text: Arc::clone(&source_text) }
        .visit_program(&parser_ret.program);
}

struct RegularExpressionVisitor {
    source_text: Arc<str>,
}

impl<'a> Visit<'a> for RegularExpressionVisitor {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        let allocator = Allocator::default();

        match kind {
            AstKind::RegExpLiteral(re) => {
                println!("🍀 {}", re.span.source_text(self.source_text.as_ref()));

                println!("{re:#?}");
                println!();
            }
            AstKind::NewExpression(new_expr)
                if new_expr
                    .callee
                    .get_identifier_reference()
                    .filter(|ident| ident.name == "RegExp")
                    .is_some() =>
            {
                println!("🍀 {}", new_expr.span.source_text(&self.source_text));

                let (pattern, pattern_span) = match new_expr.arguments.first() {
                    Some(ast::Argument::StringLiteral(sl)) => (&sl.value, &sl.span),
                    Some(ast::Argument::TemplateLiteral(tl))
                        if tl.is_no_substitution_template() =>
                    {
                        (&tl.quasi().unwrap(), &tl.span)
                    }
                    _ => return,
                };

                let flags = match new_expr.arguments.get(1) {
                    Some(ast::Argument::StringLiteral(sl)) => &sl.value,
                    Some(ast::Argument::TemplateLiteral(tl))
                        if tl.is_no_substitution_template() =>
                    {
                        &tl.quasi().unwrap()
                    }
                    _ => "",
                };

                let parsed = RegExpParser::new(
                    &allocator,
                    pattern,
                    RegExpParserOptions::default()
                        .with_span_offset(pattern_span.start + 1)
                        .with_flags(flags),
                )
                .parse();

                if let Err(error) = parsed {
                    let error = error.with_source_code(Arc::clone(&self.source_text));
                    println!("{error:?}");
                    return;
                }
                println!("{parsed:#?}");
                println!();
            }
            _ => {}
        }
    }
}
