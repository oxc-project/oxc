#![expect(clippy::print_stdout)]
use std::{env, fs, path::Path, sync::Arc};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::VisitJs;
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
    if !parser_ret.diagnostics.is_empty() {
        println!("Parsing failed:");
        for error in parser_ret.diagnostics {
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

impl<'a> VisitJs<'a> for RegularExpressionVisitor {
    fn visit_reg_exp_literal(&mut self, re: &RegExpLiteral<'a>) {
        println!("🍀 {}", re.span.source_text(self.source_text.as_ref()));

        println!("{re:#?}");
        println!();
    }

    fn visit_new_expression(&mut self, new_expr: &NewExpression<'a>) {
        if new_expr.callee.get_identifier_reference().is_some_and(|ident| ident.name == "RegExp") {
            println!("🍀 {}", new_expr.span.source_text(&self.source_text));

            let Some(sl) = new_expr
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(Expression::as_string_literal)
            else {
                return;
            };
            let pattern_span = sl.span;

            let flags_span = new_expr
                .arguments
                .get(1)
                .and_then(Argument::as_expression)
                .and_then(Expression::as_string_literal)
                .map(|sl| sl.span);

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
