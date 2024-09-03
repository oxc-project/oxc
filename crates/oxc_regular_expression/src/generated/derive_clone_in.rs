// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/clone_in.rs`

#![allow(clippy::default_trait_access)]

use oxc_allocator::{Allocator, CloneIn};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for RegularExpression<'old_alloc> {
    type Cloned = RegularExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        RegularExpression {
            span: self.span.clone_in(allocator),
            pattern: self.pattern.clone_in(allocator),
            flags: self.flags.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for Flags {
    type Cloned = Flags;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        Flags {
            span: self.span.clone_in(allocator),
            global: self.global.clone_in(allocator),
            ignore_case: self.ignore_case.clone_in(allocator),
            multiline: self.multiline.clone_in(allocator),
            unicode: self.unicode.clone_in(allocator),
            sticky: self.sticky.clone_in(allocator),
            dot_all: self.dot_all.clone_in(allocator),
            has_indices: self.has_indices.clone_in(allocator),
            unicode_sets: self.unicode_sets.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Pattern<'old_alloc> {
    type Cloned = Pattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Pattern { span: self.span.clone_in(allocator), body: self.body.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Disjunction<'old_alloc> {
    type Cloned = Disjunction<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Disjunction { span: self.span.clone_in(allocator), body: self.body.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Alternative<'old_alloc> {
    type Cloned = Alternative<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Alternative { span: self.span.clone_in(allocator), body: self.body.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Term<'old_alloc> {
    type Cloned = Term<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BoundaryAssertion(it) => Term::BoundaryAssertion(it.clone_in(allocator)),
            Self::LookAroundAssertion(it) => Term::LookAroundAssertion(it.clone_in(allocator)),
            Self::Quantifier(it) => Term::Quantifier(it.clone_in(allocator)),
            Self::Character(it) => Term::Character(it.clone_in(allocator)),
            Self::Dot(it) => Term::Dot(it.clone_in(allocator)),
            Self::CharacterClassEscape(it) => Term::CharacterClassEscape(it.clone_in(allocator)),
            Self::UnicodePropertyEscape(it) => Term::UnicodePropertyEscape(it.clone_in(allocator)),
            Self::CharacterClass(it) => Term::CharacterClass(it.clone_in(allocator)),
            Self::CapturingGroup(it) => Term::CapturingGroup(it.clone_in(allocator)),
            Self::IgnoreGroup(it) => Term::IgnoreGroup(it.clone_in(allocator)),
            Self::IndexedReference(it) => Term::IndexedReference(it.clone_in(allocator)),
            Self::NamedReference(it) => Term::NamedReference(it.clone_in(allocator)),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for BoundaryAssertion {
    type Cloned = BoundaryAssertion;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        BoundaryAssertion {
            span: self.span.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for BoundaryAssertionKind {
    type Cloned = BoundaryAssertionKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Start => BoundaryAssertionKind::Start,
            Self::End => BoundaryAssertionKind::End,
            Self::Boundary => BoundaryAssertionKind::Boundary,
            Self::NegativeBoundary => BoundaryAssertionKind::NegativeBoundary,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for LookAroundAssertion<'old_alloc> {
    type Cloned = LookAroundAssertion<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        LookAroundAssertion {
            span: self.span.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for LookAroundAssertionKind {
    type Cloned = LookAroundAssertionKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Lookahead => LookAroundAssertionKind::Lookahead,
            Self::NegativeLookahead => LookAroundAssertionKind::NegativeLookahead,
            Self::Lookbehind => LookAroundAssertionKind::Lookbehind,
            Self::NegativeLookbehind => LookAroundAssertionKind::NegativeLookbehind,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Quantifier<'old_alloc> {
    type Cloned = Quantifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Quantifier {
            span: self.span.clone_in(allocator),
            min: self.min.clone_in(allocator),
            max: self.max.clone_in(allocator),
            greedy: self.greedy.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for Character {
    type Cloned = Character;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        Character {
            span: self.span.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for CharacterKind {
    type Cloned = CharacterKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::ControlLetter => CharacterKind::ControlLetter,
            Self::HexadecimalEscape => CharacterKind::HexadecimalEscape,
            Self::Identifier => CharacterKind::Identifier,
            Self::Null => CharacterKind::Null,
            Self::Octal => CharacterKind::Octal,
            Self::SingleEscape => CharacterKind::SingleEscape,
            Self::Symbol => CharacterKind::Symbol,
            Self::UnicodeEscape => CharacterKind::UnicodeEscape,
        }
    }
}

impl<'alloc> CloneIn<'alloc> for CharacterClassEscape {
    type Cloned = CharacterClassEscape;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        CharacterClassEscape {
            span: self.span.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for CharacterClassEscapeKind {
    type Cloned = CharacterClassEscapeKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::D => CharacterClassEscapeKind::D,
            Self::NegativeD => CharacterClassEscapeKind::NegativeD,
            Self::S => CharacterClassEscapeKind::S,
            Self::NegativeS => CharacterClassEscapeKind::NegativeS,
            Self::W => CharacterClassEscapeKind::W,
            Self::NegativeW => CharacterClassEscapeKind::NegativeW,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for UnicodePropertyEscape<'old_alloc> {
    type Cloned = UnicodePropertyEscape<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        UnicodePropertyEscape {
            span: self.span.clone_in(allocator),
            negative: self.negative.clone_in(allocator),
            strings: self.strings.clone_in(allocator),
            name: self.name.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for Dot {
    type Cloned = Dot;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        Dot { span: self.span.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CharacterClass<'old_alloc> {
    type Cloned = CharacterClass<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        CharacterClass {
            span: self.span.clone_in(allocator),
            negative: self.negative.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for CharacterClassContentsKind {
    type Cloned = CharacterClassContentsKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Union => CharacterClassContentsKind::Union,
            Self::Intersection => CharacterClassContentsKind::Intersection,
            Self::Subtraction => CharacterClassContentsKind::Subtraction,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CharacterClassContents<'old_alloc> {
    type Cloned = CharacterClassContents<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::CharacterClassRange(it) => {
                CharacterClassContents::CharacterClassRange(it.clone_in(allocator))
            }
            Self::CharacterClassEscape(it) => {
                CharacterClassContents::CharacterClassEscape(it.clone_in(allocator))
            }
            Self::UnicodePropertyEscape(it) => {
                CharacterClassContents::UnicodePropertyEscape(it.clone_in(allocator))
            }
            Self::Character(it) => CharacterClassContents::Character(it.clone_in(allocator)),
            Self::NestedCharacterClass(it) => {
                CharacterClassContents::NestedCharacterClass(it.clone_in(allocator))
            }
            Self::ClassStringDisjunction(it) => {
                CharacterClassContents::ClassStringDisjunction(it.clone_in(allocator))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for CharacterClassRange {
    type Cloned = CharacterClassRange;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        CharacterClassRange {
            span: self.span.clone_in(allocator),
            min: self.min.clone_in(allocator),
            max: self.max.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ClassStringDisjunction<'old_alloc> {
    type Cloned = ClassStringDisjunction<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ClassStringDisjunction {
            span: self.span.clone_in(allocator),
            strings: self.strings.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ClassString<'old_alloc> {
    type Cloned = ClassString<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ClassString {
            span: self.span.clone_in(allocator),
            strings: self.strings.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CapturingGroup<'old_alloc> {
    type Cloned = CapturingGroup<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        CapturingGroup {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for IgnoreGroup<'old_alloc> {
    type Cloned = IgnoreGroup<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        IgnoreGroup {
            span: self.span.clone_in(allocator),
            enabling_modifiers: self.enabling_modifiers.clone_in(allocator),
            disabling_modifiers: self.disabling_modifiers.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for ModifierFlags {
    type Cloned = ModifierFlags;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        ModifierFlags {
            ignore_case: self.ignore_case.clone_in(allocator),
            sticky: self.sticky.clone_in(allocator),
            multiline: self.multiline.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for IndexedReference {
    type Cloned = IndexedReference;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        IndexedReference {
            span: self.span.clone_in(allocator),
            index: self.index.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for NamedReference<'old_alloc> {
    type Cloned = NamedReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        NamedReference { span: self.span.clone_in(allocator), name: self.name.clone_in(allocator) }
    }
}
