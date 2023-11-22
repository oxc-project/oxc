//! [`@eslint-community/regexpp`](https://github.com/eslint-community/regexpp/blob/2e8f1af992fb12eae46a446253e8fa3f6cede92a/src/ast.ts)

use oxc_allocator::{Box, Vec};
use oxc_span::{Atom, Span};

/// The type which includes all nodes.
#[derive(Debug)]
pub enum Node<'a> {
    Branch(Box<'a, Branch<'a>>),
    Leaf(Box<'a, Leaf<'a>>),
}

/// The type which includes all branch nodes.
#[derive(Debug)]
pub enum Branch<'a> {
    Alternative(Box<'a, Alternative<'a>>),
    CapturingGroup(Box<'a, CapturingGroup<'a>>),
    CharacterClass(Box<'a, CharacterClass<'a>>),
    CharacterClassRange(Box<'a, CharacterClassRange>),
    ClassIntersection(Box<'a, ClassIntersection<'a>>),
    ClassStringDisjunction(Box<'a, ClassStringDisjunction<'a>>),
    ClassSubtraction(Box<'a, ClassSubtraction<'a>>),
    ExpressionCharacterClass(Box<'a, ExpressionCharacterClass<'a>>),
    Group(Box<'a, Group<'a>>),
    LookaroundAssertion(Box<'a, LookaroundAssertion<'a>>),
    Pattern(Box<'a, Pattern<'a>>),
    Quantifier(Box<'a, Quantifier<'a>>),
    RegExpLiteral(Box<'a, RegExpLiteral<'a>>),
    StringAlternative(Box<'a, StringAlternative<'a>>),
}

/// The type which includes all leaf nodes.
#[derive(Debug)]
pub enum Leaf<'a> {
    Backreference(Box<'a, Backreference<'a>>),
    BoundaryAssertion(Box<'a, BoundaryAssertion<'a>>),
    Character(Box<'a, Character>),
    CharacterSet(Box<'a, CharacterSet<'a>>),
    Flags(Box<'a, Flags>),
}

/// The type which includes all atom nodes.
#[derive(Debug)]
pub enum Element<'a> {
    Assertion(Box<'a, Assertion<'a>>),
    QuantifiableElement(Box<'a, QuantifiableElement<'a>>),
    Quantifier(Box<'a, Quantifier<'a>>),
}

/// The type which includes all atom nodes that Quantifier node can have as children.
#[derive(Debug)]
pub enum QuantifiableElement<'a> {
    Backreference(Box<'a, Backreference<'a>>),
    CapturingGroup(Box<'a, CapturingGroup<'a>>),
    Character(Box<'a, Character>),
    CharacterClass(Box<'a, CharacterClass<'a>>),
    CharacterSet(Box<'a, CharacterSet<'a>>),
    ExpressionCharacterClass(Box<'a, ExpressionCharacterClass<'a>>),
    Group(Box<'a, Group<'a>>),
    LookaheadAssertion(Box<'a, LookaheadAssertion<'a>>),
}

/// The type which includes all character class atom nodes.
#[derive(Debug)]
pub enum CharacterClassElement<'a> {
    ClassRangesCharacterClassElement(Box<'a, ClassRangesCharacterClassElement<'a>>),
    UnicodeSetsCharacterClassElement(Box<'a, UnicodeSetsCharacterClassElement<'a>>),
}
#[derive(Debug)]
pub enum ClassRangesCharacterClassElement<'a> {
    Character(Box<'a, Character>),
    CharacterClassRange(Box<'a, CharacterClassRange>),
    CharacterUnicodePropertyCharacterSet(Box<'a, CharacterUnicodePropertyCharacterSet>),
    EscapeCharacterSet(Box<'a, EscapeCharacterSet>),
}
#[derive(Debug)]
pub enum UnicodeSetsCharacterClassElement<'a> {
    Character(Box<'a, Character>),
    CharacterClassRange(Box<'a, CharacterClassRange>),
    ClassStringDisjunction(Box<'a, ClassStringDisjunction<'a>>),
    EscapeCharacterSet(Box<'a, EscapeCharacterSet>),
    ExpressionCharacterClass(Box<'a, ExpressionCharacterClass<'a>>),
    UnicodePropertyCharacterSet(Box<'a, UnicodePropertyCharacterSet<'a>>),
    UnicodeSetsCharacterClass(Box<'a, UnicodeSetsCharacterClass<'a>>),
}

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

/// The alternative.
/// E.g. `a|b`
#[derive(Debug)]
pub struct Alternative<'a> {
    pub span: Span,
    pub elements: Vec<'a, Element<'a>>,
}

/// The uncapturing group.
/// E.g. `(?:ab)`
#[derive(Debug)]
pub struct Group<'a> {
    pub span: Span,
    pub alternatives: Vec<'a, Alternative<'a>>,
}

/// The capturing group.
/// E.g. `(ab)`, `(?<name>ab)`
#[derive(Debug)]
pub struct CapturingGroup<'a> {
    pub span: Span,
    pub name: Option<Atom>,
    pub alternatives: Vec<'a, Alternative<'a>>,
    pub references: Vec<'a, Backreference<'a>>,
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

/// The quantifier.
/// E.g. `a?`, `a*`, `a+`, `a{1,2}`, `a??`, `a*?`, `a+?`, `a{1,2}?`
#[derive(Debug)]
pub struct Quantifier<'a> {
    pub span: Span,
    pub min: f64,
    pub max: f64, // can be f64::INFINITY
    pub greedy: bool,
    pub element: QuantifiableElement<'a>,
}

/// The character class.
/// E.g. `[ab]`, `[^ab]`
#[derive(Debug)]
pub enum CharacterClass<'a> {
    ClassRangesCharacterClass(Box<'a, ClassRangesCharacterClass<'a>>),
    UnicodeSetsCharacterClass(Box<'a, UnicodeSetsCharacterClass<'a>>),
}

/// The character class used in legacy (neither `u` nor `v` flag) and Unicode mode (`u` flag).
/// This character class is guaranteed to **not** contain strings.
/// In Unicode sets mode (`v` flag), {@link UnicodeSetsCharacterClass} is used.
#[derive(Debug)]
pub struct ClassRangesCharacterClass<'a> {
    pub span: Span,
    pub unicode_sets: bool,
    pub elements: Vec<'a, ClassRangesCharacterClassElement<'a>>,
}

