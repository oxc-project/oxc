use std::{borrow::Cow, path::Path};

use cow_utils::CowUtils;
use phf::phf_set;

use crate::ir_transform::sort_imports::{
    group_config::{GroupName, ImportModifier, ImportSelector},
    options::SortImportsOptions,
    source_line::ImportLineMetadata,
};

/// Compute all metadata derived from import line metadata.
///
/// Returns `(group_idx, normalized_source, is_ignored)`.
pub fn compute_import_metadata<'a>(
    metadata: &ImportLineMetadata<'a>,
    groups: &[Vec<GroupName>],
    options: &SortImportsOptions,
) -> (usize, Cow<'a, str>, bool) {
    let ImportLineMetadata {
        source,
        is_side_effect,
        is_type_import,
        has_default_specifier,
        has_namespace_specifier,
        has_named_specifier,
    } = metadata;

    let source = extract_source_path(source);
    let is_style_import = is_style(source);
    let path_kind = to_path_kind(source, options);

    // Create group matcher from import characteristics
    let matcher = ImportGroupMatcher {
        is_side_effect: *is_side_effect,
        is_type_import: *is_type_import,
        is_style_import,
        path_kind,
        is_subpath: is_subpath(source),
        has_default_specifier: *has_default_specifier,
        has_namespace_specifier: *has_namespace_specifier,
        has_named_specifier: *has_named_specifier,
    };
    let group_idx = matcher.into_match_group_idx(groups);

    // Pre-compute normalized source for case-insensitive comparison
    let normalized_source =
        if options.ignore_case { source.cow_to_lowercase() } else { Cow::Borrowed(source) };

    // Determine if this import should be ignored (not moved between groups)
    // - If `sort_side_effects: true`, never ignore
    // - If `sort_side_effects: false` and this is a side-effect:
    //   - Check if groups contain `side-effect` or `side-effect-style`
    //     - If yes, allow regrouping (not ignored)
    //     - If no, keep in original position (ignored)
    let mut should_regroup_side_effect = false;
    let mut should_regroup_side_effect_style = false;
    for group in groups {
        for name in group {
            if name.is_plain_selector(ImportSelector::SideEffect) {
                should_regroup_side_effect = true;
            }
            if name.is_plain_selector(ImportSelector::SideEffectStyle) {
                should_regroup_side_effect_style = true;
            }
        }
    }

    let is_ignored = !options.sort_side_effects
        && *is_side_effect
        && !should_regroup_side_effect
        && (!is_style_import || !should_regroup_side_effect_style);

    (group_idx, normalized_source, is_ignored)
}

// ---

/// Helper for matching imports to configured groups.
///
/// Contains all characteristics of an import needed to determine which group it belongs to,
/// such as whether it's a type import, side-effect import, style import, and what kind of path it uses.
#[derive(Debug)]
struct ImportGroupMatcher {
    is_side_effect: bool,
    is_type_import: bool,
    is_style_import: bool,
    has_default_specifier: bool,
    has_namespace_specifier: bool,
    has_named_specifier: bool,
    path_kind: ImportPathKind,
    is_subpath: bool,
}

impl ImportGroupMatcher {
    /// Match this import against the configured groups and return the group index.
    ///
    /// This method generates possible group names in priority order (most specific to least specific)
    /// and tries to match them against the configured groups.
    /// For example, for a type import from an external package,
    /// it tries: "type-external", "external", "type-import", "import".
    ///
    /// Returns:
    /// - The index of the first matching group (if found)
    /// - The index of the "unknown" group (if no match found and "unknown" is configured)
    /// - `groups.len()` (if no match found and no "unknown" group configured)
    #[must_use]
    fn into_match_group_idx(self, groups: &[Vec<GroupName>]) -> usize {
        let possible_names = self.compute_group_names();
        let mut unknown_index = None;

        // Try each possible name in order (most specific first)
        for possible_name in &possible_names {
            for (group_idx, group) in groups.iter().enumerate() {
                for group_name in group {
                    // Check if this is the "unknown" group
                    if group_name.is_plain_selector(ImportSelector::Unknown) {
                        unknown_index = Some(group_idx);
                    }

                    // Check if this possible name matches this group
                    if possible_name == group_name {
                        return group_idx;
                    }
                }
            }
        }

        unknown_index.unwrap_or(groups.len())
    }

