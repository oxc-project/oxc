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
