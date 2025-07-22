// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`.

#![allow(unused_imports, clippy::match_same_arms, clippy::semicolon_if_nothing_returned)]

use oxc_estree::{
    Concat2, Concat3, ESTree, FlatStructSerializer, JsonSafeString, Serializer, StructSerializer,
};

use crate::raw_transfer_types::*;

impl ESTree for RawTransferData<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("program", &self.program);
        state.serialize_field("comments", &self.comments);
        state.serialize_field("module", &self.module);
        state.serialize_field("errors", &self.errors);
        state.end();
    }
}

impl ESTree for Error<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("severity", &self.severity);
        state.serialize_field("message", &self.message);
        state.serialize_field("labels", &self.labels);
        state.serialize_field("helpMessage", &self.help_message);
        state.serialize_field("codeframe", &self.codeframe);
        state.end();
    }
}

impl ESTree for ErrorSeverity {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Error => JsonSafeString("Error").serialize(serializer),
            Self::Warning => JsonSafeString("Warning").serialize(serializer),
            Self::Advice => JsonSafeString("Advice").serialize(serializer),
        }
    }
}

impl ESTree for ErrorLabel<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("message", &self.message);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for EcmaScriptModule<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("hasModuleSyntax", &self.has_module_syntax);
        state.serialize_field("staticImports", &self.static_imports);
        state.serialize_field("staticExports", &self.static_exports);
        state.serialize_field("dynamicImports", &self.dynamic_imports);
        state.serialize_field("importMetas", &self.import_metas);
        state.end();
    }
}

impl ESTree for StaticImport<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("moduleRequest", &self.module_request);
        state.serialize_field("entries", &self.entries);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for StaticExport<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("entries", &self.entries);
        state.serialize_span(self.span);
        state.end();
    }
}
