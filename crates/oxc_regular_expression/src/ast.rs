// NB: `#[span]`, `#[scope(...)]`,`#[visit(...)]` and `#[generate_derive(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in `tasks/ast_tools` and `crates/oxc_traverse/scripts`. See docs in those crates.
// Read [`macro@oxc_ast_macros::ast`] for more information.

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use std::hash::{Hash, Hasher};

use bitflags::bitflags;
use oxc_allocator::{Box, CloneIn, Vec};
use oxc_ast_macros::ast;
use oxc_span::{cmp::ContentEq, hash::ContentHash, Atom, Span};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct RegularExpression<'a> {
    pub span: Span,
    pub pattern: Pattern<'a>,
    pub flags: RegularExpressionFlags,
}

impl<'a> RegularExpression<'a> {
    pub fn flags_span(&self) -> Span {
        Span::new(
            /* + 1 to skip the `/` in the middle */ self.pattern.span.end + 1,
            self.span.end,
        )
    }
}

/// The root of the `PatternParser` result.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct Pattern<'a> {
    pub span: Span,
    pub body: Disjunction<'a>,
}

/// Pile of [`Alternative`]s separated by `|`.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct Disjunction<'a> {
    pub span: Span,
    pub body: Vec<'a, Alternative<'a>>,
}

/// Single unit of `|` separated alternatives.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct Alternative<'a> {
    pub span: Span,
    pub body: Vec<'a, Term<'a>>,
}

/// Single unit of [`Alternative`], containing various kinds.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum Term<'a> {
    // Assertion, QuantifiableAssertion
    BoundaryAssertion(BoundaryAssertion) = 0,
    LookAroundAssertion(Box<'a, LookAroundAssertion<'a>>) = 1,
    // Quantifier
    Quantifier(Box<'a, Quantifier<'a>>) = 2,
    // Atom, ExtendedAtom
    Character(Character) = 3,
    Dot(Dot) = 4,
    CharacterClassEscape(CharacterClassEscape) = 5,
    UnicodePropertyEscape(Box<'a, UnicodePropertyEscape<'a>>) = 6,
    CharacterClass(Box<'a, CharacterClass<'a>>) = 7,
    CapturingGroup(Box<'a, CapturingGroup<'a>>) = 8,
    IgnoreGroup(Box<'a, IgnoreGroup<'a>>) = 9,
    IndexedReference(IndexedReference) = 10,
    NamedReference(Box<'a, NamedReference<'a>>) = 11,
}

/// Simple form of assertion.
/// e.g. `^`, `$`, `\b`, `\B`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct BoundaryAssertion {
    pub span: Span,
    pub kind: BoundaryAssertionKind,
}

#[ast]
#[derive(Debug, Clone, PartialEq)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum BoundaryAssertionKind {
    Start = 0,
    End = 1,
    Boundary = 2,
    NegativeBoundary = 3,
}

/// Lookaround assertion.
/// e.g. `(?=...)`, `(?!...)`, `(?<=...)`, `(?<!...)`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct LookAroundAssertion<'a> {
    pub span: Span,
    pub kind: LookAroundAssertionKind,
    pub body: Disjunction<'a>,
}

#[ast]
#[derive(Debug, Clone, PartialEq)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum LookAroundAssertionKind {
    Lookahead = 0,
    NegativeLookahead = 1,
    Lookbehind = 2,
    NegativeLookbehind = 3,
}

/// Quantifier holding a [`Term`] and its repetition count.
/// e.g. `a*`, `b+`, `c?`, `d{3}`, `e{4,}`, `f{5,6}`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct Quantifier<'a> {
    pub span: Span,
    pub min: u64,
    /// `None` means no upper bound.
    pub max: Option<u64>,
    pub greedy: bool,
    pub body: Term<'a>,
}

/// Single character.
#[ast]
#[derive(Debug, Clone, Copy)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct Character {
    /// This will be invalid position when `UnicodeMode` is disabled and `value` is a surrogate pair.
    pub span: Span,
    pub kind: CharacterKind,
    /// Unicode code point or UTF-16 code unit.
    pub value: u32,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum CharacterKind {
    ControlLetter = 0,
    HexadecimalEscape = 1,
    Identifier = 2,
    Null = 3,
    Octal = 4,
    SingleEscape = 5,
    Symbol = 6,
    UnicodeEscape = 7,
}

