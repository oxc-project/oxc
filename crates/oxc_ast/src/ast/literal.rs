//! Literals

// NB: `#[span]`, `#[scope(...)]`,`#[visit(...)]` and `#[generate_derive(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in `tasks/ast_tools` and `crates/oxc_traverse/scripts`. See docs in those crates.
// Read [`macro@oxc_ast_macros::ast`] for more information.

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use oxc_allocator::{Box, CloneIn};
use oxc_ast_macros::ast;
use oxc_regular_expression::ast::{Pattern, RegularExpressionFlags};
use oxc_span::{cmp::ContentEq, hash::ContentHash, Atom, GetSpan, GetSpanMut, Span};
use oxc_syntax::number::{BigintBase, NumberBase};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

/// Boolean literal
///
/// <https://tc39.es/ecma262/#prod-BooleanLiteral>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct BooleanLiteral {
    #[serde(flatten)]
    pub span: Span,
    pub value: bool,
}

/// Null literal
///
/// <https://tc39.es/ecma262/#sec-null-literals>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct NullLiteral {
    #[serde(flatten)]
    pub span: Span,
}

/// Numeric literal
///
/// <https://tc39.es/ecma262/#sec-literals-numeric-literals>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct NumericLiteral<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// The value of the number, converted into base 10
    pub value: f64,
    /// The number as it appears in the source code
    pub raw: &'a str,
    /// The base representation used by the literal in the source code
    #[serde(skip)]
    pub base: NumberBase,
}

/// BigInt literal
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct BigIntLiteral<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// The bigint as it appears in the source code
    pub raw: Atom<'a>,
    /// The base representation used by the literal in the source code
    #[serde(skip)]
    pub base: BigintBase,
}

/// Regular expression literal
///
/// <https://tc39.es/ecma262/#sec-literals-regular-expression-literals>
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct RegExpLiteral<'a> {
    #[serde(flatten)]
    pub span: Span,
    // valid regex is printed as {}
    // invalid regex is printed as null, which we can't implement yet
    pub value: EmptyObject,
    pub regex: RegExp<'a>,
}

/// A regular expression
///
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-regexp-regular-expression-objects>
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct RegExp<'a> {
    /// The regex pattern between the slashes
    pub pattern: RegExpPattern<'a>,
    /// Regex flags after the closing slash
    pub flags: RegularExpressionFlags,
}

/// A regular expression pattern
///
/// This pattern may or may not be parsed.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum RegExpPattern<'a> {
    /// Unparsed pattern. Contains string slice of the pattern.
    /// Pattern was not parsed, so may be valid or invalid.
    Raw(&'a str) = 0,
    /// An invalid pattern. Contains string slice of the pattern.
    /// Pattern was parsed and found to be invalid.
    Invalid(&'a str) = 1,
    /// A parsed pattern. Read [Pattern] for more details.
    /// Pattern was parsed and found to be valid.
    Pattern(Box<'a, Pattern<'a>>) = 2,
}

#[ast]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct EmptyObject;

/// String literal
///
/// <https://tc39.es/ecma262/#sec-literals-string-literals>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct StringLiteral<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub value: Atom<'a>,
}
