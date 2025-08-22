use oxc_span::Span;

use crate::parser::reader::ast::CodePoint;

#[derive(Debug)]
pub struct StringLiteral {
    #[allow(unused, clippy::allow_attributes)]
    pub span: Span,
    #[allow(unused, clippy::allow_attributes)]
    pub kind: StringLiteralKind,
    pub body: Vec<CodePoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringLiteralKind {
    Double,
    Single,
}
