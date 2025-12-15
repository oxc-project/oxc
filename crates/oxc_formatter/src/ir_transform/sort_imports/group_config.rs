/// Parse groups from string-based configuration.
/// If parsing fails (= undefined), it falls back to `Unknown` selector.
pub fn parse_groups_from_strings(string_groups: &Vec<Vec<String>>) -> Vec<Vec<GroupName>> {
    let mut groups = Vec::with_capacity(string_groups.len());
    for group in string_groups {
        let mut parsed_group = Vec::with_capacity(group.len());
        for name in group {
            parsed_group.push(
                GroupName::parse(name).unwrap_or_else(|| GroupName::new(ImportSelector::Unknown)),
            );
        }
        groups.push(parsed_group);
    }
    groups
}

/// Represents a group name pattern for matching imports.
/// A group name consists of 1 selector and N modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupName {
    pub modifiers: Vec<ImportModifier>,
    pub selector: ImportSelector,
}

impl GroupName {
    /// Create a new group name with no modifiers.
    pub fn new(selector: ImportSelector) -> Self {
        Self { modifiers: vec![], selector }
    }

    /// Create a new group name with one modifier.
    pub fn with_modifier(selector: ImportSelector, modifier: ImportModifier) -> Self {
        Self { modifiers: vec![modifier], selector }
    }

    /// Check if this is a plain selector (no modifiers).
    pub fn is_plain_selector(&self, selector: ImportSelector) -> bool {
        self.selector == selector && self.modifiers.is_empty()
    }

    /// Parse a group name string into a GroupName.
    ///
    /// Format: `(modifier-)*selector`
    /// Examples:
    /// - "external" -> modifiers: (empty), selector: External
    /// - "type-external" -> modifiers: Type, selector: External
    /// - "value-builtin" -> modifiers: Value, selector: Builtin
    /// - "internal-type" -> modifiers: (empty), selector: InternalType
    /// - "side-effect-import" -> modifiers: SideEffect, selector: Import
    /// - "side-effect-type-external" -> modifiers: SideEffect, Type, selector: External
    pub fn parse(s: &str) -> Option<Self> {
        // Try to parse as a selector without modifiers first
        if let Some(selector) = ImportSelector::parse(s) {
            return Some(Self { modifiers: vec![], selector });
        }

        // Split by '-' and try parsing as modifier(s) + selector
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() < 2 {
            return None;
        }

        // Last part should be the selector
        let selector = ImportSelector::parse(parts[parts.len() - 1])?;

        // Everything before should be modifier(s)
        let modifier_parts = &parts[..parts.len() - 1];
        let mut modifiers = vec![];

        // Try to parse the entire modifier string first (handles "side-effect")
        let modifier_str = modifier_parts.join("-");
        if let Some(modifier) = ImportModifier::parse(&modifier_str) {
            modifiers.push(modifier);
        } else {
            // Otherwise, parse each part individually
            for &part in modifier_parts {
                modifiers.push(ImportModifier::parse(part)?);
            }
        }

        Some(Self { modifiers, selector })
    }
}

/// Selector types for import categorization.
/// Selectors identify the type or location of an import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportSelector {
    /// Type-only imports (`import type { ... }`)
    Type,
    /// Side-effect style imports (CSS, SCSS, etc. without bindings)
    SideEffectStyle,
    /// Side-effect imports (imports without bindings)
    SideEffect,
    /// Style file imports (CSS, SCSS, etc.)
    Style,
    /// Type import from index file
    IndexType,
    /// Type import from sibling module
    SiblingType,
    /// Type import from parent module
    ParentType,
    /// Type import from internal module
    InternalType,
    /// Type import from built-in module
    BuiltinType,
    /// Type import from external module
    ExternalType,
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
    /// Unknown/fallback group
    Unknown,
}

impl ImportSelector {
    /// Parse a string into an ImportSelector.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "type" => Some(Self::Type),
            "side-effect-style" => Some(Self::SideEffectStyle),
            "side-effect" => Some(Self::SideEffect),
            "style" => Some(Self::Style),
            "index-type" => Some(Self::IndexType),
            "sibling-type" => Some(Self::SiblingType),
            "parent-type" => Some(Self::ParentType),
            "internal-type" => Some(Self::InternalType),
            "builtin-type" => Some(Self::BuiltinType),
            "external-type" => Some(Self::ExternalType),
            "index" => Some(Self::Index),
            "sibling" => Some(Self::Sibling),
            "parent" => Some(Self::Parent),
            "subpath" => Some(Self::Subpath),
            "internal" => Some(Self::Internal),
            "builtin" => Some(Self::Builtin),
            "external" => Some(Self::External),
            "import" => Some(Self::Import),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

/// Modifier types for import categorization.
/// Modifiers describe characteristics of how an import is declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// Parse a string into an ImportModifier.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "side-effect" => Some(Self::SideEffect),
            "type" => Some(Self::Type),
            "value" => Some(Self::Value),
            "default" => Some(Self::Default),
            "wildcard" => Some(Self::Wildcard),
            "named" => Some(Self::Named),
            _ => None,
        }
    }
}
