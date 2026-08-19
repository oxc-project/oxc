use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn parser_completes_comment_attachments_without_semantic() {
    let allocator = Allocator::default();
    let source = "/* statement */ foo(/* argument */ bar); // trailing\nbaz();";
    let ret = Parser::new(&allocator, source, SourceType::default()).parse();
    assert!(ret.diagnostics.is_empty());
    let program = ret.program;

    let parser_statement_id = program.body[0].node_id();
    let statement = &program.body[0];
    let statement_comments = program.comments.node_comments(parser_statement_id).unwrap();
    assert_eq!(statement_comments.leading.len(), 1);
    assert_eq!(statement_comments.trailing.len(), 1);
    let Statement::ExpressionStatement(statement) = statement else { unreachable!() };
    let Expression::CallExpression(call) = &statement.expression else { unreachable!() };
    let Argument::Identifier(argument) = &call.arguments[0] else { unreachable!() };
    let argument_comments = program.comments.node_comments(argument.node_id()).unwrap();
    assert_eq!(argument_comments.leading.len(), 1);
}
