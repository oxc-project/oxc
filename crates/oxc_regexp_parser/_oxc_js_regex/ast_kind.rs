use super::ast::*;

#[allow(unused)]
#[derive(Debug)]
pub enum AstKind<'a> {
    Alternative(&'a Alternative<'a>),
    CapturingGroup(&'a CapturingGroup<'a>),
    CharacterClass(&'a CharacterClass<'a>),
    CharacterClassRange(&'a CharacterClassRange),
    ClassIntersection(&'a ClassIntersection<'a>),
    ClassStringDisjunction(&'a ClassStringDisjunction<'a>),
    ClassSubtraction(&'a ClassSubtraction<'a>),
    ExpressionCharacterClass(&'a ExpressionCharacterClass<'a>),
    Group(&'a Group<'a>),
    LookaroundAssertion(&'a LookaroundAssertion<'a>),
    Pattern(&'a Pattern<'a>),
    Quantifier(&'a Quantifier<'a>),
    RegExpLiteral(&'a RegExpLiteral<'a>),
    StringAlternative(&'a StringAlternative<'a>),
    Backreference(&'a Backreference<'a>),
    BoundaryAssertion(&'a BoundaryAssertion<'a>),
    Character(&'a Character),
    CharacterSet(&'a CharacterSet<'a>),
    Flags(&'a Flags),
}
