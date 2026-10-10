// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/clone_in.rs`.

#![allow(unused_imports, unused_variables, clippy::default_trait_access, clippy::inline_always)]

use std::cell::Cell;

use oxc_allocator::{Allocator, CloneIn, CloneInSemanticIds};

use crate::ast::*;

impl<'new_alloc> CloneIn<'new_alloc> for Pattern<'_> {
    type Cloned = Pattern<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Pattern {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Disjunction<'_> {
    type Cloned = Disjunction<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Disjunction {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Alternative<'_> {
    type Cloned = Alternative<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Alternative {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Term<'_> {
    type Cloned = Term<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::BoundaryAssertion(it) => {
                Term::BoundaryAssertion(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::LookAroundAssertion(it) => {
                Term::LookAroundAssertion(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::Quantifier(it) => {
                Term::Quantifier(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::Character(it) => {
                Term::Character(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::Dot(it) => Term::Dot(CloneIn::clone_in_impl(it, with_semantic_ids, allocator)),
            Self::CharacterClassEscape(it) => {
                Term::CharacterClassEscape(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::UnicodePropertyEscape(it) => Term::UnicodePropertyEscape(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::CharacterClass(it) => {
                Term::CharacterClass(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::CapturingGroup(it) => {
                Term::CapturingGroup(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::IgnoreGroup(it) => {
                Term::IgnoreGroup(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::IndexedReference(it) => {
                Term::IndexedReference(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::NamedReference(it) => {
                Term::NamedReference(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BoundaryAssertion {
    type Cloned = BoundaryAssertion;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        BoundaryAssertion {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BoundaryAssertionKind {
    type Cloned = BoundaryAssertionKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for LookAroundAssertion<'_> {
    type Cloned = LookAroundAssertion<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        LookAroundAssertion {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for LookAroundAssertionKind {
    type Cloned = LookAroundAssertionKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Quantifier<'_> {
    type Cloned = Quantifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Quantifier {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            min: CloneIn::clone_in_impl(&self.min, with_semantic_ids, allocator),
            max: CloneIn::clone_in_impl(&self.max, with_semantic_ids, allocator),
            greedy: CloneIn::clone_in_impl(&self.greedy, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Character {
    type Cloned = Character;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Character {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CharacterKind {
    type Cloned = CharacterKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CharacterClassEscape {
    type Cloned = CharacterClassEscape;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        CharacterClassEscape {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CharacterClassEscapeKind {
    type Cloned = CharacterClassEscapeKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for UnicodePropertyEscape<'_> {
    type Cloned = UnicodePropertyEscape<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        UnicodePropertyEscape {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            negative: CloneIn::clone_in_impl(&self.negative, with_semantic_ids, allocator),
            strings: CloneIn::clone_in_impl(&self.strings, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Dot {
    type Cloned = Dot;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Dot { span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator) }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CharacterClass<'_> {
    type Cloned = CharacterClass<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        CharacterClass {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            negative: CloneIn::clone_in_impl(&self.negative, with_semantic_ids, allocator),
            strings: CloneIn::clone_in_impl(&self.strings, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CharacterClassContentsKind {
    type Cloned = CharacterClassContentsKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CharacterClassContents<'_> {
    type Cloned = CharacterClassContents<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::CharacterClassRange(it) => CharacterClassContents::CharacterClassRange(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::CharacterClassEscape(it) => CharacterClassContents::CharacterClassEscape(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::UnicodePropertyEscape(it) => CharacterClassContents::UnicodePropertyEscape(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::Character(it) => CharacterClassContents::Character(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::NestedCharacterClass(it) => CharacterClassContents::NestedCharacterClass(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ClassStringDisjunction(it) => CharacterClassContents::ClassStringDisjunction(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CharacterClassRange {
    type Cloned = CharacterClassRange;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        CharacterClassRange {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            min: CloneIn::clone_in_impl(&self.min, with_semantic_ids, allocator),
            max: CloneIn::clone_in_impl(&self.max, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ClassStringDisjunction<'_> {
    type Cloned = ClassStringDisjunction<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ClassStringDisjunction {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            strings: CloneIn::clone_in_impl(&self.strings, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ClassString<'_> {
    type Cloned = ClassString<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ClassString {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            strings: CloneIn::clone_in_impl(&self.strings, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CapturingGroup<'_> {
    type Cloned = CapturingGroup<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        CapturingGroup {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for IgnoreGroup<'_> {
    type Cloned = IgnoreGroup<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        IgnoreGroup {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            modifiers: CloneIn::clone_in_impl(&self.modifiers, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Modifiers {
    type Cloned = Modifiers;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Modifiers {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            enabling: CloneIn::clone_in_impl(&self.enabling, with_semantic_ids, allocator),
            disabling: CloneIn::clone_in_impl(&self.disabling, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for IndexedReference {
    type Cloned = IndexedReference;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        IndexedReference {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            index: CloneIn::clone_in_impl(&self.index, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for NamedReference<'_> {
    type Cloned = NamedReference<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        NamedReference {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
        }
    }
}
