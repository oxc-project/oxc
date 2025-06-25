use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::tester::default_options;

/// Upstream may have modified the AST to include incorrect spans.
/// e.g. <https://github.com/rolldown/rolldown/blob/v1.0.0-beta.19/crates/rolldown/src/utils/ecma_visitors/mod.rs>
#[test]
fn incorrect_ast() {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let source_text = "foo\nvar bar = '测试'";
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    let mut program = ret.program;
    program.span = Span::new(0, 0);
    if let Statement::ExpressionStatement(s) = &mut program.body[0] {
        s.span = Span::new(17, 17);
        if let Expression::Identifier(ident) = &mut s.expression {
            ident.span = Span::new(17, 17);
        }
    }

    let ret = Codegen::new().with_options(default_options()).build(&program);
    assert!(ret.map.is_some(), "sourcemap exists");
}
