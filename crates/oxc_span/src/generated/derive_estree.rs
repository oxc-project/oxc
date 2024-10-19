// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, unused_mut, clippy::match_same_arms)]

use serde::{ser::SerializeMap, Serialize, Serializer};

#[allow(clippy::wildcard_imports)]
use crate::source_type::*;

#[allow(clippy::wildcard_imports)]
use crate::span::types::*;

impl Serialize for Span {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("start", &self.start)?;
        map.serialize_entry("end", &self.end)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type Span = ({\n\tstart: number;\n\tend: number;\n});";

impl Serialize for SourceType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("language", &self.language)?;
        map.serialize_entry("moduleKind", &self.module_kind)?;
        map.serialize_entry("variant", &self.variant)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type SourceType = ({\n\tlanguage: Language;\n\tmoduleKind: ModuleKind;\n\tvariant: LanguageVariant;\n});";

impl Serialize for Language {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            Language::JavaScript => {
                serializer.serialize_unit_variant("Language", 0u32, "javascript")
            }
            Language::TypeScript => {
                serializer.serialize_unit_variant("Language", 1u32, "typescript")
            }
            Language::TypeScriptDefinition => {
                serializer.serialize_unit_variant("Language", 2u32, "typescriptDefinition")
            }
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type Language = 'javascript' | 'typescript' | 'typescriptDefinition';";

impl Serialize for ModuleKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            ModuleKind::Script => serializer.serialize_unit_variant("ModuleKind", 0u32, "script"),
            ModuleKind::Module => serializer.serialize_unit_variant("ModuleKind", 1u32, "module"),
            ModuleKind::Unambiguous => {
                serializer.serialize_unit_variant("ModuleKind", 2u32, "unambiguous")
            }
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type ModuleKind = 'script' | 'module' | 'unambiguous';";

impl Serialize for LanguageVariant {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            LanguageVariant::Standard => {
                serializer.serialize_unit_variant("LanguageVariant", 0u32, "standard")
            }
            LanguageVariant::Jsx => {
                serializer.serialize_unit_variant("LanguageVariant", 1u32, "jsx")
            }
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type LanguageVariant = 'standard' | 'jsx';";
