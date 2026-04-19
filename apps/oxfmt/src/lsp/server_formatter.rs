use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use tower_lsp_server::ls_types::{Pattern, Position, Range, ServerCapabilities, TextEdit, Uri};
use tracing::{debug, error, warn};

use oxc_data_structures::rope::{Rope, get_line_column};
use oxc_language_server::{
    Capabilities, ConcurrentHashMap, LanguageId, TextDocument, Tool, ToolBuilder,
    ToolRestartChanges,
};

use crate::core::{
    ConfigResolver, ExternalFormatter, FormatFileStrategy, FormatResult, JsConfigLoaderCb,
    SourceFormatter, all_config_file_names, has_config_in_directory, resolve_editorconfig_path,
    utils,
};
use crate::lsp::create_fake_file_path_from_language_id;
use crate::lsp::options::FormatOptions as LSPFormatOptions;

pub struct ServerFormatterBuilder {
    js_config_loader: JsConfigLoaderCb,
    external_formatter: ExternalFormatter,
}

impl ServerFormatterBuilder {
    pub fn new(js_config_loader: JsConfigLoaderCb, external_formatter: ExternalFormatter) -> Self {
        Self { js_config_loader, external_formatter }
    }

    /// Create a dummy `ServerFormatterBuilder` for testing.
    #[cfg(test)]
    pub fn dummy() -> Self {
        Self {
            js_config_loader: std::sync::Arc::new(|_| {
                Err("JS config not supported in tests".to_string())
            }),
            external_formatter: ExternalFormatter::dummy(),
        }
    }

    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn build(&self, root_uri: &Uri, options: serde_json::Value) -> ServerFormatter {
        let options = deserialize_lsp_options(options);

        let root_path = root_uri.to_file_path().unwrap();
        debug!("root_path = {:?}", root_path.display());

        // Resolve workspace-level concerns only here.
        // Per-file config resolution is deferred to format time.

        let prettierignore_glob = match Self::create_prettierignore_glob(&root_path) {
            Ok(glob) => Some(glob),
            Err(err) => {
                warn!("Failed to create gitignore globs: {err}, proceeding without ignore globs");
                None
            }
        };

        // If `configPath` is explicitly set, load it eagerly as the single config for all files.
        let explicit_config_path = options.config_path.filter(|s| !s.is_empty()).map(PathBuf::from);

        let num_of_threads = 1; // Single threaded for LSP
        // Use `block_in_place()` to avoid nested async runtime access
        match tokio::task::block_in_place(|| self.external_formatter.init(num_of_threads)) {
            // TODO: Plugins support
            Ok(_) => {}
            Err(err) => {
                error!("Failed to setup external formatter.\n{err}\n");
            }
        }
        let source_formatter = SourceFormatter::new(num_of_threads)
            .with_external_formatter(Some(self.external_formatter.clone()));

        ServerFormatter::new(
            root_path.to_path_buf(),
            source_formatter,
            JsConfigLoaderCb::clone(&self.js_config_loader),
            resolve_editorconfig_path(&root_path),
            prettierignore_glob,
            explicit_config_path,
        )
    }
}

impl ToolBuilder for ServerFormatterBuilder {
    fn server_capabilities(
        &self,
        capabilities: &mut ServerCapabilities,
        _backend_capabilities: &mut Capabilities,
    ) {
        capabilities.document_formatting_provider =
            Some(tower_lsp_server::ls_types::OneOf::Left(true));
    }

    fn build_boxed(&self, root_uri: &Uri, options: serde_json::Value) -> Box<dyn Tool> {
        Box::new(self.build(root_uri, options))
    }
}

impl ServerFormatterBuilder {
    /// Create `.prettierignore` glob (workspace-level only).
    fn create_prettierignore_glob(root_path: &Path) -> Result<Gitignore, String> {
        let mut builder = GitignoreBuilder::new(root_path);
        for ignore_path in &load_ignore_paths(root_path) {
            if builder.add(ignore_path).is_some() {
                return Err(format!("Failed to add ignore file: {}", ignore_path.display()));
            }
        }
        builder.build().map_err(|_| "Failed to build ignore globs".to_string())
    }
}

// ---

/// Cache key for per-directory config resolution.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConfigScope {
    /// Config found within workspace root at this directory.
    Dir(PathBuf),
    /// No config file found. Discovers from workspace root upward.
    Fallback,
    /// Explicit `fmt.configPath` from LSP settings.
    Explicit,
}

