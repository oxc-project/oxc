use std::{borrow::Cow, path::Path};

use cow_utils::CowUtils;
use phf::phf_set;

use crate::{formatter::format_element::FormatElement, options};

use super::source_line::{ImportLineMetadata, SourceLine};

#[derive(Debug, Clone)]
pub struct SortableImport<'a> {
    pub leading_lines: Vec<SourceLine>,
    pub import_line: SourceLine,
    pub group_idx: usize,
    pub normalized_source: Cow<'a, str>,
    pub is_ignored: bool,
}

impl<'a> SortableImport<'a> {
    pub fn new(leading_lines: Vec<SourceLine>, import_line: SourceLine) -> Self {
        Self {
            leading_lines,
            import_line,
            // These will be computed by `collect_sort_keys()`
            group_idx: 0,
            normalized_source: Cow::Borrowed(""),
            is_ignored: false,
        }
    }

    /// Pre-compute keys needed for sorting.
    #[must_use]
    pub fn collect_sort_keys(
        mut self,
        elements: &'a [FormatElement],
        options: &options::SortImports,
    ) -> Self {
        let SourceLine::Import(
            _,
            ImportLineMetadata {
                source_idx,
                is_side_effect,
                is_type_import,
                has_default_specifier,
                has_namespace_specifier,
                has_named_specifier,
            },
        ) = &self.import_line
        else {
            unreachable!("`import_line` must be of type `SourceLine::Import`.");
        };

        let source = extract_source_text(elements, *source_idx);

        // Pre-compute normalized source for case-insensitive comparison
        self.normalized_source =
            if options.ignore_case { source.cow_to_lowercase() } else { Cow::Borrowed(source) };

        // Create group matcher from import characteristics
        let matcher = ImportGroupMatcher {
            is_side_effect: *is_side_effect,
            is_type_import: *is_type_import,
            is_style_import: is_style(source),
            has_default_specifier: *has_default_specifier,
            has_namespace_specifier: *has_namespace_specifier,
            has_named_specifier: *has_named_specifier,
            path_kind: to_path_kind(source),
        };
        self.group_idx = matcher.match_group(&options.groups);

        // TODO: Check ignore comments?
        self.is_ignored = !options.sort_side_effects && *is_side_effect;

        self
    }
}

// ---

/// Helper for matching imports to configured groups.
///
/// Contains all characteristics of an import needed to determine which group it belongs to,
/// such as whether it's a type import, side-effect import, style import, and what kind of path it uses.
#[derive(Debug, Clone)]
struct ImportGroupMatcher {
    is_side_effect: bool,
    is_type_import: bool,
    is_style_import: bool,
    has_default_specifier: bool,
    has_namespace_specifier: bool,
    has_named_specifier: bool,
    path_kind: ImportPathKind,
}

impl ImportGroupMatcher {
    /// Match this import against the configured groups and return the group index.
    /// Returns the index of the first matching group, or the index of "unknown" group if present,
    /// or the last index + 1 if no match found.
    ///
    /// Matching prioritizes more specific group names (e.g., "type-external" over "type-import").
    pub fn match_group(&self, groups: &[Vec<String>]) -> usize {
        let possible_names = self.generate_group_names();
        let mut unknown_index = None;

        // Try each possible name in order (most specific first)
        for possible_name in &possible_names {
            for (group_idx, group) in groups.iter().enumerate() {
                for group_name in group {
                    // Check if this is the "unknown" group
                    if group_name == "unknown" {
                        unknown_index = Some(group_idx);
                    }

                    // Check if this possible name matches this group
                    if possible_name == group_name {
                        return group_idx;
                    }
                }
            }
        }

        // No match found - use "unknown" group if present, otherwise return last + 1
        unknown_index.unwrap_or(groups.len())
    }

