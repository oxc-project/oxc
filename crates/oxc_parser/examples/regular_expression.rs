#![allow(clippy::print_stdout, clippy::cast_possible_truncation)]
use std::{env, fs, path::Path, sync::Arc};

use oxc_allocator::Allocator;
use oxc_ast::{ast, AstKind, Visit};
use oxc_parser::Parser;
use oxc_regular_expression::{FlagsParser, ParserOptions, PatternParser};
use oxc_span::SourceType;

// `cargo run -p oxc_parser --example regular_expression`

fn main() {
    // 1. Get the file content and parse
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);

    let source_text: Arc<str> = Arc::from(fs::read_to_string(path).unwrap());
    let source_type = SourceType::from_path(path).unwrap();

    let allocator = Allocator::default();

    let parser_ret = Parser::new(&allocator, source_text.as_ref(), source_type).parse();
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

                let parsed = PatternParser::new(
                    &allocator,
                    re.regex.pattern.as_str(),
                    ParserOptions {
                        span_offset: re.span.start + 1,
                        unicode_mode: re.regex.flags.contains(ast::RegExpFlags::U)
                            || re.regex.flags.contains(ast::RegExpFlags::V),
                        unicode_sets_mode: re.regex.flags.contains(ast::RegExpFlags::V),
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
            AstKind::NewExpression(new_expr)
                if new_expr
                    .callee
                    .get_identifier_reference()
                    .filter(|ident| ident.name == "RegExp")
                    .is_some() =>
            {
                println!("🍀 {}", new_expr.span.source_text(&self.source_text));

                let pattern = match new_expr.arguments.first() {
                    Some(ast::Argument::StringLiteral(sl)) => &sl.value,
                    Some(ast::Argument::TemplateLiteral(tl))
                        if tl.is_no_substitution_template() =>
                    {
                        &tl.quasi().unwrap()
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

                let flags =
                    FlagsParser::new(&allocator, flags, ParserOptions::default()).parse().unwrap();
                let parsed = PatternParser::new(
                    &allocator,
                    pattern,
                    ParserOptions {
                        span_offset: new_expr.span.start + 12, // = "new RegExp(\"".len()
                        unicode_mode: flags.unicode || flags.unicode_sets,
                        unicode_sets_mode: flags.unicode_sets,
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
            _ => {}
        }
    }
}