pub struct ServerFormatter {
    root_path: PathBuf,
    source_formatter: SourceFormatter,
    config_cache: ConcurrentHashMap<ConfigScope, ConfigResolver>,
    js_config_loader: JsConfigLoaderCb,
    editorconfig_path: Option<PathBuf>,
    /// `.prettierignore` glob (workspace-level, shared across all scopes).
    prettierignore_glob: Option<Gitignore>,
    /// Explicit `fmt.configPath` from LSP settings. When set, disables nested
    /// config discovery; all files use this single config.
    explicit_config_path: Option<PathBuf>,
}

impl Tool for ServerFormatter {
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    fn handle_configuration_change(
        &self,
        builder: &dyn ToolBuilder,
        root_uri: &Uri,
        old_options_json: &serde_json::Value,
        new_options_json: serde_json::Value,
    ) -> ToolRestartChanges {
        let old_option = deserialize_lsp_options(old_options_json.clone());
        let new_option = deserialize_lsp_options(new_options_json.clone());

        if old_option == new_option {
            return ToolRestartChanges { tool: None, watch_patterns: None };
        }

        builder.shutdown(root_uri);
        let new_formatter = builder.build_boxed(root_uri, new_options_json.clone());
        let watch_patterns = new_formatter.get_watcher_patterns(new_options_json);
        ToolRestartChanges { tool: Some(new_formatter), watch_patterns: Some(watch_patterns) }
    }

    fn get_watcher_patterns(&self, options: serde_json::Value) -> Vec<Pattern> {
        let options = deserialize_lsp_options(options);

        let mut patterns: Vec<Pattern> =
            if let Some(config_path) = options.config_path.as_ref().filter(|s| !s.is_empty()) {
                vec![config_path.clone()]
            } else {
                // Watch for config files in all subdirectories (nested config support)
                all_config_file_names().map(|name| format!("**/{name}")).collect()
            };

        patterns.push(".editorconfig".to_string());
        patterns
    }

    fn handle_watched_file_change(
        &self,
        _builder: &dyn ToolBuilder,
        changed_uri: &Uri,
        _root_uri: &Uri,
        _options: serde_json::Value,
    ) -> ToolRestartChanges {
        // Evict the affected cache entry instead of full restart
        if let Some(changed_path) = changed_uri.to_file_path()
            && let Some(changed_dir) = changed_path.parent()
        {
            let cache = self.config_cache.pin();

            // .editorconfig affects all scopes — clear everything
            if changed_path.file_name().and_then(|f| f.to_str()) == Some(".editorconfig") {
                cache.clear();
            } else {
                cache.remove(&ConfigScope::Dir(changed_dir.to_path_buf()));
            }
        }

        ToolRestartChanges { tool: None, watch_patterns: None }
    }

    fn run_format(&self, document: &TextDocument) -> Result<Vec<TextEdit>, String> {
        let file_content;
        let (result, source_text) = if document.uri.scheme().as_str() == "file" {
            let Some(path) = document.uri.to_file_path() else {
                return Err("Invalid file URI".to_string());
            };

            let source_text = if let Some(c) = document.text.as_deref() {
                c
            } else {
                file_content = utils::read_to_string(&path)
                    .map_err(|e| format!("Failed to read file: {e}"))?;
                &file_content
            };

            let Some(result) = self.format_file(&path, source_text) else {
                return Ok(vec![]); // No formatting for this file (unsupported or ignored)
            };

            (result, source_text)
        } else {
            let source_text = document
                .text
                .as_deref()
                .ok_or_else(|| "In-memory formatting requires content".to_string())?;

            let Some(result) =
                self.format_in_memory(document.uri, source_text, &document.language_id)
            else {
                return Ok(vec![]); // currently not supported
            };
            (result, source_text)
        };

        // Handle result
        match result {
            FormatResult::Success { code, is_changed } => {
                if !is_changed {
                    return Ok(vec![]);
                }

                let (start, end, replacement) = compute_minimal_text_edit(source_text, &code);
                let rope = Rope::from(source_text);
                let (start_line, start_character) = get_line_column(&rope, start, source_text);
                let (end_line, end_character) = get_line_column(&rope, end, source_text);

                Ok(vec![TextEdit::new(
                    Range::new(
                        Position::new(start_line, start_character),
                        Position::new(end_line, end_character),
                    ),
                    replacement.to_string(),
                )])
            }
            FormatResult::Error(_) => {
                // Errors should not be returned to the user.
                // The user probably wanted to format while typing incomplete code.
                Ok(Vec::new())
            }
        }
    }
}

