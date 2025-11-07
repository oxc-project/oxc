//! Store for custom parsers loaded from JavaScript.

use std::path::{Path, PathBuf};

use rustc_hash::FxHashMap;

/// Store for custom parsers.
///
/// Tracks which parsers have been loaded and their paths.
#[derive(Debug, Default)]
pub struct ExternalParserStore {
    /// Set of parser paths that have been loaded.
    registered_parser_paths: FxHashMap<PathBuf, String>,
}

impl ExternalParserStore {
    /// Create a new empty parser store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a parser has been registered.
    pub fn is_parser_registered(&self, parser_path: &Path) -> bool {
        self.registered_parser_paths.contains_key(parser_path)
    }

    /// Register a parser.
    pub fn register_parser(&mut self, parser_path: PathBuf, parser_name: String) {
        self.registered_parser_paths.insert(parser_path, parser_name);
    }

    /// Get the name of a registered parser.
    pub fn get_parser_name(&self, parser_path: &Path) -> Option<&String> {
        self.registered_parser_paths.get(parser_path)
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.registered_parser_paths.is_empty()
    }

    /// Get the number of registered parsers.
    pub fn len(&self) -> usize {
        self.registered_parser_paths.len()
    }
}