/// Character class.
/// e.g. `\d`, `\D`, `\s`, `\S`, `\w`, `\W`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct CharacterClassEscape {
    pub span: Span,
    pub kind: CharacterClassEscapeKind,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum CharacterClassEscapeKind {
    D = 0,
    NegativeD = 1,
    S = 2,
    NegativeS = 3,
    W = 4,
    NegativeW = 5,
}

/// Unicode property.
/// e.g. `\p{ASCII}`, `\P{ASCII}`, `\p{sc=Hiragana}`, `\P{sc=Hiragana}`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct UnicodePropertyEscape<'a> {
    pub span: Span,
    pub negative: bool,
    /// `true` if `UnicodeSetsMode` and `name` matched unicode property of strings.
    pub strings: bool,
    pub name: Atom<'a>,
    pub value: Option<Atom<'a>>,
}

/// The `.`.
#[ast]
#[derive(Debug, Clone, Copy)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct Dot {
    pub span: Span,
}

/// Character class wrapped by `[]`.
/// e.g. `[a-z]`, `[^A-Z]`, `[abc]`, `[a&&b&&c]`, `[[a-z]--x--y]`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct CharacterClass<'a> {
    pub span: Span,
    pub negative: bool,
    pub kind: CharacterClassContentsKind,
    pub body: Vec<'a, CharacterClassContents<'a>>,
}

#[ast]
#[derive(Debug, PartialEq)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum CharacterClassContentsKind {
    Union = 0,
    /// `UnicodeSetsMode` only.
    Intersection = 1,
    /// `UnicodeSetsMode` only.
    Subtraction = 2,
}

#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum CharacterClassContents<'a> {
    CharacterClassRange(Box<'a, CharacterClassRange>) = 0,
    CharacterClassEscape(CharacterClassEscape) = 1,
    UnicodePropertyEscape(Box<'a, UnicodePropertyEscape<'a>>) = 2,
    Character(Character) = 3,
    /// `UnicodeSetsMode` only
    NestedCharacterClass(Box<'a, CharacterClass<'a>>) = 4,
    /// `UnicodeSetsMode` only
    ClassStringDisjunction(Box<'a, ClassStringDisjunction<'a>>) = 5,
}

/// `-` separated range of characters.
/// e.g. `a-z`, `A-Z`, `0-9`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct CharacterClassRange {
    pub span: Span,
    pub min: Character,
    pub max: Character,
}

/// `|` separated string of characters wrapped by `\q{}`.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct ClassStringDisjunction<'a> {
    pub span: Span,
    /// `true` if body is empty or contain [`ClassString`] which `strings` is `true`
    pub strings: bool,
    pub body: Vec<'a, ClassString<'a>>,
}

/// Single unit of [`ClassStringDisjunction`].
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct ClassString<'a> {
    pub span: Span,
    /// `true` if body is empty or contain 2 more characters.
    pub strings: bool,
    pub body: Vec<'a, Character>,
}

/// Named or unnamed capturing group.
/// e.g. `(...)`, `(?<name>...)`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct CapturingGroup<'a> {
    pub span: Span,
    pub name: Option<Atom<'a>>,
    pub body: Disjunction<'a>,
}

/// Pseudo-group for ignoring.
/// e.g. `(?:...)`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct IgnoreGroup<'a> {
    pub span: Span,
    pub enabling_modifiers: Option<ModifierFlags>,
    pub disabling_modifiers: Option<ModifierFlags>,
    pub body: Disjunction<'a>,
}

#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct ModifierFlags {
    pub ignore_case: bool,
    pub sticky: bool,
    pub multiline: bool,
}

/// Backreference by index.
/// e.g. `\1`, `\2`, `\3`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct IndexedReference {
    pub span: Span,
    pub index: u32,
}

/// Backreference by name.
/// e.g. `\k<name>`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct NamedReference<'a> {
    pub span: Span,
    pub name: Atom<'a>,
}

bitflags! {
    /// Regular expression flags.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_expressions#advanced_searching_with_flags>
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RegularExpressionFlags: u8 {
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

impl ContentEq for RegularExpressionFlags {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentHash for RegularExpressionFlags {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(self, state);
    }
}

impl<'alloc> CloneIn<'alloc> for RegularExpressionFlags {
    type Cloned = Self;

    fn clone_in(&self, _: &'alloc oxc_allocator::Allocator) -> Self::Cloned {
        *self
    }
}

#[cfg(feature = "serialize")]
impl serde::Serialize for RegularExpressionFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type RegularExpressionFlags = {
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
