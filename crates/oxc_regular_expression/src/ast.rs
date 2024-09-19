// NB: `#[span]`, `#[scope(...)]`,`#[visit(...)]` and `#[generate_derive(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in `tasks/ast_tools` and `crates/oxc_traverse/scripts`. See docs in those crates.
// Read [`macro@oxc_ast_macros::ast`] for more information.

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

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
    pub flags: Flags,
}

#[ast]
#[derive(Debug, Clone)]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct Flags {
    pub span: Span,
    pub global: bool,
    pub ignore_case: bool,
    pub multiline: bool,
    pub unicode: bool,
    pub sticky: bool,
    pub dot_all: bool,
    pub has_indices: bool,
    pub unicode_sets: bool,
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
    // To distinguish leading 0 cases like `\00` and `\000`
    Octal1 = 4,
    Octal2 = 5,
    Octal3 = 6,
    SingleEscape = 7,
    Symbol = 8,
    UnicodeEscape = 9,
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
    /// `true` if `UnicodeSetsMode` and `name` matches unicode property of strings.
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
    /// `true` if:
    /// - `body` contains [`UnicodePropertyEscape`], nested [`CharacterClass`] or [`ClassStringDisjunction`] which `strings` is `true`
    /// - and matches each logic depends on `kind`
    pub strings: bool,
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
    /// `true` if body is empty or contains [`ClassString`] which `strings` is `true`.
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
    /// Group name to be referenced by [`NamedReference`].
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

/// Pattern modifiers in [`IgnoreGroup`].
/// e.g. `(?i:...)`, `(?-s:...)`
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
