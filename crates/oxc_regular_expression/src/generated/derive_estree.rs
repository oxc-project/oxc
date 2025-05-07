// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`.

#![allow(unused_imports, clippy::match_same_arms, clippy::semicolon_if_nothing_returned)]

use oxc_estree::{
    ESTree, FlatStructSerializer, JsonSafeString, Serializer, StructSerializer,
    ser::{AppendTo, AppendToConcat},
};

use crate::ast::*;

impl ESTree for Pattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Pattern"));
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for Disjunction<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Disjunction"));
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for Alternative<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Alternative"));
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("body", &self.body);
        state.end();
    }
}

impl ESTree for Term<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::BoundaryAssertion(it) => it.serialize(serializer),
            Self::LookAroundAssertion(it) => it.serialize(serializer),
            Self::Quantifier(it) => it.serialize(serializer),
            Self::Character(it) => it.serialize(serializer),
            Self::Dot(it) => it.serialize(serializer),
            Self::CharacterClassEscape(it) => it.serialize(serializer),
            Self::UnicodePropertyEscape(it) => it.serialize(serializer),
            Self::CharacterClass(it) => it.serialize(serializer),
            Self::CapturingGroup(it) => it.serialize(serializer),
            Self::IgnoreGroup(it) => it.serialize(serializer),
            Self::IndexedReference(it) => it.serialize(serializer),
            Self::NamedReference(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for BoundaryAssertion {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("BoundaryAssertion"));
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("kind", &self.kind);
        state.end();
    }
}

impl ESTree for BoundaryAssertionKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Start => JsonSafeString("start").serialize(serializer),
            Self::End => JsonSafeString("end").serialize(serializer),
            Self::Boundary => JsonSafeString("boundary").serialize(serializer),
            Self::NegativeBoundary => JsonSafeString("negativeBoundary").serialize(serializer),
        }
    }
}

impl ESTree for LookAroundAssertion<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("LookAroundAssertion"));
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
            Self::Lookahead => JsonSafeString("lookahead").serialize(serializer),
            Self::NegativeLookahead => JsonSafeString("negativeLookahead").serialize(serializer),
            Self::Lookbehind => JsonSafeString("lookbehind").serialize(serializer),
            Self::NegativeLookbehind => JsonSafeString("negativeLookbehind").serialize(serializer),
        }
    }
}

impl ESTree for Quantifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Quantifier"));
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
        state.serialize_field("type", &JsonSafeString("Character"));
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
            Self::ControlLetter => JsonSafeString("controlLetter").serialize(serializer),
            Self::HexadecimalEscape => JsonSafeString("hexadecimalEscape").serialize(serializer),
            Self::Identifier => JsonSafeString("identifier").serialize(serializer),
            Self::Null => JsonSafeString("null").serialize(serializer),
            Self::Octal1 => JsonSafeString("octal1").serialize(serializer),
            Self::Octal2 => JsonSafeString("octal2").serialize(serializer),
            Self::Octal3 => JsonSafeString("octal3").serialize(serializer),
            Self::SingleEscape => JsonSafeString("singleEscape").serialize(serializer),
            Self::Symbol => JsonSafeString("symbol").serialize(serializer),
            Self::UnicodeEscape => JsonSafeString("unicodeEscape").serialize(serializer),
        }
    }
}

impl ESTree for CharacterClassEscape {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("CharacterClassEscape"));
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("kind", &self.kind);
        state.end();
    }
}

impl ESTree for CharacterClassEscapeKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::D => JsonSafeString("d").serialize(serializer),
            Self::NegativeD => JsonSafeString("negativeD").serialize(serializer),
            Self::S => JsonSafeString("s").serialize(serializer),
            Self::NegativeS => JsonSafeString("negativeS").serialize(serializer),
            Self::W => JsonSafeString("w").serialize(serializer),
            Self::NegativeW => JsonSafeString("negativeW").serialize(serializer),
        }
    }
}

impl ESTree for UnicodePropertyEscape<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("UnicodePropertyEscape"));
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
        state.serialize_field("type", &JsonSafeString("Dot"));
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.end();
    }
}

impl ESTree for CharacterClass<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("CharacterClass"));
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
            Self::Union => JsonSafeString("union").serialize(serializer),
            Self::Intersection => JsonSafeString("intersection").serialize(serializer),
            Self::Subtraction => JsonSafeString("subtraction").serialize(serializer),
        }
    }
}

impl ESTree for CharacterClassContents<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::CharacterClassRange(it) => it.serialize(serializer),
            Self::CharacterClassEscape(it) => it.serialize(serializer),
            Self::UnicodePropertyEscape(it) => it.serialize(serializer),
            Self::Character(it) => it.serialize(serializer),
            Self::NestedCharacterClass(it) => it.serialize(serializer),
            Self::ClassStringDisjunction(it) => it.serialize(serializer),
        }
    }
}

impl ESTree for CharacterClassRange {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("CharacterClassRange"));
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
        state.serialize_field("type", &JsonSafeString("ClassStringDisjunction"));
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
        state.serialize_field("type", &JsonSafeString("ClassString"));
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
        state.serialize_field("type", &JsonSafeString("CapturingGroup"));
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
        state.serialize_field("type", &JsonSafeString("IgnoreGroup"));
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
        state.serialize_field("type", &JsonSafeString("Modifiers"));
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
        state.serialize_field("ignoreCase", &self.ignore_case);
        state.serialize_field("multiline", &self.multiline);
        state.serialize_field("sticky", &self.sticky);
        state.end();
    }
}

impl ESTree for IndexedReference {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("IndexedReference"));
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("index", &self.index);
        state.end();
    }
}

impl ESTree for NamedReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("NamedReference"));
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("name", &self.name);
        state.end();
    }
}
