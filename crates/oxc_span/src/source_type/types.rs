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
    /// Consider the file a "module" if ESM syntax is present, or else consider it a "script".
    ///
    /// ESM syntax includes `import` statement, `export` statement and `import.meta`.
    ///
    /// Note: Dynamic import expression is not ESM syntax.
    ///
    /// See <https://babel.dev/docs/options#misc-options>
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
