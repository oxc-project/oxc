use oxc_allocator::Vec;
use oxc_span::Span;

#[derive(Debug)]
pub struct StringLiteral<'a> {
    #[allow(dead_code)]
    pub span: Span,
    #[allow(dead_code)]
    pub kind: StringLiteralKind,
    pub body: Vec<'a, CodePoint>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StringLiteralKind {
    Double,
    Single,
}

#[derive(Debug)]
pub struct CodePoint {
    pub span: Span,
    pub value: u32,
}
