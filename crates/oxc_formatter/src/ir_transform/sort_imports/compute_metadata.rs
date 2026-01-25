use std::{borrow::Cow, path::Path};

use cow_utils::CowUtils;
use phf::phf_set;

use crate::ir_transform::sort_imports::{
    group_config::{ImportModifier, ImportSelector},
    group_matcher::{GroupMatcher, ImportMetadata},
    options::SortImportsOptions,
    source_line::ImportLineMetadata,
};

/// Compute all metadata derived from import line metadata.
///
/// Returns `(group_idx, normalized_source, is_ignored)`.
pub fn compute_import_metadata<'a>(
    metadata: &ImportLineMetadata<'a>,
    group_matcher: &GroupMatcher,
    options: &SortImportsOptions,
) -> (usize, Cow<'a, str>, bool) {
    let source = extract_source_path(metadata.source);
    let is_style_import = is_style(source);
    let path_kind = to_path_kind(source, options);

    let group_idx = group_matcher.compute_group_index(&ImportMetadata {
        source,
        selectors: compute_selectors(metadata, is_style_import, is_subpath(source), &path_kind),
        modifiers: compute_modifiers(metadata),
    });

    // Pre-compute normalized source for case-insensitive comparison
    let normalized_source =
        if options.ignore_case { source.cow_to_lowercase() } else { Cow::Borrowed(source) };

    // Determine if this import should be ignored (not moved between groups)
    // - If `sort_side_effects: true`, never ignore
    // - If `sort_side_effects: false` and this is a side-effect:
    //   - Check if groups contain `side-effect` or `side-effect-style`
    //     - If yes, allow regrouping (not ignored)
    //     - If no, keep in original position (ignored)
    let should_regroup_side_effect = group_matcher.should_regroup_side_effect();
    let should_regroup_side_effect_style = group_matcher.should_regroup_side_effect_style();

    let is_ignored = !options.sort_side_effects
        && metadata.is_side_effect
        && !should_regroup_side_effect
        && (!is_style_import || !should_regroup_side_effect_style);

    (group_idx, normalized_source, is_ignored)
}

// ---

/// Compute all selectors for this import, ordered from most to least specific.
///
/// Order matches perfectionist implementation:
/// 1. Special selectors (side-effect-style, side-effect, style) - most specific
/// 2. Path-type selectors (parent-type, external-type, etc.) for type imports
/// 3. Type selector
/// 4. Path-based selectors (builtin, external, internal, parent, sibling, index)
/// 5. Catch-all import selector
fn compute_selectors(
    metadata: &ImportLineMetadata,
    is_style_import: bool,
    is_subpath: bool,
    path_kind: &ImportPathKind,
) -> Vec<ImportSelector> {
    let mut selectors = vec![];

    // Most specific selectors first
    if metadata.is_side_effect && is_style_import {
        selectors.push(ImportSelector::SideEffectStyle);
    }
    if metadata.is_side_effect {
        selectors.push(ImportSelector::SideEffect);
    }
    if is_style_import {
        selectors.push(ImportSelector::Style);
    }

    // For type imports, add path-type selectors (e.g., "parent-type", "external-type")
    // These come before the generic "type" selector
    if metadata.is_type_import {
        // Type selector
        selectors.push(ImportSelector::Type);
    }

    // Path-based selectors
    // Order matches perfectionist: index, sibling, parent, subpath, internal, builtin, external
    match path_kind {
        ImportPathKind::Index => selectors.push(ImportSelector::Index),
        ImportPathKind::Sibling => selectors.push(ImportSelector::Sibling),
        ImportPathKind::Parent => selectors.push(ImportSelector::Parent),
        _ => {}
    }

    // Subpath selector (independent of path kind, comes after parent)
    if is_subpath {
        selectors.push(ImportSelector::Subpath);
    }

    // Continue with remaining path-based selectors
    match path_kind {
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
fn compute_modifiers(metadata: &ImportLineMetadata) -> Vec<ImportModifier> {
    let mut modifiers = vec![];

    if metadata.is_side_effect {
        modifiers.push(ImportModifier::SideEffect);
    }
    if metadata.is_type_import {
        modifiers.push(ImportModifier::Type);
    } else {
        modifiers.push(ImportModifier::Value);
    }
    if metadata.has_default_specifier {
        modifiers.push(ImportModifier::Default);
    }
    if metadata.has_namespace_specifier {
        modifiers.push(ImportModifier::Wildcard);
    }
    if metadata.has_named_specifier {
        modifiers.push(ImportModifier::Named);
    }

    modifiers
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

/// Check if an import source is a Node.js or Bun builtin module.
fn is_builtin(source: &str) -> bool {
    source.starts_with("node:")
        || source.starts_with("bun:")
        || nodejs_built_in_modules::is_nodejs_builtin_module(source)
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
        if source.starts_with("..") {
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
