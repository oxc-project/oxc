use oxc_ast::Span;

mod fixer;

pub use fixer::Fixer;

#[derive(Debug)]
pub struct Fix<'a> {
    pub content: &'a str,
    pub span: Span,
}

impl<'a> Fix<'a> {
    pub const fn delete(span: Span) -> Self {
        Self { content: "", span }
    }
}
