use oxc_span::Span;

use crate::parser::reader::CodePoint;

#[derive(Debug)]
pub struct TemplateLiteral {
    #[allow(unused, clippy::allow_attributes)]
    pub span: Span,
    pub body: Vec<CodePoint>,
}
