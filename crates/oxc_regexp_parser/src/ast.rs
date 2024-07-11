use oxc_allocator::{Box, Vec};
use oxc_span::{Atom as SpanAtom, Span};

// NOTE: Should keep all `enum` size == 16

/// The root node.
#[derive(Debug)]
pub struct RegExpLiteral<'a> {
    pub span: Span,
    pub pattern: Pattern<'a>,
    pub flags: Flags,
}

/// The pattern.
#[derive(Debug)]
pub struct Pattern<'a> {
    pub span: Span,
    pub alternatives: Vec<'a, Alternative<'a>>,
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

/// The alternative.
/// E.g. `a|b`
#[derive(Debug)]
pub struct Alternative<'a> {
    pub span: Span,
    pub terms: Vec<'a, Term<'a>>,
}

/// The type which includes all atom nodes.
#[derive(Debug)]
pub enum Term<'a> {
    Assertion(Box<'a, Assertion<'a>>),
    Atom(Box<'a, Atom<'a>>),
    AtomWithQuantifier(Box<'a, AtomWithQuantifier<'a>>),
}

/// The assertion.
#[derive(Debug)]
pub enum Assertion<'a> {
    BoundaryAssertion(Box<'a, BoundaryAssertion<'a>>),
    LookaroundAssertion(Box<'a, LookaroundAssertion<'a>>),
}

/// The boundary assertion.
#[derive(Debug)]
pub enum BoundaryAssertion<'a> {
    EdgeAssertion(Box<'a, EdgeAssertion>),
    WordBoundaryAssertion(Box<'a, WordBoundaryAssertion>),
}

/// The edge boundary assertion.
/// E.g. `^`, `$`
#[derive(Debug)]
pub struct EdgeAssertion {
    pub span: Span,
    pub kind: EdgeAssertionKind,
}

#[derive(Debug)]
pub enum EdgeAssertionKind {
    /// `^`
    Start,
    /// `$`
    End,
}

/// The word boundary assertion.
/// E.g. `\b`, `\B`
#[derive(Debug)]
pub struct WordBoundaryAssertion {
    pub span: Span,
    pub negate: bool,
}

/// The lookaround assertion.
#[derive(Debug)]
pub enum LookaroundAssertion<'a> {
    LookaheadAssertion(Box<'a, LookaheadAssertion<'a>>),
    LookbehindAssertion(Box<'a, LookbehindAssertion<'a>>),
}

/// The lookahead assertion.
/// E.g. `(?=ab)`, `(?!ab)`
#[derive(Debug)]
pub struct LookaheadAssertion<'a> {
    pub span: Span,
    pub negate: bool,
    pub alternatives: Vec<'a, Alternative<'a>>,
}

/// The lookbehind assertion.
/// E.g. `(?<=ab)`, `(?<!ab)`
#[derive(Debug)]
pub struct LookbehindAssertion<'a> {
    pub span: Span,
    pub negate: bool,
    pub alternatives: Vec<'a, Alternative<'a>>,
}

/// The type which includes all atom nodes that Quantifier node can have as children.
#[derive(Debug)]
pub enum Atom<'a> {
    Character(Box<'a, Character>),
    CharacterSet(Box<'a, CharacterSet<'a>>),
    Backreference(Box<'a, Backreference<'a>>),
    CharacterClass(Box<'a, CharacterClass<'a>>),
    CapturingGroup(Box<'a, CapturingGroup<'a>>),
    NonCapturingGroup(Box<'a, NonCapturingGroup<'a>>),
}

/// The quantifier.
/// E.g. `a?`, `a*`, `a+`, `a{1,2}`, `a??`, `a*?`, `a+?`, `a{1,2}?`
#[derive(Debug)]
pub struct AtomWithQuantifier<'a> {
    pub span: Span,
    pub min: usize,
    pub max: Option<usize>, // `None` means `Infinity`
    pub greedy: bool,
    pub atom: Atom<'a>,
}

/// The backreference.
/// E.g. `\1`, `\k<name>`
#[derive(Debug)]
pub enum Backreference<'a> {
    NormalBackreference(Box<'a, NormalBackreference>),
    NamedBackreference(Box<'a, NamedBackreference<'a>>),
}

