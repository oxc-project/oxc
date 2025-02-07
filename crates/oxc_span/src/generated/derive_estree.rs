// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms)]

use serde::{__private::ser::FlatMapSerializer, ser::SerializeMap, Serialize, Serializer};

use oxc_estree::ser::AppendTo;

use crate::source_type::*;
use crate::span::*;

impl Serialize for Span {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("start", &self.start)?;
        map.serialize_entry("end", &self.end)?;
        map.end()
    }
}

impl Serialize for SourceType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("sourceType", &self.module_kind)?;
        map.end()
    }
}

impl Serialize for ModuleKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ModuleKind::Script => serializer.serialize_unit_variant("ModuleKind", 0, "script"),
            ModuleKind::Module => serializer.serialize_unit_variant("ModuleKind", 1, "module"),
            ModuleKind::Unambiguous => {
                serializer.serialize_unit_variant("ModuleKind", 2, "unambiguous")
            }
        }
    }
}
