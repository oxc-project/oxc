// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#[allow(unused_imports)]
use serde::{ser::SerializeMap, Serialize, Serializer};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

impl<'a> Serialize for Pattern<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Pattern")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl<'a> Serialize for Disjunction<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Disjunction")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl<'a> Serialize for Alternative<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Alternative")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl<'a> Serialize for Term<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Term::BoundaryAssertion(ref x) => Serialize::serialize(x, serializer),
            Term::LookAroundAssertion(ref x) => Serialize::serialize(x, serializer),
            Term::Quantifier(ref x) => Serialize::serialize(x, serializer),
            Term::Character(ref x) => Serialize::serialize(x, serializer),
            Term::Dot(ref x) => Serialize::serialize(x, serializer),
            Term::CharacterClassEscape(ref x) => Serialize::serialize(x, serializer),
            Term::UnicodePropertyEscape(ref x) => Serialize::serialize(x, serializer),
            Term::CharacterClass(ref x) => Serialize::serialize(x, serializer),
            Term::CapturingGroup(ref x) => Serialize::serialize(x, serializer),
            Term::IgnoreGroup(ref x) => Serialize::serialize(x, serializer),
            Term::IndexedReference(ref x) => Serialize::serialize(x, serializer),
            Term::NamedReference(ref x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for BoundaryAssertion {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BoundaryAssertion")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("kind", &self.kind)?;
        map.end()
    }
}

impl Serialize for BoundaryAssertionKind {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            BoundaryAssertionKind::Start => {
                serializer.serialize_unit_variant("BoundaryAssertionKind", 0u32, "start")
            }
            BoundaryAssertionKind::End => {
                serializer.serialize_unit_variant("BoundaryAssertionKind", 1u32, "end")
            }
            BoundaryAssertionKind::Boundary => {
                serializer.serialize_unit_variant("BoundaryAssertionKind", 2u32, "boundary")
            }
            BoundaryAssertionKind::NegativeBoundary => {
                serializer.serialize_unit_variant("BoundaryAssertionKind", 3u32, "negativeBoundary")
            }
        }
    }
}

impl<'a> Serialize for LookAroundAssertion<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "LookAroundAssertion")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for LookAroundAssertionKind {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            LookAroundAssertionKind::Lookahead => {
                serializer.serialize_unit_variant("LookAroundAssertionKind", 0u32, "lookahead")
            }
            LookAroundAssertionKind::NegativeLookahead => serializer.serialize_unit_variant(
                "LookAroundAssertionKind",
                1u32,
                "negativeLookahead",
            ),
            LookAroundAssertionKind::Lookbehind => {
                serializer.serialize_unit_variant("LookAroundAssertionKind", 2u32, "lookbehind")
            }
            LookAroundAssertionKind::NegativeLookbehind => serializer.serialize_unit_variant(
                "LookAroundAssertionKind",
                3u32,
                "negativeLookbehind",
            ),
        }
    }
}

impl<'a> Serialize for Quantifier<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Quantifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("min", &self.min)?;
        map.serialize_entry("max", &self.max)?;
        map.serialize_entry("greedy", &self.greedy)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Character {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Character")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for CharacterKind {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            CharacterKind::ControlLetter => {
                serializer.serialize_unit_variant("CharacterKind", 0u32, "controlLetter")
            }
            CharacterKind::HexadecimalEscape => {
                serializer.serialize_unit_variant("CharacterKind", 1u32, "hexadecimalEscape")
            }
            CharacterKind::Identifier => {
                serializer.serialize_unit_variant("CharacterKind", 2u32, "identifier")
            }
            CharacterKind::Null => serializer.serialize_unit_variant("CharacterKind", 3u32, "null"),
            CharacterKind::Octal1 => {
                serializer.serialize_unit_variant("CharacterKind", 4u32, "octal1")
            }
            CharacterKind::Octal2 => {
                serializer.serialize_unit_variant("CharacterKind", 5u32, "octal2")
            }
            CharacterKind::Octal3 => {
                serializer.serialize_unit_variant("CharacterKind", 6u32, "octal3")
            }
            CharacterKind::SingleEscape => {
                serializer.serialize_unit_variant("CharacterKind", 7u32, "singleEscape")
            }
            CharacterKind::Symbol => {
                serializer.serialize_unit_variant("CharacterKind", 8u32, "symbol")
            }
            CharacterKind::UnicodeEscape => {
                serializer.serialize_unit_variant("CharacterKind", 9u32, "unicodeEscape")
            }
        }
    }
}