/// E.g. `\1`
#[derive(Debug)]
pub struct NormalBackreference {
    pub span: Span,
    pub r#ref: usize, // 1 for \1
}

/// E.g. `\k<name>`
#[derive(Debug)]
pub struct NamedBackreference<'a> {
    pub span: Span,
    pub r#ref: SpanAtom<'a>, // name for \k<name>
}

/// The capturing group.
/// E.g. `(ab)`, `(?<name>ab)`
#[derive(Debug)]
pub struct CapturingGroup<'a> {
    pub span: Span,
    pub name: Option<SpanAtom<'a>>,
    pub alternatives: Vec<'a, Alternative<'a>>,
}

/// The non-capturing group.
/// E.g. `(?:ab)`
#[derive(Debug)]
pub struct NonCapturingGroup<'a> {
    pub span: Span,
    pub alternatives: Vec<'a, Alternative<'a>>,
}

/// The character.
/// This includes escape sequences which mean a character.
/// E.g. `a`, `あ`, `✿`, `\x65`, `\u0065`, `\u{65}`, `\/`
#[derive(Debug)]
pub struct Character {
    pub span: Span,
    pub value: u32,
}

/// The character set.
#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum CharacterSet<'a> {
    AnyCharacterSet(Box<'a, AnyCharacterSet>),
    EscapeCharacterSet(Box<'a, EscapeCharacterSet>),
    UnicodePropertyCharacterSet(Box<'a, UnicodePropertyCharacterSet<'a>>),
}

/// Any = `.`
#[derive(Debug)]
pub struct AnyCharacterSet {
    pub span: Span,
}

/// The character class escape.
/// E.g. `\d`, `\s`, `\w`, `\D`, `\S`, `\W`
#[derive(Debug)]
pub struct EscapeCharacterSet {
    pub span: Span,
    pub kind: EscapeCharacterSetKind,
    pub negate: bool,
}

#[derive(Debug)]
pub enum EscapeCharacterSetKind {
    Digit,
    Space,
    Word,
}

/// The unicode property escape.
/// E.g. `\p{ASCII}`, `\P{ASCII}`, `\p{Script=Hiragana}`
#[derive(Debug)]
pub enum UnicodePropertyCharacterSet<'a> {
    CharacterUnicodePropertyCharacterSet(Box<'a, CharacterUnicodePropertyCharacterSet<'a>>),
    StringsUnicodePropertyCharacterSet(Box<'a, StringsUnicodePropertyCharacterSet<'a>>),
}

#[derive(Debug)]
pub struct CharacterUnicodePropertyCharacterSet<'a> {
    pub span: Span,
    pub key: SpanAtom<'a>,
    pub value: Option<SpanAtom<'a>>,
    pub negate: bool,
}

/// The unicode property escape with property of strings. (`v` flag only)
#[derive(Debug)]
pub struct StringsUnicodePropertyCharacterSet<'a> {
    pub span: Span,
    pub key: SpanAtom<'a>,
}

/// The character class.
/// E.g. `[ab]`, `[^ab]`
#[derive(Debug)]
pub enum CharacterClass<'a> {
    ClassRangesCharacterClass(Box<'a, ClassRangesCharacterClass<'a>>),
    UnicodeSetsCharacterClass(Box<'a, UnicodeSetsCharacterClass<'a>>),
}

/// The character class used in non Unicode sets mode (`v` flag).
#[derive(Debug)]
pub struct ClassRangesCharacterClass<'a> {
    pub span: Span,
    pub negate: bool,
    pub elements: Vec<'a, ClassRangesCharacterClassElement<'a>>,
}

