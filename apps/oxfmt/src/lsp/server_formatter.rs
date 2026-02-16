use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use tower_lsp_server::ls_types::{Pattern, Position, Range, ServerCapabilities, TextEdit, Uri};
use tracing::{debug, error, warn};

use oxc_data_structures::rope::{Rope, get_line_column};
use oxc_language_server::{Capabilities, Tool, ToolBuilder, ToolRestartChanges};

use crate::core::{
    ConfigResolver, ExternalFormatter, FormatFileStrategy, FormatResult, SourceFormatter,
    resolve_editorconfig_path, resolve_oxfmtrc_path, utils,
};
use crate::lsp::{FORMAT_CONFIG_FILES, options::FormatOptions as LSPFormatOptions};

pub struct ServerFormatterBuilder {
    external_formatter: ExternalFormatter,
}

impl ServerFormatterBuilder {
    pub fn new(external_formatter: ExternalFormatter) -> Self {
        Self { external_formatter }
    }

    /// Create a dummy `ServerFormatterBuilder` for testing.
    #[cfg(test)]
    pub fn dummy() -> Self {
        Self { external_formatter: ExternalFormatter::dummy() }
    }

    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn build(&self, root_uri: &Uri, options: serde_json::Value) -> ServerFormatter {
        let options = match serde_json::from_value::<LSPFormatOptions>(options) {
            Ok(opts) => opts,
            Err(err) => {
                warn!(
                    "Failed to deserialize LSPFormatOptions from JSON: {err}, falling back to default options"
                );
                LSPFormatOptions::default()
            }
        };

        let root_path = root_uri.to_file_path().unwrap();
        debug!("root_path = {:?}", root_path.display());

        // Build `ConfigResolver` from config paths
        let (config_resolver, ignore_patterns) =
            match Self::build_config_resolver(&root_path, options.config_path.as_ref()) {
                Ok((resolver, patterns)) => (resolver, patterns),
                Err(err) => {
                    warn!("Failed to build config resolver: {err}, falling back to default config");
                    Self::default_config_resolver()
                }
            };

        let gitignore_glob = match Self::create_ignore_globs(&root_path, &ignore_patterns) {
            Ok(glob) => Some(glob),
            Err(err) => {
                warn!("Failed to create gitignore globs: {err}, proceeding without ignore globs");
                None
            }
        };

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

        ServerFormatter::new(source_formatter, config_resolver, gitignore_glob)
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
    /// Build a `ConfigResolver` from config paths.
    /// Returns the resolver and ignore patterns.
    ///
    /// # Errors
    /// Returns error if config file parsing fails.
    fn build_config_resolver(
        root_path: &Path,
        config_path: Option<&String>,
    ) -> Result<(ConfigResolver, Vec<String>), String> {
        let oxfmtrc_path =
            resolve_oxfmtrc_path(root_path, config_path.filter(|s| !s.is_empty()).map(Path::new));
        let editorconfig_path = resolve_editorconfig_path(root_path);

        let mut resolver = ConfigResolver::from_config_paths(
            root_path,
            oxfmtrc_path.as_deref(),
            editorconfig_path.as_deref(),
        )?;

        // Validate config and cache options, returns ignore patterns
        let ignore_patterns = resolver.build_and_validate()?;

        Ok((resolver, ignore_patterns))
    }

    /// Create a default `ConfigResolver` when config loading fails.
    fn default_config_resolver() -> (ConfigResolver, Vec<String>) {
        let mut resolver = ConfigResolver::from_config_paths(Path::new("."), None, None)
            .expect("Default ConfigResolver should never fail");
        let ignore_patterns = resolver
            .build_and_validate()
            .expect("Default ConfigResolver validation should never fail");
        (resolver, ignore_patterns)
    }

    fn create_ignore_globs(
        root_path: &Path,
        ignore_patterns: &[String],
    ) -> Result<Gitignore, String> {
        let mut builder = GitignoreBuilder::new(root_path);
        for ignore_path in &load_ignore_paths(root_path) {
            if builder.add(ignore_path).is_some() {
                return Err(format!("Failed to add ignore file: {}", ignore_path.display()));
            }
        }
        for pattern in ignore_patterns {
            builder
                .add_line(None, pattern)
                .map_err(|e| format!("Invalid ignore pattern: {pattern}: {e}"))?;
        }

        builder.build().map_err(|_| "Failed to build ignore globs".to_string())
    }
}

// ---

pub struct ServerFormatter {
    source_formatter: SourceFormatter,
    config_resolver: ConfigResolver,
    gitignore_glob: Option<Gitignore>,
}

impl Tool for ServerFormatter {
    fn name(&self) -> &'static str {
        "formatter"
    }

    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    fn handle_configuration_change(
        &self,
        builder: &dyn ToolBuilder,
        root_uri: &Uri,
        old_options_json: &serde_json::Value,
        new_options_json: serde_json::Value,
    ) -> ToolRestartChanges {
        let old_option = match serde_json::from_value::<LSPFormatOptions>(old_options_json.clone())
        {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPFormatOptions from JSON: {e}. Falling back to default options."
                );
                LSPFormatOptions::default()
            }
        };

        let new_option = match serde_json::from_value::<LSPFormatOptions>(new_options_json.clone())
        {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPFormatOptions from JSON: {e}. Falling back to default options."
                );
                LSPFormatOptions::default()
            }
        };

        if old_option == new_option {
            return ToolRestartChanges { tool: None, watch_patterns: None };
        }

        builder.shutdown(root_uri);
        let new_formatter = builder.build_boxed(root_uri, new_options_json.clone());
        let watch_patterns = new_formatter.get_watcher_patterns(new_options_json);
        ToolRestartChanges { tool: Some(new_formatter), watch_patterns: Some(watch_patterns) }
    }

    fn get_watcher_patterns(&self, options: serde_json::Value) -> Vec<Pattern> {
        let options = match serde_json::from_value::<LSPFormatOptions>(options) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPFormatOptions from JSON: {e}. Falling back to default options."
                );
                LSPFormatOptions::default()
            }
        };

        let mut patterns: Vec<Pattern> =
            if let Some(config_path) = options.config_path.as_ref().filter(|s| !s.is_empty()) {
                vec![config_path.clone()]
            } else {
                FORMAT_CONFIG_FILES.iter().map(|file| (*file).to_string()).collect()
            };

        patterns.push(".editorconfig".to_string());
        patterns
    }

    fn handle_watched_file_change(
        &self,
        builder: &dyn ToolBuilder,
        _changed_uri: &Uri,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> ToolRestartChanges {
        // TODO: Check if the changed file is actually a config file
        builder.shutdown(root_uri);
        let new_formatter = builder.build_boxed(root_uri, options);

        ToolRestartChanges {
            tool: Some(new_formatter),
            // TODO: update watch patterns if config_path changed
            watch_patterns: None,
        }
    }

    fn run_format(&self, uri: &Uri, content: Option<&str>) -> Result<Vec<TextEdit>, String> {
        let Some(path) = uri.to_file_path() else { return Err("Invalid file URI".to_string()) };

        if self.is_ignored(&path) {
            debug!("File is ignored: {}", path.display());
            return Ok(Vec::new());
        }

        // Determine format strategy from file path (supports JS/TS, JSON, YAML, CSS, etc.)
        let Ok(strategy) = FormatFileStrategy::try_from(path.to_path_buf()) else {
            debug!("Unsupported file type for formatting: {}", path.display());
            return Ok(Vec::new());
        };
        let source_text = match content {
            Some(c) => c,
            None => {
                &utils::read_to_string(&path).map_err(|e| format!("Failed to read file: {e}"))?
            }
        };

        // Resolve options for this file
        let resolved_options = self.config_resolver.resolve(&strategy);
        debug!("resolved_options = {resolved_options:?}");

        let result = tokio::task::block_in_place(|| {
            self.source_formatter.format(&strategy, source_text, resolved_options)
        });

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
        source_formatter: SourceFormatter,
        config_resolver: ConfigResolver,
        gitignore_glob: Option<Gitignore>,
    ) -> Self {
        Self { source_formatter, config_resolver, gitignore_glob }
    }

    fn is_ignored(&self, path: &Path) -> bool {
        if let Some(glob) = &self.gitignore_glob {
            if !path.starts_with(glob.path()) {
                return false;
            }

            glob.matched_path_or_any_parents(path, path.is_dir()).is_ignore()
        } else {
            false
        }
    }
}

// ---

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
mod test_watchers {
    // formatter file watcher-system does not depend on the actual file system,
    // so we can use a fake directory for testing.
    const FAKE_DIR: &str = "fixtures/formatter/watchers";

    mod handle_configuration_change {
        use crate::lsp::{server_formatter::test_watchers::FAKE_DIR, tester::Tester};
        use oxc_language_server::ToolRestartChanges;
        use serde_json::json;

        #[test]
        fn test_no_change() {
            let ToolRestartChanges { watch_patterns, .. } =
                Tester::new(FAKE_DIR, json!({})).handle_configuration_change(json!({}));

            assert!(watch_patterns.is_none());
        }

        #[test]
        fn test_formatter_custom_config_path() {
            let ToolRestartChanges { watch_patterns, .. } = Tester::new(FAKE_DIR, json!({}))
                .handle_configuration_change(json!({
                    "fmt.configPath": "configs/formatter.json"
                }));

            assert!(watch_patterns.is_some());
            assert_eq!(watch_patterns.as_ref().unwrap().len(), 2);
            assert_eq!(watch_patterns.as_ref().unwrap()[0], "configs/formatter.json");
            assert_eq!(watch_patterns.as_ref().unwrap()[1], ".editorconfig");
        }
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