impl Serialize for CharacterClassEscape {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CharacterClassEscape")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("kind", &self.kind)?;
        map.end()
    }
}

impl Serialize for CharacterClassEscapeKind {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            CharacterClassEscapeKind::D => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 0u32, "d")
            }
            CharacterClassEscapeKind::NegativeD => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 1u32, "negativeD")
            }
            CharacterClassEscapeKind::S => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 2u32, "s")
            }
            CharacterClassEscapeKind::NegativeS => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 3u32, "negativeS")
            }
            CharacterClassEscapeKind::W => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 4u32, "w")
            }
            CharacterClassEscapeKind::NegativeW => {
                serializer.serialize_unit_variant("CharacterClassEscapeKind", 5u32, "negativeW")
            }
        }
    }
}

impl<'a> Serialize for UnicodePropertyEscape<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "UnicodePropertyEscape")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("negative", &self.negative)?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for Dot {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Dot")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl<'a> Serialize for CharacterClass<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CharacterClass")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("negative", &self.negative)?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for CharacterClassContentsKind {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            CharacterClassContentsKind::Union => {
                serializer.serialize_unit_variant("CharacterClassContentsKind", 0u32, "union")
            }
            CharacterClassContentsKind::Intersection => serializer.serialize_unit_variant(
                "CharacterClassContentsKind",
                1u32,
                "intersection",
            ),
            CharacterClassContentsKind::Subtraction => {
                serializer.serialize_unit_variant("CharacterClassContentsKind", 2u32, "subtraction")
            }
        }
    }
}

impl<'a> Serialize for CharacterClassContents<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            CharacterClassContents::CharacterClassRange(ref x) => {
                Serialize::serialize(x, serializer)
            }
            CharacterClassContents::CharacterClassEscape(ref x) => {
                Serialize::serialize(x, serializer)
            }
            CharacterClassContents::UnicodePropertyEscape(ref x) => {
                Serialize::serialize(x, serializer)
            }
            CharacterClassContents::Character(ref x) => Serialize::serialize(x, serializer),
            CharacterClassContents::NestedCharacterClass(ref x) => {
                Serialize::serialize(x, serializer)
            }
            CharacterClassContents::ClassStringDisjunction(ref x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}

impl Serialize for CharacterClassRange {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CharacterClassRange")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("min", &self.min)?;
        map.serialize_entry("max", &self.max)?;
        map.end()
    }
}

impl<'a> Serialize for ClassStringDisjunction<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ClassStringDisjunction")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl<'a> Serialize for ClassString<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ClassString")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl<'a> Serialize for CapturingGroup<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CapturingGroup")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl<'a> Serialize for IgnoreGroup<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "IgnoreGroup")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("enablingModifiers", &self.enabling_modifiers)?;
        map.serialize_entry("disablingModifiers", &self.disabling_modifiers)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ModifierFlags {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ModifierFlags")?;
        map.serialize_entry("ignoreCase", &self.ignore_case)?;
        map.serialize_entry("sticky", &self.sticky)?;
        map.serialize_entry("multiline", &self.multiline)?;
        map.end()
    }
}

impl Serialize for IndexedReference {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "IndexedReference")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("index", &self.index)?;
        map.end()
    }
}

impl<'a> Serialize for NamedReference<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "NamedReference")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}
