// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_hash.rs`

#![allow(clippy::match_same_arms)]

use std::{hash::Hasher, mem::discriminant};

use oxc_span::hash::ContentHash;

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

impl<'a> ContentHash for Pattern<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for Disjunction<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for Alternative<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for Term<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        match self {
            Self::BoundaryAssertion(it) => ContentHash::content_hash(it, state),
            Self::LookAroundAssertion(it) => ContentHash::content_hash(it, state),
            Self::Quantifier(it) => ContentHash::content_hash(it, state),
            Self::Character(it) => ContentHash::content_hash(it, state),
            Self::Dot(it) => ContentHash::content_hash(it, state),
            Self::CharacterClassEscape(it) => ContentHash::content_hash(it, state),
            Self::UnicodePropertyEscape(it) => ContentHash::content_hash(it, state),
            Self::CharacterClass(it) => ContentHash::content_hash(it, state),
            Self::CapturingGroup(it) => ContentHash::content_hash(it, state),
            Self::IgnoreGroup(it) => ContentHash::content_hash(it, state),
            Self::IndexedReference(it) => ContentHash::content_hash(it, state),
            Self::NamedReference(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for BoundaryAssertion {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
    }
}

impl ContentHash for BoundaryAssertionKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for LookAroundAssertion<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for LookAroundAssertionKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for Quantifier<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.min, state);
        ContentHash::content_hash(&self.max, state);
        ContentHash::content_hash(&self.greedy, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for Character {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.value, state);
    }
}

impl ContentHash for CharacterKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl ContentHash for CharacterClassEscape {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.kind, state);
    }
}

impl ContentHash for CharacterClassEscapeKind {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl<'a> ContentHash for UnicodePropertyEscape<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.negative, state);
        ContentHash::content_hash(&self.strings, state);
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.value, state);
    }
}

impl ContentHash for Dot {
    fn content_hash<H: Hasher>(&self, _: &mut H) {}
}

impl<'a> ContentHash for CharacterClass<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.negative, state);
        ContentHash::content_hash(&self.strings, state);
        ContentHash::content_hash(&self.kind, state);
        ContentHash::content_hash(&self.body, state);
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
            Self::CharacterClassRange(it) => ContentHash::content_hash(it, state),
            Self::CharacterClassEscape(it) => ContentHash::content_hash(it, state),
            Self::UnicodePropertyEscape(it) => ContentHash::content_hash(it, state),
            Self::Character(it) => ContentHash::content_hash(it, state),
            Self::NestedCharacterClass(it) => ContentHash::content_hash(it, state),
            Self::ClassStringDisjunction(it) => ContentHash::content_hash(it, state),
        }
    }
}

impl ContentHash for CharacterClassRange {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.min, state);
        ContentHash::content_hash(&self.max, state);
    }
}

impl<'a> ContentHash for ClassStringDisjunction<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.strings, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for ClassString<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.strings, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for CapturingGroup<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl<'a> ContentHash for IgnoreGroup<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.modifiers, state);
        ContentHash::content_hash(&self.body, state);
    }
}

impl ContentHash for Modifiers {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.enabling, state);
        ContentHash::content_hash(&self.disabling, state);
    }
}

impl ContentHash for Modifier {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.ignore_case, state);
        ContentHash::content_hash(&self.multiline, state);
        ContentHash::content_hash(&self.sticky, state);
    }
}

impl ContentHash for IndexedReference {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.index, state);
    }
}

impl<'a> ContentHash for NamedReference<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.name, state);
    }
}
