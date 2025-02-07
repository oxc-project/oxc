// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms)]

use serde::{__private::ser::FlatMapSerializer, ser::SerializeMap, Serialize, Serializer};

use oxc_estree::ser::AppendTo;

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
            BoundaryAssertionKind::Start => {
                serializer.serialize_unit_variant("BoundaryAssertionKind", 0, "start")
            }
            BoundaryAssertionKind::End => {
                serializer.serialize_unit_variant("BoundaryAssertionKind", 1, "end")
            }
            BoundaryAssertionKind::Boundary => {
                serializer.serialize_unit_variant("BoundaryAssertionKind", 2, "boundary")
            }
            BoundaryAssertionKind::NegativeBoundary => {
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
            LookAroundAssertionKind::Lookahead => {
                serializer.serialize_unit_variant("LookAroundAssertionKind", 0, "lookahead")
            }
            LookAroundAssertionKind::NegativeLookahead => {
                serializer.serialize_unit_variant("LookAroundAssertionKind", 1, "negativeLookahead")
            }
            LookAroundAssertionKind::Lookbehind => {
                serializer.serialize_unit_variant("LookAroundAssertionKind", 2, "lookbehind")
            }
            LookAroundAssertionKind::NegativeLookbehind => serializer.serialize_unit_variant(
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
            CharacterKind::ControlLetter => {
                serializer.serialize_unit_variant("CharacterKind", 0, "controlLetter")
            }
            CharacterKind::HexadecimalEscape => {
                serializer.serialize_unit_variant("CharacterKind", 1, "hexadecimalEscape")
            }
            CharacterKind::Identifier => {
                serializer.serialize_unit_variant("CharacterKind", 2, "identifier")
            }
            CharacterKind::Null => serializer.serialize_unit_variant("CharacterKind", 3, "null"),
            CharacterKind::Octal1 => {
                serializer.serialize_unit_variant("CharacterKind", 4, "octal1")
            }
            CharacterKind::Octal2 => {
                serializer.serialize_unit_variant("CharacterKind", 5, "octal2")
            }
            CharacterKind::Octal3 => {
                serializer.serialize_unit_variant("CharacterKind", 6, "octal3")
            }
            CharacterKind::SingleEscape => {
                serializer.serialize_unit_variant("CharacterKind", 7, "singleEscape")
            }
            CharacterKind::Symbol => {
                serializer.serialize_unit_variant("CharacterKind", 8, "symbol")
            }
            CharacterKind::UnicodeEscape => {
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
            CharacterClassEscapeKind::D => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 0, "d")
            }
            CharacterClassEscapeKind::NegativeD => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 1, "negativeD")
            }
            CharacterClassEscapeKind::S => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 2, "s")
            }
            CharacterClassEscapeKind::NegativeS => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 3, "negativeS")
            }
            CharacterClassEscapeKind::W => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 4, "w")
            }
            CharacterClassEscapeKind::NegativeW => {
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
            CharacterClassContentsKind::Union => {
                serializer.serialize_unit_variant("CharacterClassContentsKind", 0, "union")
            }
            CharacterClassContentsKind::Intersection => {
                serializer.serialize_unit_variant("CharacterClassContentsKind", 1, "intersection")
            }
            CharacterClassContentsKind::Subtraction => {
                serializer.serialize_unit_variant("CharacterClassContentsKind", 2, "subtraction")
            }
        }
    }
}

impl Serialize for CharacterClassContents<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