impl ServerFormatter {
    pub fn new(
        root_path: PathBuf,
        source_formatter: SourceFormatter,
        js_config_loader: JsConfigLoaderCb,
        editorconfig_path: Option<PathBuf>,
        prettierignore_glob: Option<Gitignore>,
        explicit_config_path: Option<PathBuf>,
    ) -> Self {
        Self {
            root_path,
            source_formatter,
            config_cache: ConcurrentHashMap::default(),
            js_config_loader,
            editorconfig_path,
            prettierignore_glob,
            explicit_config_path,
        }
    }

    /// Determine which config scope applies for a given file path.
    ///
    /// If an explicit config path is set, always returns `Explicit`.
    /// Otherwise, searches upward from the file's parent directory for a config file.
    fn resolve_config_scope(&self, file_path: &Path) -> ConfigScope {
        if self.explicit_config_path.is_some() {
            return ConfigScope::Explicit;
        }

        let Some(start_dir) = file_path.parent() else {
            return ConfigScope::Fallback;
        };

        for dir in start_dir.ancestors() {
            if has_config_in_directory(dir) {
                return ConfigScope::Dir(dir.to_path_buf());
            }
        }

        ConfigScope::Fallback
    }

    /// Load a `ConfigResolver` for the given directory.
    /// Falls back to default config on any error.
    fn load_cached_config(&self, cwd: &Path) -> ConfigResolver {
        let result = ConfigResolver::from_config(
            cwd,
            self.explicit_config_path.as_deref(),
            self.editorconfig_path.as_deref(),
            Some(&self.js_config_loader),
        )
        .and_then(|mut resolver| {
            resolver.build_and_validate()?;
            Ok(resolver)
        });

        result.unwrap_or_else(|err| {
            warn!("Failed to load config at {}: {err}, falling back to default", cwd.display());
            let mut resolver = ConfigResolver::from_json_config(Path::new("."), None, None)
                .expect("Default ConfigResolver should never fail");
            resolver
                .build_and_validate()
                .expect("Default ConfigResolver validation should never fail");
            resolver
        })
    }

    /// Resolve config and format a file at the given path.
    /// Returns `None` if the file is unsupported or ignored.
    fn resolve_and_format(&self, path: &Path, source_text: &str) -> Option<FormatResult> {
        let Ok(strategy) = FormatFileStrategy::try_from(path.to_path_buf()) else {
            debug!("Unsupported file type for formatting: {}", path.display());
            return None;
        };

        let config_scope = self.resolve_config_scope(path);
        let cache = self.config_cache.pin();
        let cached = cache.get_or_insert_with(config_scope.clone(), || {
            let cwd = match &config_scope {
                ConfigScope::Dir(dir) => dir.as_path(),
                ConfigScope::Fallback | ConfigScope::Explicit => &self.root_path,
            };
            self.load_cached_config(cwd)
        });

        if cached.is_path_ignored(path, path.is_dir()) {
            debug!("File is ignored by config ignorePatterns: {}", path.display());
            return None;
        }

        let resolved_options = cached.resolve(&strategy);
        debug!("resolved_options = {resolved_options:?}");

        Some(tokio::task::block_in_place(|| {
            self.source_formatter.format(&strategy, source_text, resolved_options)
        }))
    }

    fn format_file(&self, path: &Path, source_text: &str) -> Option<FormatResult> {
        if self.prettierignore_glob.as_ref().is_some_and(|glob| {
            path.starts_with(glob.path())
                && glob.matched_path_or_any_parents(path, path.is_dir()).is_ignore()
        }) {
            debug!("File is ignored by .prettierignore: {}", path.display());
            return None;
        }
        self.resolve_and_format(path, source_text)
    }

    fn format_in_memory(
        &self,
        uri: &Uri,
        source_text: &str,
        language_id: &LanguageId,
    ) -> Option<FormatResult> {
        let Some(path) = create_fake_file_path_from_language_id(language_id, &self.root_path, uri)
        else {
            debug!("Unsupported language id for in-memory formatting: {language_id:?}");
            return None;
        };
        self.resolve_and_format(&path, source_text)
    }
}

// ---

/// Deserialize `LSPFormatOptions` from JSON, falling back to defaults on failure.
fn deserialize_lsp_options(value: serde_json::Value) -> LSPFormatOptions {
    match serde_json::from_value::<LSPFormatOptions>(value) {
        Ok(opts) => opts,
        Err(err) => {
            warn!(
                "Failed to deserialize LSPFormatOptions from JSON: {err}, falling back to default options"
            );
            LSPFormatOptions::default()
        }
    }
}

