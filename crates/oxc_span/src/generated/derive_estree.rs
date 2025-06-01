// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`.

#![allow(unused_imports, clippy::match_same_arms, clippy::semicolon_if_nothing_returned)]

use oxc_estree::{
    Concat2, Concat3, ESTree, FlatStructSerializer, JsonSafeString, Serializer, StructSerializer,
};

use crate::source_type::*;
use crate::span::*;

impl ESTree for Span {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("start", &self.start);
        state.serialize_field("end", &self.end);
        state.end();
    }
}

impl ESTree for SourceType {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("sourceType", &self.module_kind);
        state.end();
    }
}

impl ESTree for ModuleKind {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Script => JsonSafeString("script").serialize(serializer),
            Self::Module => JsonSafeString("module").serialize(serializer),
            Self::Unambiguous => unreachable!("This enum variant is skipped."),
        }
    }
}