    /// Generate all possible group names for this import, ordered by specificity.
    /// For each selector (in order), generate all modifier combinations with that selector.
    ///
    /// Example with:
    /// - selectors: "style", "parent"
    /// - and modifiers: "value", "default"
    ///
    /// Generates:
    /// - value-default-style, value-style, default-style, style
    /// - value-default-parent, value-parent, default-parent, parent
    fn compute_group_names(&self) -> Vec<GroupName> {
        let selectors = self.compute_selectors();
        let modifiers = self.compute_modifiers();

        let mut group_names = vec![];

        // For each selector, generate all modifier combinations
        for selector in &selectors {
            match selector {
                // For path selectors, combine with type/value modifier
                ImportSelector::Builtin
                | ImportSelector::External
                | ImportSelector::Internal
                | ImportSelector::Parent
                | ImportSelector::Sibling
                | ImportSelector::Index
                | ImportSelector::Subpath => {
                    let modifier = if self.is_type_import {
                        ImportModifier::Type
                    } else {
                        ImportModifier::Value
                    };
                    group_names.push(GroupName::with_modifier(*selector, modifier));
                }
                // For special selectors (side-effect, style, etc.), combine with all modifiers
                ImportSelector::SideEffectStyle
                | ImportSelector::SideEffect
                | ImportSelector::Style
                | ImportSelector::Import => {
                    for modifier in &modifiers {
                        group_names.push(GroupName::with_modifier(*selector, *modifier));
                    }
                }
                _ => {}
            }
            group_names.push(GroupName::new(*selector));
        }

        // Add final "import" catch-all with modifiers
        // This generates combinations like "side-effect-import", "type-import", "value-import", etc.
        for modifier in &modifiers {
            group_names.push(GroupName::with_modifier(ImportSelector::Import, *modifier));
        }
        group_names.push(GroupName::new(ImportSelector::Import));

        group_names
    }

    /// Compute all selectors for this import, ordered from most to least specific.
    ///
    /// Order matches perfectionist implementation:
    /// 1. Special selectors (side-effect-style, side-effect, style) - most specific
    /// 2. Path-type selectors (parent-type, external-type, etc.) for type imports
    /// 3. Type selector
    /// 4. Path-based selectors (builtin, external, internal, parent, sibling, index)
    /// 5. Catch-all import selector
    fn compute_selectors(&self) -> Vec<ImportSelector> {
        let mut selectors = vec![];

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

        // For type imports, add path-type selectors (e.g., "parent-type", "external-type")
        // These come before the generic "type" selector
        if self.is_type_import {
            match self.path_kind {
                ImportPathKind::Index => selectors.push(ImportSelector::IndexType),
                ImportPathKind::Sibling => selectors.push(ImportSelector::SiblingType),
                ImportPathKind::Parent => selectors.push(ImportSelector::ParentType),
                ImportPathKind::Internal => selectors.push(ImportSelector::InternalType),
                ImportPathKind::Builtin => selectors.push(ImportSelector::BuiltinType),
                ImportPathKind::External => selectors.push(ImportSelector::ExternalType),
                ImportPathKind::Unknown => {}
            }
            // Type selector
            selectors.push(ImportSelector::Type);
        }

        // Path-based selectors
        // Order matches perfectionist: index, sibling, parent, subpath, internal, builtin, external
        match self.path_kind {
            ImportPathKind::Index => selectors.push(ImportSelector::Index),
            ImportPathKind::Sibling => selectors.push(ImportSelector::Sibling),
            ImportPathKind::Parent => selectors.push(ImportSelector::Parent),
            _ => {}
        }

        // Subpath selector (independent of path kind, comes after parent)
        if self.is_subpath {
            selectors.push(ImportSelector::Subpath);
        }

        // Continue with remaining path-based selectors
        match self.path_kind {
            ImportPathKind::Internal => selectors.push(ImportSelector::Internal),
            ImportPathKind::Builtin => selectors.push(ImportSelector::Builtin),
            ImportPathKind::External => selectors.push(ImportSelector::External),
            _ => {}
        }

        // Catch-all selector
        selectors.push(ImportSelector::Import);

        selectors
    }

    /// Compute all modifiers for this import.
    fn compute_modifiers(&self) -> Vec<ImportModifier> {
        let mut modifiers = vec![];

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

// ---

/// Extract the import source path.
///
/// This removes quotes and query parameters from the source string.
/// For example, `"./foo.js?bar"` becomes `./foo.js`.
fn extract_source_path(source: &str) -> &str {
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

#[derive(Debug, PartialEq, Eq, Default)]
enum ImportPathKind {
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
fn to_path_kind(source: &str, options: &SortImportsOptions) -> ImportPathKind {
    if is_builtin(source) {
        return ImportPathKind::Builtin;
    }

    if source.starts_with('.') {
        if matches!(
            source,
            "." | "./" | "./index" | "./index.js" | "./index.ts" | "./index.d.ts" | "./index.d.js"
        ) {
            return ImportPathKind::Index;
        }
        if source.starts_with("../") {
            return ImportPathKind::Parent;
        }
        return ImportPathKind::Sibling;
    }

    // Check if source matches any internal pattern
    if options.internal_pattern.iter().any(|p| source.starts_with(p.as_str())) {
        return ImportPathKind::Internal;
    }

    // Subpath imports (e.g., `#foo`) are also considered external
    ImportPathKind::External
}

/// Check if an import source is a subpath import (starts with '#').
fn is_subpath(source: &str) -> bool {
    source.starts_with('#')
}
