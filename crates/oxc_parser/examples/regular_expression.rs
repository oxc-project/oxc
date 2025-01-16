#![allow(clippy::print_stdout, clippy::cast_possible_truncation)]
use std::{env, fs, path::Path, sync::Arc};

use oxc_allocator::Allocator;
use oxc_ast::{ast::*, Visit};
use oxc_parser::{ParseOptions, Parser};
use oxc_regular_expression::{ConstructorParser as RegExpParser, Options as RegExpParserOptions};
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
    fn visit_reg_exp_literal(&mut self, re: &RegExpLiteral<'a>) {
        println!("🍀 {}", re.span.source_text(self.source_text.as_ref()));

        println!("{re:#?}");
        println!();
    }

    fn visit_new_expression(&mut self, new_expr: &NewExpression<'a>) {
        if new_expr
            .callee
            .get_identifier_reference()
            .filter(|ident| ident.name == "RegExp")
            .is_some()
        {
            println!("🍀 {}", new_expr.span.source_text(&self.source_text));

            let pattern_span = match new_expr.arguments.first() {
                Some(Argument::StringLiteral(sl)) => sl.span,
                _ => return,
            };

            let flags_span = match new_expr.arguments.get(1) {
                Some(Argument::StringLiteral(sl)) => Some(sl.span),
                _ => None,
            };

            let allocator = Allocator::default();
            let parsed = RegExpParser::new(
                &allocator,
                pattern_span.source_text(&self.source_text),
                flags_span.map(|span| span.source_text(&self.source_text)),
                RegExpParserOptions {
                    pattern_span_offset: pattern_span.start,
                    flags_span_offset: flags_span.map_or(0, |span| span.start),
                },
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
    }
}
