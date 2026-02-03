//! Literals

// NB: `#[span]`, `#[scope(...)]`,`#[visit(...)]` and `#[generate_derive(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in `tasks/ast_tools` and `crates/oxc_traverse/scripts`. See docs in those crates.
// Read [`macro@oxc_ast_macros::ast`] for more information.

use std::hash::Hash;

use bitflags::bitflags;
use oxc_allocator::{Box, CloneIn, Dummy, TakeIn, UnstableAddress};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_regular_expression::ast::Pattern;
use oxc_span::{Atom, ContentEq, GetSpan, GetSpanMut, Span};
use oxc_syntax::{
    node::NodeId,
    number::{BigintBase, NumberBase},
};

/// Boolean literal
///
/// <https://tc39.es/ecma262/#prod-BooleanLiteral>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, ContentEq, ESTree, UnstableAddress)]
#[estree(rename = "Literal", add_fields(raw = BooleanLiteralRaw))]
pub struct BooleanLiteral {
    /// Unique identifier for this AST node.
    pub node_id: NodeId,
    /// Node location in source code.
    pub span: Span,
    /// The boolean value itself
    pub value: bool,
}

/// Null literal
///
/// <https://tc39.es/ecma262/#sec-null-literals>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, Dummy, TakeIn, GetSpan, GetSpanMut, ContentEq, ESTree, UnstableAddress)]
#[estree(rename = "Literal", add_fields(value = Null, raw = NullLiteralRaw))]
pub struct NullLiteral {
    /// Unique identifier for this AST node.
    pub node_id: NodeId,
    /// Node location in source code.
    pub span: Span,
}

/// Numeric literal
///
/// <https://tc39.es/ecma262/#sec-literals-numeric-literals>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, Dummy, TakeIn, ContentEq, GetSpan, GetSpanMut, ESTree, UnstableAddress)]
#[estree(rename = "Literal")]
pub struct NumericLiteral<'a> {
    /// Unique identifier for this AST node.
    pub node_id: NodeId,
    /// Node location in source code.
    pub span: Span,
    /// The value of the number, converted into base 10
    pub value: f64,
    /// The number as it appears in source code
    ///
    /// `None` when this ast node is not constructed from the parser.
    #[content_eq(skip)]
    #[estree(json_safe)]
    pub raw: Option<Atom<'a>>,
    /// The base representation used by the literal in source code
    #[content_eq(skip)]
    #[estree(skip)]
    pub base: NumberBase,
}

/// String literal
///
/// <https://tc39.es/ecma262/#sec-literals-string-literals>
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, Dummy, TakeIn, ContentEq, GetSpan, GetSpanMut, ESTree, UnstableAddress)]
#[estree(rename = "Literal")]
pub struct StringLiteral<'a> {
    /// Unique identifier for this AST node.
    pub node_id: NodeId,
    /// Node location in source code.
    pub span: Span,
    /// The value of the string.
    ///
    /// Any escape sequences in the raw code are unescaped.
    #[estree(via = StringLiteralValue)]
    pub value: Atom<'a>,

    /// The raw string as it appears in source code.
    ///
    /// `None` when this ast node is not constructed from the parser.
    #[content_eq(skip)]
    pub raw: Option<Atom<'a>>,

    /// The string value contains lone surrogates.
    ///
    /// `value` is encoded using `\u{FFFD}` (the lossy replacement character) as an escape character.
    /// Lone surrogates are encoded as `\u{FFFD}XXXX`, where `XXXX` is the code unit in hex.
    /// The lossy escape character itself is encoded as `\u{FFFD}fffd`.
    #[builder(default)]
    #[estree(skip)]
    pub lone_surrogates: bool,
}

/// BigInt literal
#[ast(visit)]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, Dummy, TakeIn, ContentEq, GetSpan, GetSpanMut, ESTree, UnstableAddress)]
#[estree(rename = "Literal", add_fields(bigint = BigIntLiteralBigint))]
pub struct BigIntLiteral<'a> {
    /// Unique identifier for this AST node.
    pub node_id: NodeId,
    /// Node location in source code.
    pub span: Span,
    /// Bigint value in base 10 with no underscores
    #[estree(via = BigIntLiteralValue)]
    pub value: Atom<'a>,
    /// The bigint as it appears in source code
    #[content_eq(skip)]
    #[estree(json_safe)]
    pub raw: Option<Atom<'a>>,
    /// The base representation used by the literal in source code
    #[content_eq(skip)]
    #[estree(skip)]
    pub base: BigintBase,
}

/// Regular expression literal
///
/// <https://tc39.es/ecma262/#sec-literals-regular-expression-literals>
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, ContentEq, GetSpan, GetSpanMut, ESTree, UnstableAddress)]
#[estree(
    rename = "Literal",
    add_fields(value = RegExpLiteralValue),
    field_order(value, raw, regex, span),
)]
pub struct RegExpLiteral<'a> {
    /// Unique identifier for this AST node.
    pub node_id: NodeId,
    /// Node location in source code.
    pub span: Span,
    /// The parsed regular expression. See [`oxc_regular_expression`] for more
    /// details.
    pub regex: RegExp<'a>,
    /// The regular expression as it appears in source code
    ///
    /// `None` when this ast node is not constructed from the parser.
    #[content_eq(skip)]
    pub raw: Option<Atom<'a>>,
}

/// A regular expression
///
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-regexp-regular-expression-objects>
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, Dummy, TakeIn, ContentEq, ESTree)]
#[estree(no_type, ts_alias = "{ pattern: string; flags: string; }")]
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
#[generate_derive(CloneIn, Dummy, TakeIn, ContentEq, ESTree)]
#[estree(no_type, flatten)]
pub struct RegExpPattern<'a> {
    /// The regexp's pattern as a string.
    ///
    /// If `pattern` is defined, `pattern` and `text` must be in sync.
    /// i.e. If you alter the regexp by mutating `pattern`, you must regenerate `text` to match it,
    /// using `format_atom!("{}", &pattern)`.
    ///
    /// `oxc_codegen` ignores `pattern` field, and prints `text`.
    #[estree(rename = "pattern")]
    pub text: Atom<'a>,
    /// Parsed regexp pattern
    #[content_eq(skip)]
    #[estree(skip)]
    pub pattern: Option<Box<'a, Pattern<'a>>>,
}

/// The list of valid regular expression flags.
pub const REGEXP_FLAGS_LIST: &str = "gimsuydv";

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

/// Dummy type to communicate the content of `RegExpFlags` to `oxc_ast_tools`.
#[ast(foreign = RegExpFlags)]
#[generate_derive(ESTree)]
#[estree(no_type, via = RegExpFlagsConverter)]
#[expect(dead_code)]
struct RegExpFlagsAlias(#[estree(skip)] u8);
