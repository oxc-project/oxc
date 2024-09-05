// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_hash.rs`

#![allow(clippy::match_same_arms)]

use std::{hash::Hasher, mem::discriminant};

use oxc_span::hash::ContentHash;

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

impl<'a> ContentHash for RegularExpression<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.pattern.content_hash(state);
        self.flags.content_hash(state);
    }
}

impl ContentHash for Flags {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.global.content_hash(state);
        self.ignore_case.content_hash(state);
        self.multiline.content_hash(state);
        self.unicode.content_hash(state);
        self.sticky.content_hash(state);
        self.dot_all.content_hash(state);
        self.has_indices.content_hash(state);
        self.unicode_sets.content_hash(state);
    }
}

impl<'a> ContentHash for Pattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for Disjunction<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for Alternative<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for Term<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::BoundaryAssertion(it) => it.content_hash(state),
            Self::LookAroundAssertion(it) => it.content_hash(state),
            Self::Quantifier(it) => it.content_hash(state),
            Self::Character(it) => it.content_hash(state),
            Self::Dot(it) => it.content_hash(state),
            Self::CharacterClassEscape(it) => it.content_hash(state),
            Self::UnicodePropertyEscape(it) => it.content_hash(state),
            Self::CharacterClass(it) => it.content_hash(state),
            Self::CapturingGroup(it) => it.content_hash(state),
            Self::IgnoreGroup(it) => it.content_hash(state),
            Self::IndexedReference(it) => it.content_hash(state),
            Self::NamedReference(it) => it.content_hash(state),
        }
    }
}

impl ContentHash for BoundaryAssertion {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.kind.content_hash(state);
    }
}

impl ContentHash for BoundaryAssertionKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for LookAroundAssertion<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.kind.content_hash(state);
        self.body.content_hash(state);
    }
}

impl ContentHash for LookAroundAssertionKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for Quantifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.min.content_hash(state);
        self.max.content_hash(state);
        self.greedy.content_hash(state);
        self.body.content_hash(state);
    }
}

impl ContentHash for Character {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.kind.content_hash(state);
        self.value.content_hash(state);
    }
}

impl ContentHash for CharacterKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl ContentHash for CharacterClassEscape {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.kind.content_hash(state);
    }
}

impl ContentHash for CharacterClassEscapeKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for UnicodePropertyEscape<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.negative.content_hash(state);
        self.strings.content_hash(state);
        self.name.content_hash(state);
        self.value.content_hash(state);
    }
}

impl ContentHash for Dot {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for CharacterClass<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.negative.content_hash(state);
        self.kind.content_hash(state);
        self.body.content_hash(state);
    }
}

impl ContentHash for CharacterClassContentsKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for CharacterClassContents<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::CharacterClassRange(it) => it.content_hash(state),
            Self::CharacterClassEscape(it) => it.content_hash(state),
            Self::UnicodePropertyEscape(it) => it.content_hash(state),
            Self::Character(it) => it.content_hash(state),
            Self::NestedCharacterClass(it) => it.content_hash(state),
            Self::ClassStringDisjunction(it) => it.content_hash(state),
        }
    }
}

impl ContentHash for CharacterClassRange {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.min.content_hash(state);
        self.max.content_hash(state);
    }
}

impl<'a> ContentHash for ClassStringDisjunction<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.strings.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for ClassString<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.strings.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for CapturingGroup<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
        self.body.content_hash(state);
    }
}

impl<'a> ContentHash for IgnoreGroup<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.enabling_modifiers.content_hash(state);
        self.disabling_modifiers.content_hash(state);
        self.body.content_hash(state);
    }
}

impl ContentHash for ModifierFlags {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.ignore_case.content_hash(state);
        self.sticky.content_hash(state);
        self.multiline.content_hash(state);
    }
}

impl ContentHash for IndexedReference {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.index.content_hash(state);
    }
}

impl<'a> ContentHash for NamedReference<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        self.name.content_hash(state);
    }
}
