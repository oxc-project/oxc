use std::path::{Path, PathBuf};

use oxc_index::{IndexVec, define_index_type};
use rustc_hash::FxHashSet;

use crate::{ExternalLinter, LoadParserResult, config::GlobSet};

define_index_type! {
    pub struct ExternalParserId = u32;
}

/// Store for external JS parsers.
///
/// Manages loaded parsers and their file pattern associations.
/// When a file is being linted, this store can be queried to determine
/// if a custom parser should be used based on the file path.
#[derive(Debug, Default)]
pub struct ExternalParserStore {
    /// Paths of parser modules that have been registered
    registered_parser_paths: FxHashSet<PathBuf>,
    /// All registered parsers
    parsers: IndexVec<ExternalParserId, ExternalParser>,
}

#[derive(Debug)]
struct ExternalParser {
    /// The parser's unique ID from JS side
    parser_id: u32,
    /// Whether the parser implements `parseForESLint`
    has_parse_for_eslint: bool,
    /// File patterns this parser handles
    patterns: GlobSet,
    /// Parser options JSON
    parser_options_json: String,
}

impl ExternalParserStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if no parsers have been loaded.
    pub fn is_empty(&self) -> bool {
        self.parsers.is_empty()
    }

    /// Check if a parser module at the given path is already registered.
    pub fn is_parser_registered(&self, parser_path: &Path) -> bool {
        self.registered_parser_paths.contains(parser_path)
    }

    /// Register a parser with its patterns.
    ///
    /// # Arguments
    /// * `parser_path` - Path to the parser module
    /// * `patterns` - Glob patterns for files this parser handles
    /// * `parser_options_json` - Parser options as JSON string
    /// * `result` - Result from loading the parser via callback
    ///
    /// # Panics
    /// Panics if parser at `parser_path` is already registered.
    pub fn register_parser(
        &mut self,
        parser_path: PathBuf,
        patterns: Vec<String>,
        parser_options_json: String,
        result: &LoadParserResult,
    ) {
        let newly_inserted = self.registered_parser_paths.insert(parser_path);
        assert!(newly_inserted, "register_parser: parser already registered");

        self.parsers.push(ExternalParser {
            parser_id: result.parser_id,
            has_parse_for_eslint: result.has_parse_for_eslint,
            patterns: GlobSet::new(patterns),
            parser_options_json,
        });
    }

    /// Find a parser that matches the given file path.
    ///
    /// Returns the parser's JS-side ID, options JSON, and whether it has parseForESLint.
    ///
    /// Note: Parsers are matched in the order they were configured. If multiple parsers
    /// could match the same file, the first one wins.
    ///
    /// # Arguments
    /// * `path` - The absolute path to the file being linted
    /// * `config_path` - Optional path to the config file; if provided, `path` will be made
    ///   relative to the config file's parent directory for pattern matching
    pub fn find_parser_for_path(
        &self,
        path: &Path,
        config_path: Option<&Path>,
    ) -> Option<(u32, &str, bool)> {
        // Make the path relative to the config file's parent directory if config_path is provided
        // This matches how override patterns work in apply_overrides
        let relative_path = config_path
            .and_then(|cp| cp.parent())
            .map_or(path, |parent| path.strip_prefix(parent).unwrap_or(path));

        // Use lossy conversion for non-UTF-8 paths (rare but possible on some systems)
        let path_str = relative_path.to_string_lossy();

        // Check each parser's patterns (first match wins)
        for parser in &self.parsers {
            if parser.patterns.is_match(&path_str) {
                return Some((
                    parser.parser_id,
                    &parser.parser_options_json,
                    parser.has_parse_for_eslint,
                ));
            }
        }

        None
    }

    /// Load all parsers from configuration.
    ///
    /// This should be called during linter setup, after parsing the config.
    ///
    /// # Errors
    /// Returns an error if any parser fails to load.
    pub fn load_parsers_from_config(
        &mut self,
        parsers_config: &[(PathBuf, Vec<String>, Option<serde_json::Value>)],
        external_linter: &ExternalLinter,
    ) -> Result<(), String> {
        let load_parser = external_linter.load_parser.as_ref().ok_or_else(|| {
            "jsParsers configured but parser loading callback not available".to_string()
        })?;

        for (parser_path, patterns, parser_options) in parsers_config {
            if self.is_parser_registered(parser_path) {
                continue;
            }

            let parser_options_json = parser_options.as_ref().map_or_else(
                || "null".to_string(),
                |v| serde_json::to_string(v).unwrap_or_else(|_| "null".to_string()),
            );

            let parser_url = format!("file://{}", parser_path.display());
            let result = load_parser(parser_url, parser_options_json.clone())?;

            self.register_parser(
                parser_path.clone(),
                patterns.clone(),
                parser_options_json,
                &result,
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parser_store_empty() {
        let store = ExternalParserStore::new();
        assert!(store.is_empty());
    }

    #[test]
    fn test_parser_registration() {
        let mut store = ExternalParserStore::new();

        store.register_parser(
            PathBuf::from("/path/to/parser.js"),
            vec!["*.marko".to_string()],
            "null".to_string(),
            &LoadParserResult { parser_id: 1, has_parse_for_eslint: true },
        );

        assert!(!store.is_empty());
        assert!(store.is_parser_registered(Path::new("/path/to/parser.js")));
        assert!(!store.is_parser_registered(Path::new("/other/parser.js")));
    }

    #[test]
    fn test_find_parser_for_path() {
        let mut store = ExternalParserStore::new();

        store.register_parser(
            PathBuf::from("/path/to/marko-parser.js"),
            vec!["*.marko".to_string()],
            r#"{"ecmaVersion": 2020}"#.to_string(),
            &LoadParserResult { parser_id: 42, has_parse_for_eslint: true },
        );

        // Should match .marko files (no config path, uses absolute path matching)
        let result = store.find_parser_for_path(Path::new("/project/component.marko"), None);
        assert!(result.is_some());
        let (parser_id, options, has_parse_for_eslint) = result.unwrap();
        assert_eq!(parser_id, 42);
        assert_eq!(options, r#"{"ecmaVersion": 2020}"#);
        assert!(has_parse_for_eslint);

        // Should not match other files
        let result = store.find_parser_for_path(Path::new("/project/file.js"), None);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_parser_for_path_with_relative_patterns() {
        let mut store = ExternalParserStore::new();

        store.register_parser(
            PathBuf::from("/project/.oxlintrc.json"),
            vec!["src/**/*.custom".to_string()],
            "null".to_string(),
            &LoadParserResult { parser_id: 1, has_parse_for_eslint: true },
        );

        let config_path = Path::new("/project/.oxlintrc.json");

        // Should match when path is made relative to config
        let result = store.find_parser_for_path(
            Path::new("/project/src/components/test.custom"),
            Some(config_path),
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 1);

        // Should not match files outside the pattern
        let result =
            store.find_parser_for_path(Path::new("/project/other/test.custom"), Some(config_path));
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_parsers() {
        let mut store = ExternalParserStore::new();

        store.register_parser(
            PathBuf::from("/path/to/marko-parser.js"),
            vec!["*.marko".to_string()],
            "null".to_string(),
            &LoadParserResult { parser_id: 1, has_parse_for_eslint: true },
        );

        store.register_parser(
            PathBuf::from("/path/to/mdx-parser.js"),
            vec!["*.mdx".to_string()],
            "null".to_string(),
            &LoadParserResult { parser_id: 2, has_parse_for_eslint: false },
        );

        // Check .marko matches first parser
        let result = store.find_parser_for_path(Path::new("test.marko"), None);
        assert_eq!(result.unwrap().0, 1);

        // Check .mdx matches second parser
        let result = store.find_parser_for_path(Path::new("test.mdx"), None);
        assert_eq!(result.unwrap().0, 2);
    }
}