/// Returns the minimal text edit (start, end, replacement) to transform `source_text` into `formatted_text`
#[expect(clippy::cast_possible_truncation)]
fn compute_minimal_text_edit<'a>(
    source_text: &str,
    formatted_text: &'a str,
) -> (u32, u32, &'a str) {
    debug_assert!(source_text != formatted_text);

    // Find common prefix (byte offset)
    let mut prefix_byte = 0;
    for (a, b) in source_text.chars().zip(formatted_text.chars()) {
        if a == b {
            prefix_byte += a.len_utf8();
        } else {
            break;
        }
    }

    // Find common suffix (byte offset from end)
    let mut suffix_byte = 0;
    let src_bytes = source_text.as_bytes();
    let fmt_bytes = formatted_text.as_bytes();
    let src_len = src_bytes.len();
    let fmt_len = fmt_bytes.len();

    while suffix_byte < src_len - prefix_byte
        && suffix_byte < fmt_len - prefix_byte
        && src_bytes[src_len - 1 - suffix_byte] == fmt_bytes[fmt_len - 1 - suffix_byte]
    {
        suffix_byte += 1;
    }

    let start = prefix_byte as u32;
    let end = (src_len - suffix_byte) as u32;
    let replacement_start = prefix_byte;
    let replacement_end = fmt_len - suffix_byte;
    let replacement = &formatted_text[replacement_start..replacement_end];

    (start, end, replacement)
}

// Almost the same as `cli::walk::load_ignore_paths`, but does not handle custom ignore files.
//
// NOTE: `.gitignore` is intentionally NOT included here.
// In LSP, every file is explicitly opened by the user (like directly specifying a file in CLI),
// so `.gitignore` should not prevent formatting.
// Only formatter-specific ignore files apply.
fn load_ignore_paths(cwd: &Path) -> Vec<PathBuf> {
    let path = cwd.join(".prettierignore");
    if path.exists() { vec![path] } else { vec![] }
}

// ---

#[cfg(test)]
mod tests_builder {
    use crate::lsp::server_formatter::ServerFormatterBuilder;
    use oxc_language_server::{Capabilities, ToolBuilder};

    #[test]
    fn test_server_capabilities() {
        use tower_lsp_server::ls_types::{OneOf, ServerCapabilities};

        let builder = ServerFormatterBuilder::dummy();
        let mut capabilities = ServerCapabilities::default();

        builder.server_capabilities(&mut capabilities, &mut Capabilities::default());

        assert_eq!(capabilities.document_formatting_provider, Some(OneOf::Left(true)));
    }
}

#[cfg(test)]
mod tests {
    use super::compute_minimal_text_edit;

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_no_change() {
        let src = "abc";
        let formatted = "abc";
        compute_minimal_text_edit(src, formatted);
    }

    #[test]
    fn test_single_char_change() {
        let src = "abc";
        let formatted = "axc";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        // Only 'b' replaced by 'x'
        assert_eq!((start, end, replacement), (1, 2, "x"));
    }

    #[test]
    fn test_insert_char() {
        let src = "abc";
        let formatted = "abxc";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        // Insert 'x' after 'b'
        assert_eq!((start, end, replacement), (2, 2, "x"));
    }

    #[test]
    fn test_delete_char() {
        let src = "abc";
        let formatted = "ac";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        // Delete 'b'
        assert_eq!((start, end, replacement), (1, 2, ""));
    }

    #[test]
    fn test_replace_multiple_chars() {
        let src = "abcdef";
        let formatted = "abXYef";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        // Replace "cd" with "XY"
        assert_eq!((start, end, replacement), (2, 4, "XY"));
    }

    #[test]
    fn test_replace_multiple_chars_between_similars_complex() {
        let src = "aYabYb";
        let formatted = "aXabXb";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        assert_eq!((start, end, replacement), (1, 5, "XabX"));
    }

    #[test]
    fn test_unicode() {
        let src = "a😀b";
        let formatted = "a😃b";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);
        // Replace 😀 with 😃
        assert_eq!((start, end, replacement), (1, 5, "😃"));
    }

    #[test]
    fn test_append() {
        let src = "a".repeat(100);
        let mut formatted = src.clone();
        formatted.push('b'); // Add a character at the end

        let (start, end, replacement) = compute_minimal_text_edit(&src, &formatted);
        assert_eq!((start, end, replacement), (100, 100, "b"));
    }

    #[test]
    fn test_prepend() {
        let src = "a".repeat(100);
        let mut formatted = String::from("b");
        formatted.push_str(&src); // Add a character at the start

        let (start, end, replacement) = compute_minimal_text_edit(&src, &formatted);
        assert_eq!((start, end, replacement), (0, 0, "b"));
    }
}
