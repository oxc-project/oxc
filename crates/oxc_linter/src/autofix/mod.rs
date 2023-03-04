use std::borrow::Cow;

use oxc_ast::Span;

mod fixer;

pub use fixer::Fixer;

#[derive(Debug)]
pub struct Fix<'a> {
    pub content: Cow<'a, str>,
    pub span: Span,
}

impl<'a> Fix<'a> {
    pub const fn delete(span: Span) -> Self {
        Self { content: Cow::Borrowed(""), span }
    }

    pub fn new<T: Into<Cow<'a, str>>>(content: T, span: Span) -> Self {
        Self { content: content.into(), span }
    }
}
