use std::cmp::Ordering;

/// Represents a group name pattern for matching imports.
/// A group name consists of 1 selector and N modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupName {
    pub selector: ImportSelector,
    pub modifiers: Vec<ImportModifier>,
}

impl GroupName {
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
    /// - "side-effect-import" -> modifiers: SideEffect, selector: Import
    /// - "side-effect-type-external" -> modifiers: SideEffect, Type, selector: External
    /// - "named-side-effect-type-builtin" -> modifiers: SideEffect, Type, Named, selector: Builtin
    pub fn parse(s: &str) -> Option<Self> {
        // Try to parse as a selector without modifiers first
        if let Some(selector) = ImportSelector::parse(s) {
            return Some(Self { modifiers: vec![], selector });
        }

        // Last part should be the selector
        let selector =
            *ImportSelector::ALL_SELECTORS.iter().find(|selector| s.ends_with(selector.name()))?;

        // The remaining part represents a sequence of modifiers joined by "-".
        // Since modifiers themselves may contain "-", splitting by "-" would be ambiguous.
        // Instead, we iterate over modifiers in a predefined order and check
        // whether they appear in the remaining string.
        // This guarantees the extracted modifiers are already ordered
        // and no additional sorting is required.
        //
        // The trade-off is that this approach may tolerate invalid input,
        // as unmatched or malformed segments are not strictly rejected.
        let mut modifiers = Vec::with_capacity(ImportModifier::ALL_MODIFIERS.len());
        for m in ImportModifier::ALL_MODIFIERS {
            if s.contains(m.name()) {
                modifiers.push(*m);
            }
        }

        Some(Self { selector, modifiers })
    }

    /// Check if it represents a possible group name of the given import.
    pub fn is_a_possible_name_of(
        &self,
        selectors: &[ImportSelector],
        modifiers: &[ImportModifier],
    ) -> bool {
        selectors.contains(&self.selector) && self.modifiers.iter().all(|m| modifiers.contains(m))
    }
}

impl PartialOrd for GroupName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GroupName {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.selector.cmp(&other.selector) {
            Ordering::Equal => {}
            ord => return ord,
        }
        let self_modifier_cnt = self.modifiers.len();
        let other_modifier_cnt = other.modifiers.len();
        if self_modifier_cnt > other_modifier_cnt {
            return Ordering::Less;
        } else if self_modifier_cnt < other_modifier_cnt {
            return Ordering::Greater;
        }
        self.modifiers.cmp(&other.modifiers)
    }
}

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
            "side-effect-style" => Some(Self::SideEffectStyle),
            "side-effect" => Some(Self::SideEffect),
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
            ImportSelector::SideEffectStyle => "side-effect-style",
            ImportSelector::SideEffect => "side-effect",
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

    pub fn name(&self) -> &str {
        match self {
            ImportModifier::SideEffect => "side-effect",
            ImportModifier::Type => "type",
            ImportModifier::Value => "value",
            ImportModifier::Default => "default",
            ImportModifier::Wildcard => "wildcard",
            ImportModifier::Named => "named",
        }
    }
}