    /// Generate all possible group names for this import, ordered by specificity.
    /// Returns group names in the format used by perfectionist.
    ///
    /// Perfectionist format examples:
    /// - `type-external` - type modifier + path selector
    /// - `value-internal` - value modifier + path selector
    /// - `type-import` - type modifier + import selector
    /// - `external` - path selector only
    fn generate_group_names(&self) -> Vec<String> {
        let selectors = self.selectors();
        let modifiers = self.modifiers();

        let mut group_names = Vec::new();

        // Most specific: type/value modifier combined with path selectors
        // e.g., "type-external", "value-internal", "type-parent"
        let type_or_value_modifier = if self.is_type_import { "type" } else { "value" };

        for selector in &selectors {
            // Skip the generic "type" selector since it's already in the modifier
            if matches!(selector, ImportSelector::Type) {
                continue;
            }

            // For path-based selectors, combine with type/value modifier
            if matches!(
                selector,
                ImportSelector::Builtin
                    | ImportSelector::External
                    | ImportSelector::Internal
                    | ImportSelector::Parent
                    | ImportSelector::Sibling
                    | ImportSelector::Index
            ) {
                let name = format!("{}-{}", type_or_value_modifier, selector.as_str());
                group_names.push(name);
            }
        }

        // Add other modifier combinations for special selectors
        for selector in &selectors {
            // Skip path-based selectors (already handled above) and "import" selector
            if matches!(
                selector,
                ImportSelector::Builtin
                    | ImportSelector::External
                    | ImportSelector::Internal
                    | ImportSelector::Parent
                    | ImportSelector::Sibling
                    | ImportSelector::Index
                    | ImportSelector::Import
                    | ImportSelector::Type
            ) {
                continue;
            }

            // For special selectors like side-effect, side-effect-style, style
            // combine with relevant modifiers
            for modifier in &modifiers {
                let name = format!("{}-{}", modifier.as_str(), selector.as_str());
                group_names.push(name);
            }

            // Selector-only name
            group_names.push(selector.as_str().to_string());
        }

        // Add "type-import" or "value-import" or just "import"
        if self.is_type_import {
            group_names.push("type-import".to_string());
        }

        group_names.push("import".to_string());

        group_names
    }

    /// Compute all selectors for this import, ordered from most to least specific.
    fn selectors(&self) -> Vec<ImportSelector> {
        let mut selectors = Vec::new();

        // Most specific selectors first
        if self.is_side_effect && self.is_style_import {
            selectors.push(ImportSelector::SideEffectStyle);
        }
        if self.is_side_effect {
            selectors.push(ImportSelector::SideEffect);
        }
        if self.is_style_import {
            selectors.push(ImportSelector::Style);
        }
        // Type selector
        if self.is_type_import {
            selectors.push(ImportSelector::Type);
        }
        // Path-based selectors
        match self.path_kind {
            ImportPathKind::Index => selectors.push(ImportSelector::Index),
            ImportPathKind::Sibling => selectors.push(ImportSelector::Sibling),
            ImportPathKind::Parent => selectors.push(ImportSelector::Parent),
            ImportPathKind::Internal => selectors.push(ImportSelector::Internal),
            ImportPathKind::Builtin => selectors.push(ImportSelector::Builtin),
            ImportPathKind::External => selectors.push(ImportSelector::External),
            ImportPathKind::Unknown => {}
        }
        // Catch-all selector
        selectors.push(ImportSelector::Import);

        selectors
    }

    /// Compute all modifiers for this import.
    fn modifiers(&self) -> Vec<ImportModifier> {
        let mut modifiers = Vec::new();

        if self.is_side_effect {
            modifiers.push(ImportModifier::SideEffect);
        }
        if self.is_type_import {
            modifiers.push(ImportModifier::Type);
        } else {
            modifiers.push(ImportModifier::Value);
        }
        if self.has_default_specifier {
            modifiers.push(ImportModifier::Default);
        }
        if self.has_namespace_specifier {
            modifiers.push(ImportModifier::Wildcard);
        }
        if self.has_named_specifier {
            modifiers.push(ImportModifier::Named);
        }

        modifiers
    }
}

