// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_eq.rs`

#![allow(clippy::match_like_matches_macro)]

use oxc_span::cmp::ContentEq;

use crate::ast::*;

impl ContentEq for Pattern<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for Disjunction<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for Alternative<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for Term<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::BoundaryAssertion(a), Self::BoundaryAssertion(b)) => a.content_eq(b),
            (Self::LookAroundAssertion(a), Self::LookAroundAssertion(b)) => a.content_eq(b),
            (Self::Quantifier(a), Self::Quantifier(b)) => a.content_eq(b),
            (Self::Character(a), Self::Character(b)) => a.content_eq(b),
            (Self::Dot(a), Self::Dot(b)) => a.content_eq(b),
            (Self::CharacterClassEscape(a), Self::CharacterClassEscape(b)) => a.content_eq(b),
            (Self::UnicodePropertyEscape(a), Self::UnicodePropertyEscape(b)) => a.content_eq(b),
            (Self::CharacterClass(a), Self::CharacterClass(b)) => a.content_eq(b),
            (Self::CapturingGroup(a), Self::CapturingGroup(b)) => a.content_eq(b),
            (Self::IgnoreGroup(a), Self::IgnoreGroup(b)) => a.content_eq(b),
            (Self::IndexedReference(a), Self::IndexedReference(b)) => a.content_eq(b),
            (Self::NamedReference(a), Self::NamedReference(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for BoundaryAssertion {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.kind, &other.kind)
    }
}

impl ContentEq for BoundaryAssertionKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for LookAroundAssertion<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for LookAroundAssertionKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for Quantifier<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.min, &other.min)
            && ContentEq::content_eq(&self.max, &other.max)
            && ContentEq::content_eq(&self.greedy, &other.greedy)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for Character {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.value, &other.value)
    }
}

impl ContentEq for CharacterKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for CharacterClassEscape {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.kind, &other.kind)
    }
}

impl ContentEq for CharacterClassEscapeKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for UnicodePropertyEscape<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.negative, &other.negative)
            && ContentEq::content_eq(&self.strings, &other.strings)
            && ContentEq::content_eq(&self.name, &other.name)
            && ContentEq::content_eq(&self.value, &other.value)
    }
}

impl ContentEq for Dot {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for CharacterClass<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.negative, &other.negative)
            && ContentEq::content_eq(&self.strings, &other.strings)
            && ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for CharacterClassContentsKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for CharacterClassContents<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::CharacterClassRange(a), Self::CharacterClassRange(b)) => a.content_eq(b),
            (Self::CharacterClassEscape(a), Self::CharacterClassEscape(b)) => a.content_eq(b),
            (Self::UnicodePropertyEscape(a), Self::UnicodePropertyEscape(b)) => a.content_eq(b),
            (Self::Character(a), Self::Character(b)) => a.content_eq(b),
            (Self::NestedCharacterClass(a), Self::NestedCharacterClass(b)) => a.content_eq(b),
            (Self::ClassStringDisjunction(a), Self::ClassStringDisjunction(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for CharacterClassRange {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.min, &other.min) && ContentEq::content_eq(&self.max, &other.max)
    }
}

impl ContentEq for ClassStringDisjunction<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.strings, &other.strings)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for ClassString<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.strings, &other.strings)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for CapturingGroup<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for IgnoreGroup<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.modifiers, &other.modifiers)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for Modifiers {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.enabling, &other.enabling)
            && ContentEq::content_eq(&self.disabling, &other.disabling)
    }
}

impl ContentEq for Modifier {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.ignore_case, &other.ignore_case)
            && ContentEq::content_eq(&self.multiline, &other.multiline)
            && ContentEq::content_eq(&self.sticky, &other.sticky)
    }
}

impl ContentEq for IndexedReference {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.index, &other.index)
    }
}

impl ContentEq for NamedReference<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
    }
}
