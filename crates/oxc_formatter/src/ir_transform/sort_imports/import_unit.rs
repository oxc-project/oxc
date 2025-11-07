use std::path::Path;

use cow_utils::CowUtils;
use phf::phf_set;

use crate::{formatter::format_element::FormatElement, options};

use super::source_line::{ImportLine, SourceLine};

#[derive(Debug)]
pub struct ImportUnits(pub Vec<SortableImport>);

impl IntoIterator for ImportUnits {
    type Item = SortableImport;
    type IntoIter = std::vec::IntoIter<SortableImport>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl ImportUnits {
    pub fn sort_imports(&mut self, elements: &[FormatElement], options: options::SortImports) {
        let imports_len = self.0.len();

        // Perform sorting only if needed
        if imports_len < 2 {
            return;
        }

        // Separate imports into:
        // - sortable: indices of imports that should be sorted
        // - fixed: indices of imports that should be ignored
        //   - e.g. side-effect imports when `sort_side_effects: false`, with ignore comments, etc...
        let mut sortable_indices = vec![];
        let mut fixed_indices = vec![];
        for (idx, si) in self.0.iter().enumerate() {
            if si.is_ignored(options) {
                fixed_indices.push(idx);
            } else {
                sortable_indices.push(idx);
            }
        }

        // Sort indices by comparing their corresponding import groups, then sources
        sortable_indices.sort_by(|&a, &b| {
            let metadata_a = self.0[a].get_metadata(elements);
            let metadata_b = self.0[b].get_metadata(elements);

            // First, compare by group
            let group_ord = metadata_a.group().cmp(&metadata_b.group());
            if group_ord != std::cmp::Ordering::Equal {
                return if options.order.is_desc() { group_ord.reverse() } else { group_ord };
            }

            // Within the same group, compare by source
            let source_ord = if options.ignore_case {
                metadata_a.source.cow_to_lowercase().cmp(&metadata_b.source.cow_to_lowercase())
            } else {
                metadata_a.source.cmp(metadata_b.source)
            };

            if options.order.is_desc() { source_ord.reverse() } else { source_ord }
        });

        // Create a permutation map
        let mut permutation = vec![0; imports_len];
        let mut sortable_iter = sortable_indices.into_iter();
        for (idx, perm) in permutation.iter_mut().enumerate() {
            // NOTE: This is O(n), but side-effect imports are usually few
            if fixed_indices.contains(&idx) {
                *perm = idx;
            } else if let Some(sorted_idx) = sortable_iter.next() {
                *perm = sorted_idx;
            }
        }
        debug_assert!(
            permutation.iter().copied().collect::<rustc_hash::FxHashSet<_>>().len() == imports_len,
            "`permutation` must be a valid permutation, all indices must be unique."
        );

        // Apply permutation in-place using cycle decomposition
        let mut visited = vec![false; imports_len];
        for idx in 0..imports_len {
            // Already visited or already in the correct position
            if visited[idx] || permutation[idx] == idx {
                continue;
            }
            // Follow the cycle
            let mut current = idx;
            loop {
                let next = permutation[current];
                visited[current] = true;
                if next == idx {
                    break;
                }
                self.0.swap(current, next);
                current = next;
            }
        }
        debug_assert!(self.0.len() == imports_len, "Length must remain the same after sorting.");
    }
}

// ---

#[derive(Debug, Clone)]
pub struct SortableImport {
    pub leading_lines: Vec<SourceLine>,
    pub import_line: SourceLine,
}

impl SortableImport {
    pub fn new(leading_lines: Vec<SourceLine>, import_line: SourceLine) -> Self {
        Self { leading_lines, import_line }
    }

    /// Get all import metadata in one place.
    pub fn get_metadata<'a>(&self, elements: &'a [FormatElement]) -> ImportMetadata<'a> {
        let SourceLine::Import(ImportLine {
            source_idx,
            is_side_effect,
            is_type_import,
            has_default_specifier,
            has_namespace_specifier,
            has_named_specifier,
            ..
        }) = &self.import_line
        else {
            unreachable!("`import_line` must be of type `SourceLine::Import`.");
        };

        // Strip quotes and params
        let source = match &elements[*source_idx] {
            FormatElement::Text { text } => *text,
            _ => unreachable!(
                "`source_idx` must point to either `LocatedTokenText` or `Text` in the `elements`."
            ),
        };
        let source = source.trim_matches('"').trim_matches('\'');
        let source = source.split('?').next().unwrap_or(source);

