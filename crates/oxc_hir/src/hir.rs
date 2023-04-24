use oxc_allocator::Vec;
use oxc_ast::{SourceType, Span};
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct Program<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub source_type: SourceType,
    pub directives: Vec<'a, Directive<'a>>,
    // pub body: Vec<'a, Statement<'a>>,
}

/// Directive Prologue
#[derive(Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct Directive<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    // pub expression: StringLiteral,
    // directives should always use the unescaped raw string
    pub directive: &'a str,
}

// ... All AST copied over