#[derive(Debug)]
pub enum ClassRangesCharacterClassElement<'a> {
    Character(Box<'a, Character>),
    CharacterClassRange(Box<'a, CharacterClassRange>),
    EscapeCharacterSet(Box<'a, EscapeCharacterSet>),
    CharacterUnicodePropertyCharacterSet(Box<'a, CharacterUnicodePropertyCharacterSet<'a>>),
}

/// The character class.
/// E.g. `[a-b]`
#[derive(Debug)]
pub struct CharacterClassRange {
    pub span: Span,
    pub min: Character,
    pub max: Character,
}

// TODO: Lines below are not yet finalized.

/// The character class used in Unicode sets mode (`v` flag).
#[derive(Debug)]
pub struct UnicodeSetsCharacterClass<'a> {
    pub span: Span,
    pub negate: bool,
    pub elements: Vec<'a, UnicodeSetsCharacterClassElement<'a>>,
}

#[derive(Debug)]
pub enum UnicodeSetsCharacterClassElement<'a> {
    Character(Box<'a, Character>),
    CharacterClassRange(Box<'a, CharacterClassRange>),
    EscapeCharacterSet(Box<'a, EscapeCharacterSet>),
    CharacterUnicodePropertyCharacterSet(Box<'a, CharacterUnicodePropertyCharacterSet<'a>>),
    // Above are same as `ClassRangesCharacterClassElement`
    StringsUnicodePropertyCharacterSet(Box<'a, StringsUnicodePropertyCharacterSet<'a>>),
    ClassStringDisjunction(Box<'a, ClassStringDisjunction<'a>>),
    ExpressionCharacterClass(Box<'a, ExpressionCharacterClass<'a>>),
    UnicodeSetsCharacterClass(Box<'a, UnicodeSetsCharacterClass<'a>>),
}

/// The character class string disjunction.
/// E.g. `\q{a|b}`
#[derive(Debug)]
pub struct ClassStringDisjunction<'a> {
    pub span: Span,
    pub alternatives: Vec<'a, StringAlternative<'a>>,
}

/// Only used for `\q{alt}`([`ClassStringDisjunction`]).
#[derive(Debug)]
pub struct StringAlternative<'a> {
    pub span: Span,
    pub elements: Vec<'a, Character>,
}

/// The expression character class.
/// E.g. `[a--b]`, `[a&&b]`,`[^a--b]`, `[^a&&b]`
#[derive(Debug)]
pub struct ExpressionCharacterClass<'a> {
    pub span: Span,
    pub negate: bool,
    pub expression: ExpressionCharacterClassExpr<'a>,
}

#[derive(Debug)]
pub enum ExpressionCharacterClassExpr<'a> {
    ClassIntersection(Box<'a, ClassIntersection<'a>>),
    ClassSubtraction(Box<'a, ClassSubtraction<'a>>),
}

/// The character class intersection.
/// E.g. `a&&b`
#[derive(Debug)]
pub struct ClassIntersection<'a> {
    pub span: Span,
    pub left: ClassIntersectionLeft<'a>,
    pub right: ClassSetOperand<'a>,
}

#[derive(Debug)]
pub enum ClassIntersectionLeft<'a> {
    ClassIntersection(Box<'a, ClassIntersection<'a>>),
    ClassSetOperand(Box<'a, ClassSetOperand<'a>>),
}

/// The character class subtraction.
/// E.g. `a--b`
#[derive(Debug)]
pub struct ClassSubtraction<'a> {
    pub span: Span,
    pub left: ClassSubtractionLeft<'a>,
    pub right: ClassSetOperand<'a>,
}

#[derive(Debug)]
pub enum ClassSubtractionLeft<'a> {
    ClassSetOperand(Box<'a, ClassSetOperand<'a>>),
    ClassSubtraction(Box<'a, ClassSubtraction<'a>>),
}

#[derive(Debug)]
pub enum ClassSetOperand<'a> {
    Character(Box<'a, Character>),
    ClassStringDisjunction(Box<'a, ClassStringDisjunction<'a>>),
    EscapeCharacterSet(Box<'a, EscapeCharacterSet>),
    ExpressionCharacterClass(Box<'a, ExpressionCharacterClass<'a>>),
    UnicodePropertyCharacterSet(Box<'a, UnicodePropertyCharacterSet<'a>>),
    UnicodeSetsCharacterClass(Box<'a, UnicodeSetsCharacterClass<'a>>),
}
