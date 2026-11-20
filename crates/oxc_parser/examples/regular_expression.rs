#![expect(clippy::print_stdout)]

use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::VisitJs;
use oxc_parser::{ParseOptions, Source, SourceFileError};
use oxc_regular_expression::{ConstructorParser as RegExpParser, Options as RegExpParserOptions};

// `cargo run -p oxc_parser --example regular_expression`

fn main() -> Result<(), SourceFileError> {
    // Get the file content and parse
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);

    let allocator = Allocator::default();
    let source = Source::from_file(path, &allocator)?;

    let options = ParseOptions {
        parse_regular_expression: true,
        enable_ident_hashes: false,
        ..ParseOptions::default()
    };
    let parser_ret = source.parse_with_options(options);

    let source_text = parser_ret.program.source_text;

    if !parser_ret.diagnostics.is_empty() {
        println!("Parsing failed:");
        for error in parser_ret.diagnostics {
            println!("{}", error.render_with_source_code(source_text));
        }
        return Ok(());
    }

    // Parse regular expressions
    // - RegExpLiteral
    // - new RegExp() with string or template literal if static
    RegularExpressionVisitor { source_text }.visit_program(&parser_ret.program);

    Ok(())
}

struct RegularExpressionVisitor<'a> {
    source_text: &'a str,
}

impl<'a> VisitJs<'a> for RegularExpressionVisitor<'a> {
    fn visit_reg_exp_literal(&mut self, re: &RegExpLiteral<'a>) {
        println!("🍀 {}", re.span.source_text(self.source_text));

        println!("{re:#?}");
        println!();
    }

    fn visit_new_expression(&mut self, new_expr: &NewExpression<'a>) {
        if new_expr.callee.get_identifier_reference().is_some_and(|ident| ident.name == "RegExp") {
            println!("🍀 {}", new_expr.span.source_text(self.source_text));

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
                pattern_span.source_text(self.source_text),
                flags_span.map(|span| span.source_text(self.source_text)),
                RegExpParserOptions {
                    pattern_span_offset: pattern_span.start,
                    flags_span_offset: flags_span.map_or(0, |span| span.start),
                },
            )
            .parse();

            if let Err(error) = parsed {
                println!("{}", error.render_with_source_code(self.source_text));
                return;
            }
            println!("{parsed:#?}");
            println!();
        }
    }
}
