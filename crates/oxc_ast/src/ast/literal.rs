//! Literals

// NB: `#[span]`, `#[scope(...)]` and `#[visit(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in
// `tasks/ast_codegen` and `crates/oxc_traverse/scripts`. See docs in those crates.

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use std::hash::Hash;

use bitflags::bitflags;
use oxc_ast_macros::{ast, CloneIn};
use oxc_span::{Atom, Span};
use oxc_syntax::number::{BigintBase, NumberBase};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

/// Boolean literal
///
/// <https://tc39.es/ecma262/#prod-BooleanLiteral>
#[ast(visit)]
#[derive(Debug, Clone, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
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
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
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
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
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
#[derive(Debug, Clone, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
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
#[derive(Debug, Clone, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
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
#[derive(Debug, Clone, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct RegExp<'a> {
    /// The regex pattern between the slashes
    pub pattern: Atom<'a>,
    /// Regex flags after the closing slash
    pub flags: RegExpFlags,
}

#[ast]
#[derive(Debug, Clone, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct EmptyObject;

/// String literal
///
/// <https://tc39.es/ecma262/#sec-literals-string-literals>
#[ast(visit)]
#[derive(Debug, Clone, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct StringLiteral<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub value: Atom<'a>,
}

bitflags! {
    /// Regular expression flags.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_expressions#advanced_searching_with_flags>
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, CloneIn)]
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
