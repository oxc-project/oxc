// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms, clippy::semicolon_if_nothing_returned)]

use oxc_estree::{
    ser::{AppendTo, AppendToConcat},
    ESTree, FlatStructSerializer, Serializer, StructSerializer,
};

use crate::ast::*;

impl ESTree for Pattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Pattern");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for Disjunction<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Disjunction");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for Alternative<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Alternative");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for Term<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Term::BoundaryAssertion(it) => it.serialize(serializer),
            Term::LookAroundAssertion(it) => it.serialize(serializer),
            Term::Quantifier(it) => it.serialize(serializer),
            Term::Character(it) => it.serialize(serializer),
            Term::Dot(it) => it.serialize(serializer),
            Term::CharacterClassEscape(it) => it.serialize(serializer),
            Term::UnicodePropertyEscape(it) => it.serialize(serializer),
            Term::CharacterClass(it) => it.serialize(serializer),
            Term::CapturingGroup(it) => it.serialize(serializer),
            Term::IgnoreGroup(it) => it.serialize(serializer),
            Term::IndexedReference(it) => it.serialize(serializer),
            Term::NamedReference(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for BoundaryAssertion {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "BoundaryAssertion");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("kind", &self.kind);
        state.end();
    }
}

impl ESTree for BoundaryAssertionKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            BoundaryAssertionKind::Start => "start".serialize(serializer),
            BoundaryAssertionKind::End => "end".serialize(serializer),
            BoundaryAssertionKind::Boundary => "boundary".serialize(serializer),
            BoundaryAssertionKind::NegativeBoundary => "negativeBoundary".serialize(serializer),
        }
    }
}

impl ESTree for LookAroundAssertion<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "LookAroundAssertion");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("kind", &self.kind);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for LookAroundAssertionKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            LookAroundAssertionKind::Lookahead => "lookahead".serialize(serializer),
            LookAroundAssertionKind::NegativeLookahead => "negativeLookahead".serialize(serializer),
            LookAroundAssertionKind::Lookbehind => "lookbehind".serialize(serializer),
            LookAroundAssertionKind::NegativeLookbehind => {
                "negativeLookbehind".serialize(serializer)
            }
        }
    }
}

impl ESTree for Quantifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Quantifier");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("min", &self.min);
        state.serialize_field("max", &self.max);
        state.serialize_field("greedy", &self.greedy);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for Character {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Character");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("kind", &self.kind);
        state.serialize_field("value", &self.value);
        state.end();
    }
}

impl ESTree for CharacterKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            CharacterKind::ControlLetter => "controlLetter".serialize(serializer),
            CharacterKind::HexadecimalEscape => "hexadecimalEscape".serialize(serializer),
            CharacterKind::Identifier => "identifier".serialize(serializer),
            CharacterKind::Null => "null".serialize(serializer),
            CharacterKind::Octal1 => "octal1".serialize(serializer),
            CharacterKind::Octal2 => "octal2".serialize(serializer),
            CharacterKind::Octal3 => "octal3".serialize(serializer),
            CharacterKind::SingleEscape => "singleEscape".serialize(serializer),
            CharacterKind::Symbol => "symbol".serialize(serializer),
            CharacterKind::UnicodeEscape => "unicodeEscape".serialize(serializer),
        }
    }
}

impl ESTree for CharacterClassEscape {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "CharacterClassEscape");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("kind", &self.kind);
        state.end();
    }
}

impl ESTree for CharacterClassEscapeKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            CharacterClassEscapeKind::D => "d".serialize(serializer),
            CharacterClassEscapeKind::NegativeD => "negativeD".serialize(serializer),
            CharacterClassEscapeKind::S => "s".serialize(serializer),
            CharacterClassEscapeKind::NegativeS => "negativeS".serialize(serializer),
            CharacterClassEscapeKind::W => "w".serialize(serializer),
            CharacterClassEscapeKind::NegativeW => "negativeW".serialize(serializer),
        }
    }
}

impl ESTree for UnicodePropertyEscape<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "UnicodePropertyEscape");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("negative", &self.negative);
        state.serialize_field("strings", &self.strings);
        state.serialize_field("name", &self.name);
        state.serialize_field("value", &self.value);
        state.end();
    }
}

impl ESTree for Dot {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Dot");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for CharacterClass<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "CharacterClass");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("negative", &self.negative);
        state.serialize_field("strings", &self.strings);
        state.serialize_field("kind", &self.kind);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for CharacterClassContentsKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            CharacterClassContentsKind::Union => "union".serialize(serializer),
            CharacterClassContentsKind::Intersection => "intersection".serialize(serializer),
            CharacterClassContentsKind::Subtraction => "subtraction".serialize(serializer),
        }
    }
}

impl ESTree for CharacterClassContents<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            CharacterClassContents::CharacterClassRange(it) => it.serialize(serializer),
            CharacterClassContents::CharacterClassEscape(it) => it.serialize(serializer),
            CharacterClassContents::UnicodePropertyEscape(it) => it.serialize(serializer),
            CharacterClassContents::Character(it) => it.serialize(serializer),
            CharacterClassContents::NestedCharacterClass(it) => it.serialize(serializer),
            CharacterClassContents::ClassStringDisjunction(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for CharacterClassRange {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "CharacterClassRange");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("min", &self.min);
        state.serialize_field("max", &self.max);
        state.end();
    }
}

impl ESTree for ClassStringDisjunction<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ClassStringDisjunction");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("strings", &self.strings);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for ClassString<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ClassString");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("strings", &self.strings);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for CapturingGroup<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "CapturingGroup");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for IgnoreGroup<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "IgnoreGroup");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("modifiers", &self.modifiers);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for Modifiers {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Modifiers");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("enabling", &self.enabling);
        state.serialize_field("disabling", &self.disabling);
        state.end();
    }
}

impl ESTree for Modifier {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Modifier");
        state.serialize_field("ignoreCase", &self.ignore_case);
        state.serialize_field("multiline", &self.multiline);
        state.serialize_field("sticky", &self.sticky);
        state.end();
    }
}

impl ESTree for IndexedReference {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "IndexedReference");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("index", &self.index);
        state.end();
    }
}

impl ESTree for NamedReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "NamedReference");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.end();
    }
}
