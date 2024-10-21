//! Literals
#![warn(missing_docs)]

// NB: `#[span]`, `#[scope(...)]`,`#[visit(...)]` and `#[generate_derive(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in `tasks/ast_tools` and `crates/oxc_traverse/scripts`. See docs in those crates.
// Read [`macro@oxc_ast_macros::ast`] for more information.

use std::hash::Hash;

use bitflags::bitflags;
use oxc_allocator::{Box, CloneIn};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_regular_expression::ast::Pattern;
use oxc_span::{cmp::ContentEq, hash::ContentHash, Atom, GetSpan, GetSpanMut, Span};
use oxc_syntax::number::{BigintBase, NumberBase};

/// Boolean literal
///
/// <https://tc39.es/ecma262/#prod-BooleanLiteral>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct BooleanLiteral {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The boolean value itself
    pub value: bool,
}

/// Null literal
///
/// <https://tc39.es/ecma262/#sec-null-literals>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ESTree)]
pub struct NullLiteral {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
}

/// Numeric literal
///
/// <https://tc39.es/ecma262/#sec-literals-numeric-literals>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ESTree)]
pub struct NumericLiteral<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The value of the number, converted into base 10
    pub value: f64,
    /// The number as it appears in source code
    pub raw: &'a str,
    /// The base representation used by the literal in source code
    #[estree(skip)]
    pub base: NumberBase,
}

/// BigInt literal
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct BigIntLiteral<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The bigint as it appears in source code
    pub raw: Atom<'a>,
    /// The base representation used by the literal in source code
    #[estree(skip)]
    pub base: BigintBase,
}

/// Regular expression literal
///
/// <https://tc39.es/ecma262/#sec-literals-regular-expression-literals>
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct RegExpLiteral<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// Placeholder for printing.
    ///
    /// Valid regular expressions are printed as `{}`, while invalid ones are
    /// printed as `null`. Note that invalid regular expressions are not yet
    /// printed properly.
    pub value: EmptyObject,
    /// The parsed regular expression. See [`oxc_regular_expression`] for more
    /// details.
    pub regex: RegExp<'a>,
}

/// A regular expression
///
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-regexp-regular-expression-objects>
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(no_type)]
pub struct RegExp<'a> {
    /// The regex pattern between the slashes
    pub pattern: RegExpPattern<'a>,
    /// Regex flags after the closing slash
    pub flags: RegExpFlags,
}

/// A regular expression pattern
///
/// This pattern may or may not be parsed.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(untagged)]
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

/// An empty object literal (`{}`)
#[ast]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
#[estree(no_type)]
pub struct EmptyObject;

/// String literal
///
/// <https://tc39.es/ecma262/#sec-literals-string-literals>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct StringLiteral<'a> {
    /// Node location in source code
    #[estree(flatten)]
    pub span: Span,
    /// The string as it appears in source code
    pub value: Atom<'a>,
}

bitflags! {
    /// Regular expression flags.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_expressions#advanced_searching_with_flags>
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RegExpFlags: u8 {
        /// Global flag
        ///
        /// Causes the pattern to match multiple times.
        const G = 1 << 0;
        /// Ignore case flag
        ///
        /// Causes the pattern to ignore case.
        const I = 1 << 1;
        /// Multiline flag
        ///
        /// Causes `^` and `$` to match the start/end of each line.
        const M = 1 << 2;
        /// DotAll flag
        ///
        /// Causes `.` to also match newlines.
        const S = 1 << 3;
        /// Unicode flag
        ///
        /// Causes the pattern to treat the input as a sequence of Unicode code points.
        const U = 1 << 4;
        /// Sticky flag
        ///
        /// Perform a "sticky" search that matches starting at the current position in the target string.
        const Y = 1 << 5;
        /// Indices flag
        ///
        /// Causes the regular expression to generate indices for substring matches.
        const D = 1 << 6;
        /// Unicode sets flag
        ///
        /// Similar to the `u` flag, but also enables the `\\p{}` and `\\P{}` syntax.
        /// Added by the [`v` flag proposal](https://github.com/tc39/proposal-regexp-set-notation).
        const V = 1 << 7;
    }
}

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type RegExpFlags = {
    /** Global flag */
    G: 1,
    /** Ignore case flag */
    I: 2,
    /** Multiline flag */
    M: 4,
    /** DotAll flag */
    S: 8,
    /** Unicode flag */
    U: 16,
    /** Sticky flag */
    Y: 32,
    /** Indices flag */
    D: 64,
    /** Unicode sets flag */
    V: 128
};
"#;
