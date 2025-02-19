// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms)]

use serde::{__private::ser::FlatMapSerializer, ser::SerializeMap, Serialize, Serializer};

use oxc_estree::ser::{AppendTo, AppendToConcat};

use crate::ast::*;

impl Serialize for Pattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Pattern")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Disjunction<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Disjunction")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Alternative<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Alternative")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Term<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for BoundaryAssertion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BoundaryAssertion")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("kind", &self.kind)?;
        map.end()
    }
}

impl Serialize for BoundaryAssertionKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Start => serializer.serialize_unit_variant("BoundaryAssertionKind", 0, "start"),
            Self::End => serializer.serialize_unit_variant("BoundaryAssertionKind", 1, "end"),
            Self::Boundary => {
                serializer.serialize_unit_variant("BoundaryAssertionKind", 2, "boundary")
            }
            Self::NegativeBoundary => {
                serializer.serialize_unit_variant("BoundaryAssertionKind", 3, "negativeBoundary")
            }
        }
    }
}

impl Serialize for LookAroundAssertion<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "LookAroundAssertion")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for LookAroundAssertionKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Lookahead => {
                serializer.serialize_unit_variant("LookAroundAssertionKind", 0, "lookahead")
            }
            Self::NegativeLookahead => {
                serializer.serialize_unit_variant("LookAroundAssertionKind", 1, "negativeLookahead")
            }
            Self::Lookbehind => {
                serializer.serialize_unit_variant("LookAroundAssertionKind", 2, "lookbehind")
            }
            Self::NegativeLookbehind => serializer.serialize_unit_variant(
                "LookAroundAssertionKind",
                3,
                "negativeLookbehind",
            ),
        }
    }
}

impl Serialize for Quantifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Quantifier")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("min", &self.min)?;
        map.serialize_entry("max", &self.max)?;
        map.serialize_entry("greedy", &self.greedy)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Character {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Character")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for CharacterKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::ControlLetter => {
                serializer.serialize_unit_variant("CharacterKind", 0, "controlLetter")
            }
            Self::HexadecimalEscape => {
                serializer.serialize_unit_variant("CharacterKind", 1, "hexadecimalEscape")
            }
            Self::Identifier => serializer.serialize_unit_variant("CharacterKind", 2, "identifier"),
            Self::Null => serializer.serialize_unit_variant("CharacterKind", 3, "null"),
            Self::Octal1 => serializer.serialize_unit_variant("CharacterKind", 4, "octal1"),
            Self::Octal2 => serializer.serialize_unit_variant("CharacterKind", 5, "octal2"),
            Self::Octal3 => serializer.serialize_unit_variant("CharacterKind", 6, "octal3"),
            Self::SingleEscape => {
                serializer.serialize_unit_variant("CharacterKind", 7, "singleEscape")
            }
            Self::Symbol => serializer.serialize_unit_variant("CharacterKind", 8, "symbol"),
            Self::UnicodeEscape => {
                serializer.serialize_unit_variant("CharacterKind", 9, "unicodeEscape")
            }
        }
    }
}

impl Serialize for CharacterClassEscape {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CharacterClassEscape")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("kind", &self.kind)?;
        map.end()
    }
}

impl Serialize for CharacterClassEscapeKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::D => serializer.serialize_unit_variant("CharacterClassEscapeKind", 0, "d"),
            Self::NegativeD => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 1, "negativeD")
            }
            Self::S => serializer.serialize_unit_variant("CharacterClassEscapeKind", 2, "s"),
            Self::NegativeS => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 3, "negativeS")
            }
            Self::W => serializer.serialize_unit_variant("CharacterClassEscapeKind", 4, "w"),
            Self::NegativeW => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 5, "negativeW")
            }
        }
    }
}

impl Serialize for UnicodePropertyEscape<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "UnicodePropertyEscape")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("negative", &self.negative)?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for Dot {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Dot")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.end()
    }
}

impl Serialize for CharacterClass<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CharacterClass")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("negative", &self.negative)?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for CharacterClassContentsKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Union => {
                serializer.serialize_unit_variant("CharacterClassContentsKind", 0, "union")
            }
            Self::Intersection => {
                serializer.serialize_unit_variant("CharacterClassContentsKind", 1, "intersection")
            }
            Self::Subtraction => {
                serializer.serialize_unit_variant("CharacterClassContentsKind", 2, "subtraction")
            }
        }
    }
}

impl Serialize for CharacterClassContents<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for CharacterClassRange {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CharacterClassRange")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("min", &self.min)?;
        map.serialize_entry("max", &self.max)?;
        map.end()
    }
}

impl Serialize for ClassStringDisjunction<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ClassStringDisjunction")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ClassString<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ClassString")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for CapturingGroup<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CapturingGroup")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for IgnoreGroup<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "IgnoreGroup")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("modifiers", &self.modifiers)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Modifiers {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Modifiers")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("enabling", &self.enabling)?;
        map.serialize_entry("disabling", &self.disabling)?;
        map.end()
    }
}

impl Serialize for Modifier {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Modifier")?;
        map.serialize_entry("ignoreCase", &self.ignore_case)?;
        map.serialize_entry("multiline", &self.multiline)?;
        map.serialize_entry("sticky", &self.sticky)?;
        map.end()
    }
}

impl Serialize for IndexedReference {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "IndexedReference")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("index", &self.index)?;
        map.end()
    }
}

impl Serialize for NamedReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "NamedReference")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}