/// The character class used in Unicode sets mode (`v` flag).
/// This character class may contain strings.
#[derive(Debug)]
pub struct UnicodeSetsCharacterClass<'a> {
    pub span: Span,
    pub elements: Vec<'a, UnicodeSetsCharacterClassElement<'a>>,
}

/// The character class.
/// E.g. `[a-b]`
#[derive(Debug)]
pub struct CharacterClassRange {
    pub span: Span,
    pub min: Character,
    pub max: Character,
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
    Start,
    End,
}

/// The word boundary assertion.
/// E.g. `\b`, `\B`
#[derive(Debug)]
pub struct WordBoundaryAssertion {
    pub span: Span,
    pub negate: bool,
}

/// The character set.
#[derive(Debug)]
pub enum CharacterSet<'a> {
    AnyCharacterSet,
    EscapeCharacterSet(Box<'a, EscapeCharacterSet>),
    UnicodePropertyCharacterSet(Box<'a, UnicodePropertyCharacterSet<'a>>),
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
    CharacterUnicodePropertyCharacterSet(Box<'a, CharacterUnicodePropertyCharacterSet>),
    StringsUnicodePropertyCharacterSet(Box<'a, StringsUnicodePropertyCharacterSet>),
}

#[derive(Debug)]
pub struct CharacterUnicodePropertyCharacterSet {
    pub span: Span,
    pub key: Atom,
    pub value: Option<Atom>,
    pub negate: bool,
}

/// StringsUnicodePropertyCharacterSet is Unicode property escape with property of strings.
#[derive(Debug)]
pub struct StringsUnicodePropertyCharacterSet {
    pub span: Span,
    pub key: Atom,
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

#[derive(Debug)]
pub enum ClassSetOperand<'a> {
    Character(Box<'a, Character>),
    ClassStringDisjunction(Box<'a, ClassStringDisjunction<'a>>),
    EscapeCharacterSet(Box<'a, EscapeCharacterSet>),
    ExpressionCharacterClass(Box<'a, ExpressionCharacterClass<'a>>),
    UnicodePropertyCharacterSet(Box<'a, UnicodePropertyCharacterSet<'a>>),
    UnicodeSetsCharacterClass(Box<'a, UnicodeSetsCharacterClass<'a>>),
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

/// The character class string disjunction.
/// E.g. `\q{a|b}`
#[derive(Debug)]
pub struct ClassStringDisjunction<'a> {
    pub span: Span,
    pub alternatives: Vec<'a, StringAlternative<'a>>,
}

/// StringAlternative is only used for `\q{alt}`({@link ClassStringDisjunction}).
#[derive(Debug)]
pub struct StringAlternative<'a> {
    pub span: Span,
    pub elements: Vec<'a, Character>,
}

/// This includes escape sequences which mean a character.
/// E.g. `a`, `あ`, `✿`, `\x65`, `\u0065`, `\u{65}`, `\/`
#[derive(Debug)]
pub struct Character {
    pub span: Span,
    pub value: u16, // UTF-16 code point
}

#[derive(Debug)]
pub enum BackreferenceRef {
    Number(i32),
    Atom(Atom),
}

/// The backreference.
/// E.g. `\1`, `\k<name>`
#[derive(Debug)]
pub struct Backreference<'a> {
    pub span: Span,
    pub reference: BackreferenceRef,
    pub resolved: CapturingGroup<'a>,
}

/// The flags.
#[derive(Debug)]
pub struct Flags {
    pub span: Span,
    pub dot_all: bool,
    pub global: bool,
    pub has_indices: bool,
    pub ignore_case: bool,
    pub multiline: bool,
    pub sticky: bool,
    pub unicode: bool,
    pub unicode_sets: bool,
}
