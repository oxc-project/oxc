// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, unused_mut, clippy::match_same_arms)]

use serde::{ser::SerializeMap, Serialize, Serializer};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

impl<'a> Serialize for Pattern<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Pattern")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type Pattern = ({\n\ttype: 'Pattern';\n\tbody: Disjunction;\n}) & Span;";

impl<'a> Serialize for Disjunction<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Disjunction")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type Disjunction = ({\n\ttype: 'Disjunction';\n\tbody: Array<Alternative>;\n}) & Span;";

impl<'a> Serialize for Alternative<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Alternative")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type Alternative = ({\n\ttype: 'Alternative';\n\tbody: Array<Term>;\n}) & Span;";

impl<'a> Serialize for Term<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Term::BoundaryAssertion(x) => Serialize::serialize(x, serializer),
            Term::LookAroundAssertion(x) => Serialize::serialize(x, serializer),
            Term::Quantifier(x) => Serialize::serialize(x, serializer),
            Term::Character(x) => Serialize::serialize(x, serializer),
            Term::Dot(x) => Serialize::serialize(x, serializer),
            Term::CharacterClassEscape(x) => Serialize::serialize(x, serializer),
            Term::UnicodePropertyEscape(x) => Serialize::serialize(x, serializer),
            Term::CharacterClass(x) => Serialize::serialize(x, serializer),
            Term::CapturingGroup(x) => Serialize::serialize(x, serializer),
            Term::IgnoreGroup(x) => Serialize::serialize(x, serializer),
            Term::IndexedReference(x) => Serialize::serialize(x, serializer),
            Term::NamedReference(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Term = BoundaryAssertion | LookAroundAssertion | Quantifier | Character | Dot | CharacterClassEscape | UnicodePropertyEscape | CharacterClass | CapturingGroup | IgnoreGroup | IndexedReference | NamedReference;";

impl Serialize for BoundaryAssertion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BoundaryAssertion")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("kind", &self.kind)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type BoundaryAssertion = ({\n\ttype: 'BoundaryAssertion';\n\tspan: Span;\n\tkind: BoundaryAssertionKind;\n});";

impl Serialize for BoundaryAssertionKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type BoundaryAssertionKind = 'start' | 'end' | 'boundary' | 'negativeBoundary';";

impl<'a> Serialize for LookAroundAssertion<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "LookAroundAssertion")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type LookAroundAssertion = ({\n\ttype: 'LookAroundAssertion';\n\tkind: LookAroundAssertionKind;\n\tbody: Disjunction;\n}) & Span;";

impl Serialize for LookAroundAssertionKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type LookAroundAssertionKind = 'lookahead' | 'negativeLookahead' | 'lookbehind' | 'negativeLookbehind';";

impl<'a> Serialize for Quantifier<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Quantifier = ({\n\ttype: 'Quantifier';\n\tmin: number;\n\tmax: (number) | null;\n\tgreedy: boolean;\n\tbody: Term;\n}) & Span;";

impl Serialize for Character {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Character")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Character = ({\n\ttype: 'Character';\n\tkind: CharacterKind;\n\tvalue: number;\n}) & Span;";

impl Serialize for CharacterKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type CharacterKind = 'controlLetter' | 'hexadecimalEscape' | 'identifier' | 'null' | 'octal1' | 'octal2' | 'octal3' | 'singleEscape' | 'symbol' | 'unicodeEscape';";

impl Serialize for CharacterClassEscape {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CharacterClassEscape")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("kind", &self.kind)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type CharacterClassEscape = ({\n\ttype: 'CharacterClassEscape';\n\tkind: CharacterClassEscapeKind;\n}) & Span;";

impl Serialize for CharacterClassEscapeKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type CharacterClassEscapeKind = 'd' | 'negativeD' | 's' | 'negativeS' | 'w' | 'negativeW';";

impl<'a> Serialize for UnicodePropertyEscape<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type UnicodePropertyEscape = ({\n\ttype: 'UnicodePropertyEscape';\n\tnegative: boolean;\n\tstrings: boolean;\n\tname: string;\n\tvalue: (string) | null;\n}) & Span;";

impl Serialize for Dot {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Dot")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Dot = ({\n\ttype: 'Dot';\n}) & Span;";

impl<'a> Serialize for CharacterClass<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type CharacterClass = ({\n\ttype: 'CharacterClass';\n\tnegative: boolean;\n\tstrings: boolean;\n\tkind: CharacterClassContentsKind;\n\tbody: Array<CharacterClassContents>;\n}) & Span;";

impl Serialize for CharacterClassContentsKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type CharacterClassContentsKind = 'union' | 'intersection' | 'subtraction';";

impl<'a> Serialize for CharacterClassContents<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            CharacterClassContents::CharacterClassRange(x) => Serialize::serialize(x, serializer),
            CharacterClassContents::CharacterClassEscape(x) => Serialize::serialize(x, serializer),
            CharacterClassContents::UnicodePropertyEscape(x) => Serialize::serialize(x, serializer),
            CharacterClassContents::Character(x) => Serialize::serialize(x, serializer),
            CharacterClassContents::NestedCharacterClass(x) => Serialize::serialize(x, serializer),
            CharacterClassContents::ClassStringDisjunction(x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type CharacterClassContents = CharacterClassRange | CharacterClassEscape | UnicodePropertyEscape | Character | CharacterClass | ClassStringDisjunction;";

impl Serialize for CharacterClassRange {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CharacterClassRange")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("min", &self.min)?;
        map.serialize_entry("max", &self.max)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type CharacterClassRange = ({\n\ttype: 'CharacterClassRange';\n\tmin: Character;\n\tmax: Character;\n}) & Span;";

impl<'a> Serialize for ClassStringDisjunction<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ClassStringDisjunction")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ClassStringDisjunction = ({\n\ttype: 'ClassStringDisjunction';\n\tstrings: boolean;\n\tbody: Array<ClassString>;\n}) & Span;";

impl<'a> Serialize for ClassString<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ClassString")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ClassString = ({\n\ttype: 'ClassString';\n\tstrings: boolean;\n\tbody: Array<Character>;\n}) & Span;";

impl<'a> Serialize for CapturingGroup<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CapturingGroup")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type CapturingGroup = ({\n\ttype: 'CapturingGroup';\n\tname: (string) | null;\n\tbody: Disjunction;\n}) & Span;";

impl<'a> Serialize for IgnoreGroup<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "IgnoreGroup")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("modifiers", &self.modifiers)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type IgnoreGroup = ({\n\ttype: 'IgnoreGroup';\n\tmodifiers: (Modifiers) | null;\n\tbody: Disjunction;\n}) & Span;";

impl Serialize for Modifiers {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Modifiers")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("enabling", &self.enabling)?;
        map.serialize_entry("disabling", &self.disabling)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Modifiers = ({\n\ttype: 'Modifiers';\n\tenabling: (Modifier) | null;\n\tdisabling: (Modifier) | null;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Modifier = ({\n\ttype: 'Modifier';\n\tignoreCase: boolean;\n\tmultiline: boolean;\n\tsticky: boolean;\n});";

impl Serialize for IndexedReference {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "IndexedReference")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("index", &self.index)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type IndexedReference = ({\n\ttype: 'IndexedReference';\n\tindex: number;\n}) & Span;";

impl<'a> Serialize for NamedReference<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "NamedReference")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type NamedReference = ({\n\ttype: 'NamedReference';\n\tname: string;\n}) & Span;";
