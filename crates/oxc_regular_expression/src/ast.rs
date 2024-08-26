use oxc_allocator::{Box, Vec};
use oxc_span::{Atom as SpanAtom, Span};

#[derive(Debug)]
pub struct RegularExpression<'a> {
    pub span: Span,
    pub pattern: Pattern<'a>,
    pub flags: Flags,
}

#[derive(Debug)]
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
#[derive(Debug)]
pub struct Pattern<'a> {
    pub span: Span,
    pub body: Disjunction<'a>,
}

/// Pile of [`Alternative`]s separated by `|`.
#[derive(Debug)]
pub struct Disjunction<'a> {
    pub span: Span,
    pub body: Vec<'a, Alternative<'a>>,
}

/// Single unit of `|` separated alternatives.
#[derive(Debug)]
pub struct Alternative<'a> {
    pub span: Span,
    pub body: Vec<'a, Term<'a>>,
}

/// Single unit of [`Alternative`], containing various kinds.
#[derive(Debug)]
pub enum Term<'a> {
    // Assertion, QuantifiableAssertion
    BoundaryAssertion(BoundaryAssertion),
    LookAroundAssertion(Box<'a, LookAroundAssertion<'a>>),
    // Quantifier
    Quantifier(Box<'a, Quantifier<'a>>),
    // Atom, ExtendedAtom
    Character(Character),
    Dot(Dot),
    CharacterClassEscape(CharacterClassEscape),
    UnicodePropertyEscape(Box<'a, UnicodePropertyEscape<'a>>),
    CharacterClass(Box<'a, CharacterClass<'a>>),
    CapturingGroup(Box<'a, CapturingGroup<'a>>),
    IgnoreGroup(Box<'a, IgnoreGroup<'a>>),
    IndexedReference(IndexedReference),
    NamedReference(Box<'a, NamedReference<'a>>),
}

/// Simple form of assertion.
/// e.g. `^`, `$`, `\b`, `\B`
#[derive(Debug)]
pub struct BoundaryAssertion {
    pub span: Span,
    pub kind: BoundaryAssertionKind,
}
#[derive(Debug)]
pub enum BoundaryAssertionKind {
    Start,
    End,
    Boundary,
    NegativeBoundary,
}

/// Lookaround assertion.
/// e.g. `(?=...)`, `(?!...)`, `(?<=...)`, `(?<!...)`
#[derive(Debug)]
pub struct LookAroundAssertion<'a> {
    pub span: Span,
    pub kind: LookAroundAssertionKind,
    pub body: Disjunction<'a>,
}
#[derive(Debug)]
pub enum LookAroundAssertionKind {
    Lookahead,
    NegativeLookahead,
    Lookbehind,
    NegativeLookbehind,
}

/// Quantifier holding a [`Term`] and its repetition count.
/// e.g. `a*`, `b+`, `c?`, `d{3}`, `e{4,}`, `f{5,6}`
#[derive(Debug)]
pub struct Quantifier<'a> {
    pub span: Span,
    pub min: u32,
    /// `None` means no upper bound.
    pub max: Option<u32>,
    pub greedy: bool,
    pub body: Term<'a>,
}

/// Single character.
#[derive(Debug, Copy, Clone)]
pub struct Character {
    pub span: Span,
    pub kind: CharacterKind,
    pub value: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CharacterKind {
    ControlLetter,
    HexadecimalEscape,
    Identifier,
    Null,
    Octal,
    SingleEscape,
    Symbol,
    /// In non `UnicodeMode`, some `Symbol` is marked as `SurrogatePairs`.
    SurrogatePairs,
    UnicodeEscape,
}

/// Character class.
/// e.g. `\d`, `\D`, `\s`, `\S`, `\w`, `\W`
#[derive(Debug)]
pub struct CharacterClassEscape {
    pub span: Span,
    pub kind: CharacterClassEscapeKind,
}

#[derive(Debug)]
pub enum CharacterClassEscapeKind {
    D,
    NegativeD,
    S,
    NegativeS,
    W,
    NegativeW,
}

/// Unicode property.
/// e.g. `\p{ASCII}`, `\P{ASCII}`, `\p{sc=Hiragana}`, `\P{sc=Hiragana}`
#[derive(Debug)]
pub struct UnicodePropertyEscape<'a> {
    pub span: Span,
    pub negative: bool,
    /// `true` if `UnicodeSetsMode` and `name` matched unicode property of strings.
    pub strings: bool,
    pub name: SpanAtom<'a>,
    pub value: Option<SpanAtom<'a>>,
}

