// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use oxc_ast_macros::ast;
#[cfg(feature = "serialize")]
use {serde::Serialize, tsify::Tsify};

/// Source Type for JavaScript vs TypeScript / Script vs Module / JSX
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SourceType {
    /// JavaScript or TypeScript, default JavaScript
    pub(super) language: Language,

    /// Script or Module, default Module
    pub(super) module_kind: ModuleKind,

    /// Support JSX for JavaScript and TypeScript? default without JSX
    pub(super) variant: LanguageVariant,

    /// Mark strict mode as always strict
    ///
    /// See <https://github.com/tc39/test262/blob/main/INTERPRETING.md#strict-mode>
    pub(super) always_strict: bool,
}

/// JavaScript or TypeScript
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "lowercase")]
pub enum Language {
    JavaScript = 0,
    TypeScript = 1,
    #[serde(rename = "typescriptDefinition")]
    TypeScriptDefinition = 2,
}

/// Script or Module
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum ModuleKind {
    /// Regular JS script or CommonJS file
    Script = 0,
    /// ES6 Module
    Module = 1,
    /// Unambiguous
    Unambiguous = 2,
}

/// JSX for JavaScript and TypeScript
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum LanguageVariant {
    Standard = 0,
    Jsx = 1,
}
