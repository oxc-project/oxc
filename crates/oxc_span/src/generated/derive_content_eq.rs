// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_eq.rs`.

#![allow(clippy::match_same_arms)]

use oxc_span::ContentEq;

use crate::source_type::*;

impl ContentEq for SourceType {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.language, &other.language)
            && ContentEq::content_eq(&self.module_kind, &other.module_kind)
            && ContentEq::content_eq(&self.variant, &other.variant)
            && ContentEq::content_eq(&self.extension, &other.extension)
    }
}

impl ContentEq for Language {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for ModuleKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for LanguageVariant {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for FileExtension {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}