/// Selector types for import categorization.
/// Selectors identify the type or location of an import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportSelector {
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
    /// Returns the string representation used in group names.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::SideEffectStyle => "side-effect-style",
            Self::SideEffect => "side-effect",
            Self::Style => "style",
            Self::Index => "index",
            Self::Sibling => "sibling",
            Self::Parent => "parent",
            Self::Internal => "internal",
            Self::Builtin => "builtin",
            Self::External => "external",
            Self::Import => "import",
        }
    }
}

/// Modifier types for import categorization.
/// Modifiers describe characteristics of how an import is declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportModifier {
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
    /// Returns the string representation used in group names.
    const fn as_str(self) -> &'static str {
        match self {
            Self::SideEffect => "side-effect",
            Self::Type => "type",
            Self::Value => "value",
            Self::Default => "default",
            Self::Wildcard => "wildcard",
            Self::Named => "named",
        }
    }
}

// ---

/// Extract the import source text from format elements.
///
/// This removes quotes and query parameters from the source string.
/// For example, `"./foo?bar"` becomes `./foo`.
fn extract_source_text<'a>(elements: &'a [FormatElement], source_idx: usize) -> &'a str {
    let source = match &elements[source_idx] {
        FormatElement::Text { text, .. } => *text,
        _ => unreachable!("`source_idx` must point to the `Text` in the `elements`."),
    };

    let source = source.trim_matches('"').trim_matches('\'');
    source.split('?').next().unwrap_or(source)
}

// spellchecker:off
static STYLE_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "css",
    "scss",
    "sass",
    "less",
    "styl",
    "pcss",
    "sss",
};
// spellchecker:on

/// Check if an import source is a style file based on its extension.
fn is_style(source: &str) -> bool {
    Path::new(source)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| STYLE_EXTENSIONS.contains(ext))
}

static NODE_BUILTINS: phf::Set<&'static str> = phf_set! {
    "assert", "async_hooks", "buffer", "child_process", "cluster", "console",
    "constants", "crypto", "dgram", "diagnostics_channel", "dns", "domain",
    "events", "fs", "http", "http2", "https", "inspector", "module", "net",
    "os", "path", "perf_hooks", "process", "punycode", "querystring",
    "readline", "repl", "stream", "string_decoder", "sys", "timers", "tls",
    "trace_events", "tty", "url", "util", "v8", "vm", "wasi", "worker_threads",
    "zlib",
};

/// Check if an import source is a Node.js or Bun builtin module.
fn is_builtin(source: &str) -> bool {
    source.starts_with("node:") || source.starts_with("bun:") || NODE_BUILTINS.contains(source)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImportPathKind {
    /// Node.js builtin module (e.g., `node:fs`, `fs`)
    Builtin,
    /// External package from node_modules (e.g., `react`, `lodash`)
    External,
    /// Internal module matching internal patterns (e.g., `~/...`, `@/...`)
    Internal,
    /// Parent directory relative import (e.g., `../foo`)
    Parent,
    /// Sibling directory relative import (e.g., `./foo`)
    Sibling,
    /// Index file import (e.g., `./`, `../`)
    Index,
    /// Unknown or unclassified
    #[default]
    Unknown,
}

/// Determine the path kind for an import source.
fn to_path_kind(source: &str) -> ImportPathKind {
    if is_builtin(source) {
        return ImportPathKind::Builtin;
    }

    if source.starts_with('.') {
        if source == "." || source == ".." || source.ends_with('/') {
            return ImportPathKind::Index;
        }
        if source.starts_with("../") {
            return ImportPathKind::Parent;
        }
        return ImportPathKind::Sibling;
    }

    // TODO: This can be changed via `options.internalPattern`
    if source.starts_with('~') || source.starts_with('@') {
        return ImportPathKind::Internal;
    }

    ImportPathKind::External
}
