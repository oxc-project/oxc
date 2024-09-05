// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_eq.rs`

use oxc_span::cmp::ContentEq;

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

impl<'a> ContentEq for RegularExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.pattern.content_eq(&other.pattern) && self.flags.content_eq(&other.flags)
    }
}

impl ContentEq for Flags {
    fn content_eq(&self, other: &Self) -> bool {
        self.global.content_eq(&other.global)
            && self.ignore_case.content_eq(&other.ignore_case)
            && self.multiline.content_eq(&other.multiline)
            && self.unicode.content_eq(&other.unicode)
            && self.sticky.content_eq(&other.sticky)
            && self.dot_all.content_eq(&other.dot_all)
            && self.has_indices.content_eq(&other.has_indices)
            && self.unicode_sets.content_eq(&other.unicode_sets)
    }
}

impl<'a> ContentEq for Pattern<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for Disjunction<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for Alternative<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for Term<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::BoundaryAssertion(it) => {
                matches!(other, Self::BoundaryAssertion(other) if it.content_eq(other))
            }
            Self::LookAroundAssertion(it) => {
                matches!(other, Self::LookAroundAssertion(other) if it.content_eq(other))
            }
            Self::Quantifier(it) => {
                matches!(other, Self::Quantifier(other) if it.content_eq(other))
            }
            Self::Character(it) => {
                matches!(other, Self::Character(other) if it.content_eq(other))
            }
            Self::Dot(it) => matches!(other, Self::Dot(other) if it.content_eq(other)),
            Self::CharacterClassEscape(it) => {
                matches!(
                    other, Self::CharacterClassEscape(other) if it.content_eq(other)
                )
            }
            Self::UnicodePropertyEscape(it) => {
                matches!(
                    other, Self::UnicodePropertyEscape(other) if it.content_eq(other)
                )
            }
            Self::CharacterClass(it) => {
                matches!(other, Self::CharacterClass(other) if it.content_eq(other))
            }
            Self::CapturingGroup(it) => {
                matches!(other, Self::CapturingGroup(other) if it.content_eq(other))
            }
            Self::IgnoreGroup(it) => {
                matches!(other, Self::IgnoreGroup(other) if it.content_eq(other))
            }
            Self::IndexedReference(it) => {
                matches!(other, Self::IndexedReference(other) if it.content_eq(other))
            }
            Self::NamedReference(it) => {
                matches!(other, Self::NamedReference(other) if it.content_eq(other))
            }
        }
    }
}

impl ContentEq for BoundaryAssertion {
    fn content_eq(&self, other: &Self) -> bool {
        self.kind.content_eq(&other.kind)
    }
}

impl ContentEq for BoundaryAssertionKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for LookAroundAssertion<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.kind.content_eq(&other.kind) && self.body.content_eq(&other.body)
    }
}

impl ContentEq for LookAroundAssertionKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for Quantifier<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.min.content_eq(&other.min)
            && self.max.content_eq(&other.max)
            && self.greedy.content_eq(&other.greedy)
            && self.body.content_eq(&other.body)
    }
}

impl ContentEq for Character {
    fn content_eq(&self, other: &Self) -> bool {
        self.kind.content_eq(&other.kind) && self.value.content_eq(&other.value)
    }
}

impl ContentEq for CharacterKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for CharacterClassEscape {
    fn content_eq(&self, other: &Self) -> bool {
        self.kind.content_eq(&other.kind)
    }
}

impl ContentEq for CharacterClassEscapeKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for UnicodePropertyEscape<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.negative.content_eq(&other.negative)
            && self.strings.content_eq(&other.strings)
            && self.name.content_eq(&other.name)
            && self.value.content_eq(&other.value)
    }
}

impl ContentEq for Dot {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for CharacterClass<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.negative.content_eq(&other.negative)
            && self.kind.content_eq(&other.kind)
            && self.body.content_eq(&other.body)
    }
}

impl ContentEq for CharacterClassContentsKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for CharacterClassContents<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::CharacterClassRange(it) => {
                matches!(other, Self::CharacterClassRange(other) if it.content_eq(other))
            }
            Self::CharacterClassEscape(it) => {
                matches!(
                    other, Self::CharacterClassEscape(other) if it.content_eq(other)
                )
            }
            Self::UnicodePropertyEscape(it) => {
                matches!(
                    other, Self::UnicodePropertyEscape(other) if it.content_eq(other)
                )
            }
            Self::Character(it) => {
                matches!(other, Self::Character(other) if it.content_eq(other))
            }
            Self::NestedCharacterClass(it) => {
                matches!(
                    other, Self::NestedCharacterClass(other) if it.content_eq(other)
                )
            }
            Self::ClassStringDisjunction(it) => {
                matches!(
                    other, Self::ClassStringDisjunction(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl ContentEq for CharacterClassRange {
    fn content_eq(&self, other: &Self) -> bool {
        self.min.content_eq(&other.min) && self.max.content_eq(&other.max)
    }
}

impl<'a> ContentEq for ClassStringDisjunction<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.strings.content_eq(&other.strings) && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for ClassString<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.strings.content_eq(&other.strings) && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for CapturingGroup<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name) && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for IgnoreGroup<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.enabling_modifiers.content_eq(&other.enabling_modifiers)
            && self.disabling_modifiers.content_eq(&other.disabling_modifiers)
            && self.body.content_eq(&other.body)
    }
}

impl ContentEq for ModifierFlags {
    fn content_eq(&self, other: &Self) -> bool {
        self.ignore_case.content_eq(&other.ignore_case)
            && self.sticky.content_eq(&other.sticky)
            && self.multiline.content_eq(&other.multiline)
    }
}

impl ContentEq for IndexedReference {
    fn content_eq(&self, other: &Self) -> bool {
        self.index.content_eq(&other.index)
    }
}

impl<'a> ContentEq for NamedReference<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name)
    }
}
