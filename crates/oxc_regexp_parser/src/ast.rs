use oxc_allocator::{Box, Vec};
use oxc_span::{Atom as SpanAtom, Span};

// NOTE: Should keep all `enum` size <= 16

/// The root.
#[derive(Debug)]
pub struct RegExpLiteral<'a> {
    pub span: Span,
    pub pattern: Pattern<'a>,
    pub flags: Flags,
}

/// The flags.
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

/// The pattern.
#[derive(Debug)]
pub struct Pattern<'a> {
    pub span: Span,
    pub body: Disjunction<'a>,
}

#[derive(Debug)]
pub struct Disjunction<'a> {
    pub span: Span,
    pub body: Vec<'a, Alternative<'a>>,
}

#[derive(Debug)]
pub struct Alternative<'a> {
    pub span: Span,
    pub body: Vec<'a, RootNode<'a>>,
}

#[derive(Debug)]
pub enum RootNode<'a> {
    BoundaryAssertion(BoundaryAssertion),
    LookAroundAssertion(Box<'a, LookAroundAssertion<'a>>),
    Quantifier(Box<'a, Quantifier<'a>>),
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

#[derive(Debug)]
pub struct Quantifier<'a> {
    pub span: Span,
    pub min: u32,
    pub max: Option<u32>,
    pub greedy: bool,
    pub body: RootNode<'a>,
}

#[derive(Debug)]
pub struct Character {
    pub span: Span,
    pub kind: CharacterKind,
    pub value: u32,
}
#[derive(Debug)]
pub enum CharacterKind {
    ControlLetter,
    HexadecimalEscape,
    Identifier,
    Null,
    Octal,
    SingleEscape,
    Symbol,
    UnicodeEscape,
    UnicodeCodePointEscape, // TODO: Should distinguish from `UnicodeEscape`?
}

#[derive(Debug)]
pub struct Dot {
    pub span: Span,
}

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

#[derive(Debug)]
pub struct UnicodePropertyEscape<'a> {
    pub span: Span,
    pub negative: bool,
    pub name: SpanAtom<'a>,
    pub value: Option<SpanAtom<'a>>,
    // TODO: Should add strings related flag?
}

#[derive(Debug)]
pub struct CharacterClass<'a> {
    pub span: Span,
    pub negative: bool,
    pub kind: CharacterClassKind,
    pub body: Vec<'a, CharacterClassBody<'a>>,
}
#[derive(Debug)]
pub enum CharacterClassKind {
    Union,
    Intersection,
    Subtraction,
}
#[derive(Debug)]
pub enum CharacterClassBody<'a> {
    CharacterClassRange(Box<'a, CharacterClassRange>),
    CharacterClassEscape(CharacterClassEscape),
    UnicodePropertyEscape(Box<'a, UnicodePropertyEscape<'a>>),
    Character(Character),
}
#[derive(Debug)]
pub struct CharacterClassRange {
    pub span: Span,
    pub min: Character,
    pub max: Character,
}

#[derive(Debug)]
pub struct CapturingGroup<'a> {
    pub span: Span,
    pub name: Option<SpanAtom<'a>>,
    pub body: Disjunction<'a>,
}

#[derive(Debug)]
pub struct IgnoreGroup<'a> {
    pub span: Span,
    pub enabling_modifiers: Option<ModifierFlags>,
    pub disabling_modifiers: Option<ModifierFlags>,
    pub body: Disjunction<'a>,
}
#[derive(Debug)]
pub struct ModifierFlags {
    pub span: Span,
    pub ignore_case: bool,
    pub sticky: bool,
    pub multiline: bool,
}

#[derive(Debug)]
pub struct IndexedReference {
    pub span: Span,
    pub index: u32,
}

#[derive(Debug)]
pub struct NamedReference<'a> {
    pub span: Span,
    pub name: SpanAtom<'a>,
}
