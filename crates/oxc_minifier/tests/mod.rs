#![feature(let_chains)]

use oxc_allocator::Allocator;
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions, PrinterOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use walkdir::WalkDir;

#[test]
fn test() {
    let files = WalkDir::new("tests/fixtures")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .collect::<Vec<_>>();
    assert!(!files.is_empty());
    for file in files {
        let path = file.path();
        let source_text = std::fs::read_to_string(path).unwrap();
        let source_type = SourceType::from_path(path).unwrap();
        let allocator = Allocator::default();
        let parser_return = Parser::new(&allocator, &source_text, source_type).parse();
        let program = allocator.alloc(parser_return.program);
        TestSuite::from_program(&source_text, program).execute_tests();
    }
}

#[derive(Debug, Default)]
struct TestSuite {
    tests: Vec<TestCase>,
}

impl TestSuite {
    fn from_program<'a>(source_text: &str, program: &'a Program<'a>) -> Self {
        let mut tests = vec![];
        for stmt in &program.body {
            if let Statement::LabeledStatement(labeled_stmt) = stmt {
                let test_case = TestCase::from_labeled_statement(source_text, labeled_stmt);
                tests.push(test_case);
            }
        }
        Self { tests }
    }

    fn execute_tests(&self) {
        assert!(!self.tests.is_empty());
        for test in &self.tests {
            test.execute_test();
        }
    }
}

#[derive(Debug, Default)]
struct TestCase {
    name: String,
    compress_options: CompressOptions,
    input: Box<str>,
    expect: Box<str>,
}

impl TestCase {
    fn execute_test(&self) {
        fn remove_whitespace(s: &str) -> String {
            s.replace(char::is_whitespace, "")
        }

        let source_type = SourceType::default();
        let options = MinifierOptions {
            mangle: false,
            compress: self.compress_options,
            print: PrinterOptions { minify_whitespace: false, ..PrinterOptions::default() },
        };
        let minified_source_text = Minifier::new(self.input.as_ref(), source_type, options).build();
        assert_eq!(
            remove_whitespace(minified_source_text.as_str()),
            remove_whitespace(self.expect.as_ref()),
            "{} {:?}",
            &self.name,
            &self.compress_options
        );
    }

    fn from_labeled_statement<'a>(
        source_text: &str,
        labeled_stmt: &'a LabeledStatement<'a>,
    ) -> Self {
        let name = labeled_stmt.label.name.to_string();
        let mut options = CompressOptions::default();
        let mut input = String::new().into_boxed_str();
        let mut expect = String::new().into_boxed_str();

        if let Statement::BlockStatement(block_stmt) = &labeled_stmt.body {
            for stmt in &block_stmt.body {
                // Parse options
                if let Statement::ExpressionStatement(expr_stmt) = stmt
                && let Expression::AssignmentExpression(assign_expr) = &expr_stmt.expression
                && let AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::AssignmentTargetIdentifier(ident)) = &assign_expr.left
                && ident.name == "options"
                && let Expression::ObjectExpression(object_expr) = &assign_expr.right {
                    options = Self::parse_options(object_expr);
                }

                // Parse input / expect
                if let Statement::LabeledStatement(labeled_stmt) = stmt
                && let Statement::BlockStatement(block_stmt) = &labeled_stmt.body {
                    let code = block_stmt.span.source_text(source_text).to_string().into_boxed_str();
                    match labeled_stmt.label.name.as_str() {
                        "input" => {
                            input = code;
                        }
                        "expect" => {
                            expect = code;
                        }
                        _ => {}
                    }
                }
            }
        }

        Self { name, compress_options: options, input, expect }
    }

    #[allow(clippy::single_match)]
    fn parse_options<'a>(object_expr: &'a ObjectExpression<'a>) -> CompressOptions {
        let mut options = CompressOptions::default();
        for object_property in &object_expr.properties {
            if let ObjectPropertyKind::ObjectProperty(property) = object_property
                && let Some(name) = property.key.static_name() {
                match name.as_str() {
                    "drop_debugger" => {
                        options.drop_debugger = Self::get_boolean(&property.value);
                    }
                    _ => {}
                }
            }
        }
        options
    }

    fn get_boolean<'a>(expr: &'a Expression<'a>) -> bool {
        if let Expression::BooleanLiteral(boolean_literal) = expr {
            boolean_literal.value
        } else {
            false
        }
    }
}
