use crate::ir_transform::sort_common::groups::{self, SortVocabulary};

/// The import target's selector/modifier vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImportVocabulary;

impl SortVocabulary for ImportVocabulary {
    type Selector = ImportSelector;
    type Modifier = ImportModifier;

    fn parse_selector(s: &str) -> Option<ImportSelector> {
        ImportSelector::parse(s)
    }

    fn parse_modifier(s: &str) -> Option<ImportModifier> {
        ImportModifier::parse(s)
    }
}

/// A group name for imports, e.g. `"type-external"`.
pub type GroupName = groups::GroupName<ImportVocabulary>;
/// A parsed `groups` entry for imports.
pub type GroupEntry = groups::GroupEntry<ImportVocabulary>;

/// Selector types for import categorization.
/// Selectors identify the type or location of an import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportSelector {
    /// Type-only imports (`import type { ... }`)
    Type,
    /// Side-effect style imports (CSS, SCSS, etc. without bindings)
    SideEffectStyle,
    /// Side-effect imports (imports without bindings)
    SideEffect,
    /// Style file imports (CSS, SCSS, etc.)
    Style,
    /// Index file imports (`./`, `../`)
    Index,
    /// Sibling module imports (`./foo`)
    Sibling,
    /// Parent module imports (`../foo`)
    Parent,
    /// Subpath imports (package.json imports field, e.g., `#foo`)
    Subpath,
    /// Internal module imports (matching internal patterns like `~/`, `@/`)
    Internal,
    /// Built-in module imports (`node:fs`, `fs`)
    Builtin,
    /// External module imports (from node_modules)
    External,
    /// Catch-all selector
    Import,
}

impl ImportSelector {
    /// Parse a string into an ImportSelector.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "type" => Some(Self::Type),
            "side_effect_style" => Some(Self::SideEffectStyle),
            "side_effect" => Some(Self::SideEffect),
            "style" => Some(Self::Style),
            "index" => Some(Self::Index),
            "sibling" => Some(Self::Sibling),
            "parent" => Some(Self::Parent),
            "subpath" => Some(Self::Subpath),
            "internal" => Some(Self::Internal),
            "builtin" => Some(Self::Builtin),
            "external" => Some(Self::External),
            "import" => Some(Self::Import),
            _ => None,
        }
    }

    pub const ALL_SELECTORS: &[ImportSelector] = &[
        ImportSelector::Type,
        ImportSelector::SideEffectStyle,
        ImportSelector::SideEffect,
        ImportSelector::Style,
        ImportSelector::Index,
        ImportSelector::Sibling,
        ImportSelector::Parent,
        ImportSelector::Subpath,
        ImportSelector::Internal,
        ImportSelector::Builtin,
        ImportSelector::External,
        ImportSelector::Import,
    ];

    pub fn name(&self) -> &str {
        match self {
            ImportSelector::Type => "type",
            ImportSelector::SideEffectStyle => "side_effect_style",
            ImportSelector::SideEffect => "side_effect",
            ImportSelector::Style => "style",
            ImportSelector::Index => "index",
            ImportSelector::Sibling => "sibling",
            ImportSelector::Parent => "parent",
            ImportSelector::Subpath => "subpath",
            ImportSelector::Internal => "internal",
            ImportSelector::Builtin => "builtin",
            ImportSelector::External => "external",
            ImportSelector::Import => "import",
        }
    }
}

/// Modifier types for import categorization.
/// Modifiers describe characteristics of how an import is declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportModifier {
    /// Side-effect imports
    SideEffect,
    /// Type-only imports
    Type,
    /// Value imports (non-type)
    Value,
    /// Default specifier present
    Default,
    /// Namespace/wildcard specifier present (`* as`)
    Wildcard,
    /// Named specifiers present
    Named,
}

impl ImportModifier {
    pub const ALL_MODIFIERS: &[ImportModifier] = &[
        ImportModifier::SideEffect,
        ImportModifier::Type,
        ImportModifier::Value,
        ImportModifier::Default,
        ImportModifier::Wildcard,
        ImportModifier::Named,
    ];

    /// Parse a string into an ImportModifier.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "side_effect" => Some(Self::SideEffect),
            "type" => Some(Self::Type),
            "value" => Some(Self::Value),
            "default" => Some(Self::Default),
            "wildcard" => Some(Self::Wildcard),
            "named" => Some(Self::Named),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ImportModifier::SideEffect => "side_effect",
            ImportModifier::Type => "type",
            ImportModifier::Value => "value",
            ImportModifier::Default => "default",
            ImportModifier::Wildcard => "wildcard",
            ImportModifier::Named => "named",
        }
    }
}