        ImportMetadata {
            source,
            is_side_effect: *is_side_effect,
            is_type_import: *is_type_import,
            is_style_import: is_style(source),
            has_default_specifier: *has_default_specifier,
            has_namespace_specifier: *has_namespace_specifier,
            has_named_specifier: *has_named_specifier,
            path_kind: ImportPathKind::new(source),
        }
    }

    /// Check if this import should be ignored (not sorted).
    pub fn is_ignored(&self, options: options::SortImports) -> bool {
        match self.import_line {
            SourceLine::Import(ImportLine { is_side_effect, .. }) => {
                // TODO: Check ignore comments?
                !options.sort_side_effects && is_side_effect
            }
            _ => unreachable!("`import_line` must be of type `SourceLine::Import`."),
        }
    }
}

/// Import group classification for sorting.
///
/// NOTE: The order of variants in this enum determines the sort order when comparing groups.
/// Groups are sorted in the order they appear here (TypeImport first, Unknown last).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImportGroup {
    /// Type-only imports from builtin or external packages
    /// e.g., `import type { Foo } from 'react'`
    TypeImport,
    /// Value imports from Node.js builtin modules or external packages
    /// Corresponds to `['value-builtin', 'value-external']` in perfectionist
    /// e.g., `import fs from 'node:fs'`, `import React from 'react'`
    ValueBuiltinOrExternal,
    /// Type-only imports from internal modules
    /// e.g., `import type { Config } from '~/types'`, `import type { User } from '@/models'`
    TypeInternal,
    /// Value imports from internal modules
    /// e.g., `import { config } from '~/config'`, `import { utils } from '@/utils'`
    ValueInternal,
    /// Type-only imports from relative paths (parent, sibling, or index)
    /// Corresponds to `['type-parent', 'type-sibling', 'type-index']` in perfectionist
    /// e.g., `import type { Props } from '../types'`, `import type { State } from './types'`
    TypeRelative,
    /// Value imports from relative paths (parent, sibling, or index)
    /// Corresponds to `['value-parent', 'value-sibling', 'value-index']` in perfectionist
    /// e.g., `import { helper } from '../parent'`, `import { Component } from './sibling'`
    ValueRelative,
    /// Unclassified imports (fallback)
    Unknown,
}

/// Metadata about an import for sorting purposes.
#[derive(Debug, Clone)]
pub struct ImportMetadata<'a> {
    pub source: &'a str,
    pub is_side_effect: bool,
    pub is_type_import: bool,
    pub is_style_import: bool,
    pub has_default_specifier: bool,
    pub has_namespace_specifier: bool,
    pub has_named_specifier: bool,
    pub path_kind: ImportPathKind,
}

impl ImportMetadata<'_> {
    /// Determine the import group based on metadata.
    pub fn group(&self) -> ImportGroup {
        if self.is_type_import {
            return match self.path_kind {
                ImportPathKind::Builtin | ImportPathKind::External => ImportGroup::TypeImport,
                ImportPathKind::Internal => ImportGroup::TypeInternal,
                ImportPathKind::Parent | ImportPathKind::Sibling | ImportPathKind::Index => {
                    ImportGroup::TypeRelative
                }
                ImportPathKind::Unknown => ImportGroup::Unknown,
            };
        }

        match self.path_kind {
            ImportPathKind::Builtin | ImportPathKind::External => {
                ImportGroup::ValueBuiltinOrExternal
            }
            ImportPathKind::Internal => ImportGroup::ValueInternal,
            ImportPathKind::Parent | ImportPathKind::Sibling | ImportPathKind::Index => {
                ImportGroup::ValueRelative
            }
            ImportPathKind::Unknown => ImportGroup::Unknown,
        }
    }
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

/// Classification of import path types for grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Unknown,
}

impl ImportPathKind {
    fn new(source: &str) -> Self {
        if source.starts_with("node:")
            || source.starts_with("bun:")
            || NODE_BUILTINS.contains(source)
        {
            return Self::Builtin;
        }

        // Check for relative imports
        if source == "." || source == ".." {
            return Self::Index;
        }

        if source.starts_with("./") || source.starts_with("../") {
            if source.ends_with('/') {
                return Self::Index;
            }
            if source.starts_with("../") {
                return Self::Parent;
            }
            return Self::Sibling;
        }

        // TODO: This can be changed via `options.internalPattern`
        if source.starts_with('~') || source.starts_with('@') {
            return Self::Internal;
        }

        Self::External
    }
}
