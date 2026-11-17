//! Validate spans after an AST is complete, rather than during node construction.
//!
//! Enabled by the `conformance` feature and called explicitly by conformance runners.

use oxc_ast::ast::Program;
use oxc_span::Span;

use crate::Visit;

/// Check every AST span, including comments, in a completed program.
///
/// # Panics
/// Panics if any span ends before it starts.
pub fn assert_valid_spans(program: &Program<'_>) {
    let mut validator = SpanValidator;
    validator.visit_program(program);
    // Comments are not included in the AST walk.
    for comment in &program.comments {
        validator.visit_span(&comment.span);
    }
}

struct SpanValidator;

impl Visit<'_> for SpanValidator {
    fn visit_span(&mut self, span: &Span) {
        assert!(span.start <= span.end, "Invalid span: {span:?}");
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::{Allocator, Dummy};
    use oxc_ast::{
        ast::{Comment, CommentKind, Expression, Program, Statement},
        builder::AstBuilder,
    };
    use oxc_span::{SPAN, Span};

    use super::assert_valid_spans;

    #[test]
    fn accepts_empty_and_generated_spans() {
        let allocator = Allocator::default();
        let builder = AstBuilder::new(&allocator);
        let mut program = Program::dummy(&allocator);
        let expression = Expression::new_boolean_literal(Span::empty(5), true, &builder);
        program.body.push(Statement::new_expression_statement(SPAN, expression, &builder));
        assert_valid_spans(&program);
    }

    #[test]
    #[should_panic(expected = "Invalid span: Span { start: 5, end: 3 }")]
    fn rejects_reversed_program_span() {
        let allocator = Allocator::default();
        let mut program = Program::dummy(&allocator);
        program.span = Span::new(5, 3);
        assert_valid_spans(&program);
    }

    #[test]
    #[should_panic(expected = "Invalid span: Span { start: 5, end: 3 }")]
    fn rejects_reversed_nested_span() {
        let allocator = Allocator::default();
        let builder = AstBuilder::new(&allocator);
        let mut program = Program::dummy(&allocator);
        let expression = Expression::new_boolean_literal(Span::new(5, 3), true, &builder);
        program.body.push(Statement::new_expression_statement(SPAN, expression, &builder));
        assert_valid_spans(&program);
    }

    #[test]
    #[should_panic(expected = "Invalid span: Span { start: 5, end: 3 }")]
    fn rejects_reversed_comment_span() {
        let allocator = Allocator::default();
        let mut program = Program::dummy(&allocator);
        program.comments.push(Comment::new(5, 3, CommentKind::Line));
        assert_valid_spans(&program);
    }
}
