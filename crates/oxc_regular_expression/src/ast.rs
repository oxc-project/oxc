use bitflags::bitflags;

use oxc_allocator::{Box, CloneIn, GetAddress, Vec};
use oxc_ast_macros::ast;
use oxc_span::{Atom, ContentEq, Span};

/// The root of the `PatternParser` result.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
pub struct Pattern<'a> {
    pub span: Span,
    pub body: Disjunction<'a>,
}

/// Pile of [`Alternative`]s separated by `|`.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
pub struct Disjunction<'a> {
    pub span: Span,
    pub body: Vec<'a, Alternative<'a>>,
}

/// Single unit of `|` separated alternatives.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
pub struct Alternative<'a> {
    pub span: Span,
    pub body: Vec<'a, Term<'a>>,
}

/// Single unit of [`Alternative`], containing various kinds.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
pub enum Term<'a> {
    // Assertion, QuantifiableAssertion
    BoundaryAssertion(Box<'a, BoundaryAssertion>) = 0,
    LookAroundAssertion(Box<'a, LookAroundAssertion<'a>>) = 1,
    // Quantifier
    Quantifier(Box<'a, Quantifier<'a>>) = 2,
    // Atom, ExtendedAtom
    Character(Box<'a, Character>) = 3,
    Dot(Dot) = 4,
    CharacterClassEscape(Box<'a, CharacterClassEscape>) = 5,
    UnicodePropertyEscape(Box<'a, UnicodePropertyEscape<'a>>) = 6,
    CharacterClass(Box<'a, CharacterClass<'a>>) = 7,
    CapturingGroup(Box<'a, CapturingGroup<'a>>) = 8,
    IgnoreGroup(Box<'a, IgnoreGroup<'a>>) = 9,
    IndexedReference(Box<'a, IndexedReference>) = 10,
    NamedReference(Box<'a, NamedReference<'a>>) = 11,
}

/// Simple form of assertion.
/// e.g. `^`, `$`, `\b`, `\B`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
pub struct BoundaryAssertion {
    pub span: Span,
    pub kind: BoundaryAssertionKind,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq)]
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
#[generate_derive(CloneIn, ContentEq)]
pub struct LookAroundAssertion<'a> {
    pub span: Span,
    pub kind: LookAroundAssertionKind,
    pub body: Disjunction<'a>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq)]
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
#[generate_derive(CloneIn, ContentEq)]
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
#[generate_derive(CloneIn, ContentEq)]
pub struct Character {
    /// This will be invalid position when `UnicodeMode` is disabled and `value` is a surrogate pair.
    pub span: Span,
    pub kind: CharacterKind,
    /// Unicode code point or UTF-16 code unit.
    pub value: u32,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq)]
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
#[generate_derive(CloneIn, ContentEq)]
pub struct CharacterClassEscape {
    pub span: Span,
    pub kind: CharacterClassEscapeKind,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq)]
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
#[generate_derive(CloneIn, ContentEq)]
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
#[generate_derive(CloneIn, ContentEq)]
pub struct Dot {
    pub span: Span,
}

/// Character class wrapped by `[]`.
/// e.g. `[a-z]`, `[^A-Z]`, `[abc]`, `[a&&b&&c]`, `[[a-z]--x--y]`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq)]
pub enum CharacterClassContentsKind {
    Union = 0,
    /// `UnicodeSetsMode` only.
    Intersection = 1,
    /// `UnicodeSetsMode` only.
    Subtraction = 2,
}

#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq, GetAddress)]
pub enum CharacterClassContents<'a> {
    CharacterClassRange(Box<'a, CharacterClassRange>) = 0,
    CharacterClassEscape(Box<'a, CharacterClassEscape>) = 1,
    UnicodePropertyEscape(Box<'a, UnicodePropertyEscape<'a>>) = 2,
    Character(Box<'a, Character>) = 3,
    /// `UnicodeSetsMode` only
    NestedCharacterClass(Box<'a, CharacterClass<'a>>) = 4,
    /// `UnicodeSetsMode` only
    ClassStringDisjunction(Box<'a, ClassStringDisjunction<'a>>) = 5,
}

/// `-` separated range of characters.
/// e.g. `a-z`, `A-Z`, `0-9`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
pub struct CharacterClassRange {
    pub span: Span,
    pub min: Character,
    pub max: Character,
}

/// `|` separated string of characters wrapped by `\q{}`.
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
pub struct ClassStringDisjunction<'a> {
    pub span: Span,
    /// `true` if body is empty or contains [`ClassString`] which `strings` is `true`.
    pub strings: bool,
    pub body: Vec<'a, ClassString<'a>>,
}

/// Single unit of [`ClassStringDisjunction`].
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
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
#[generate_derive(CloneIn, ContentEq)]
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
#[generate_derive(CloneIn, ContentEq)]
pub struct IgnoreGroup<'a> {
    pub span: Span,
    pub modifiers: Option<Modifiers>,
    pub body: Disjunction<'a>,
}

/// Modifiers in [`IgnoreGroup`].
/// e.g. `i` in `(?i:...)`, `-s` in `(?-s:...)`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
pub struct Modifiers {
    pub span: Span,
    pub enabling: Modifier,
    pub disabling: Modifier,
}

bitflags! {
    /// Each part of modifier in [`Modifiers`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Modifier: u8 {
        /// Ignore case flag
        const I = 1 << 0;
        /// Multiline flag
        const M = 1 << 1;
        /// DotAll flag
        const S = 1 << 2;
    }
}
/// Dummy type to communicate the content of `Modifier` to `oxc_ast_tools`.
#[ast(foreign = Modifier)]
#[expect(dead_code)]
struct ModifierAlias(u8);

/// Backreference by index.
/// e.g. `\1`, `\2`, `\3`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
pub struct IndexedReference {
    pub span: Span,
    pub index: u32,
}

/// Backreference by name.
/// e.g. `\k<name>`
#[ast]
#[derive(Debug)]
#[generate_derive(CloneIn, ContentEq)]
pub struct NamedReference<'a> {
    pub span: Span,
    pub name: Atom<'a>,
}

// See `oxc_ast/src/lib.rs` for the details
#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    assert!(size_of::<Term>() == 16);
    assert!(size_of::<CharacterClassContents>() == 16);
}