/// The `.`.
#[derive(Debug)]
pub struct Dot {
    pub span: Span,
}

/// Character class wrapped by `[]`.
/// e.g. `[a-z]`, `[^A-Z]`, `[abc]`, `[a&&b&&c]`, `[[a-z]--x--y]`
#[derive(Debug)]
pub struct CharacterClass<'a> {
    pub span: Span,
    pub negative: bool,
    pub kind: CharacterClassContentsKind,
    pub body: Vec<'a, CharacterClassContents<'a>>,
}

#[derive(Debug)]
pub enum CharacterClassContentsKind {
    Union,
    /// `UnicodeSetsMode` only.
    Intersection,
    /// `UnicodeSetsMode` only.
    Subtraction,
}

#[derive(Debug)]
pub enum CharacterClassContents<'a> {
    CharacterClassRange(Box<'a, CharacterClassRange>),
    CharacterClassEscape(CharacterClassEscape),
    UnicodePropertyEscape(Box<'a, UnicodePropertyEscape<'a>>),
    Character(Character),
    /// `UnicodeSetsMode` only
    NestedCharacterClass(Box<'a, CharacterClass<'a>>),
    /// `UnicodeSetsMode` only
    ClassStringDisjunction(Box<'a, ClassStringDisjunction<'a>>),
}

/// `-` separated range of characters.
/// e.g. `a-z`, `A-Z`, `0-9`
#[derive(Debug)]
pub struct CharacterClassRange {
    pub span: Span,
    pub min: Character,
    pub max: Character,
}

/// `|` separated string of characters wrapped by `\q{}`.
#[derive(Debug)]
pub struct ClassStringDisjunction<'a> {
    pub span: Span,
    /// `true` if body is empty or contain [`ClassString`] which `strings` is `true`
    pub strings: bool,
    pub body: Vec<'a, ClassString<'a>>,
}

/// Single unit of [`ClassStringDisjunction`].
#[derive(Debug)]
pub struct ClassString<'a> {
    pub span: Span,
    /// `true` if body is empty or contain 2 more characters.
    pub strings: bool,
    pub body: Vec<'a, Character>,
}

/// Named or unnamed capturing group.
/// e.g. `(...)`, `(?<name>...)`
#[derive(Debug)]
pub struct CapturingGroup<'a> {
    pub span: Span,
    pub name: Option<SpanAtom<'a>>,
    pub body: Disjunction<'a>,
}

/// Pseudo-group for ignoring.
/// e.g. `(?:...)`
#[derive(Debug)]
pub struct IgnoreGroup<'a> {
    pub span: Span,
    pub enabling_modifiers: Option<ModifierFlags>,
    pub disabling_modifiers: Option<ModifierFlags>,
    pub body: Disjunction<'a>,
}

#[derive(Debug)]
pub struct ModifierFlags {
    pub ignore_case: bool,
    pub sticky: bool,
    pub multiline: bool,
}

/// Backreference by index.
/// e.g. `\1`, `\2`, `\3`
#[derive(Debug)]
pub struct IndexedReference {
    pub span: Span,
    pub index: u32,
}

/// Backreference by name.
/// e.g. `\k<name>`
#[derive(Debug)]
pub struct NamedReference<'a> {
    pub span: Span,
    pub name: SpanAtom<'a>,
}
