use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use tower_lsp_server::ls_types::{Pattern, Position, Range, ServerCapabilities, TextEdit, Uri};
use tracing::{debug, error, warn};

use oxc_data_structures::rope::{Rope, get_line_column};
use oxc_language_server::{
    Capabilities, LanguageId, TextDocument, Tool, ToolBuilder, ToolRestartChanges,
};

use crate::core::{
    ConfigResolver, ExternalFormatter, FormatResult, JsConfigLoaderCb, NestedConfigCtx,
    ResolveOutcome, SourceFormatter, classify_file_kind, config_discovery,
    resolve_editorconfig_path, resolve_file_scope_config, utils,
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

/// Per-rebuild config snapshot.
///
/// Held behind `RwLock<Arc<_>>` on [`ServerFormatter`] so a watch event can
/// swap a fresh state in without blocking concurrent format requests:
/// in-flight readers clone the `Arc` and continue using the old snapshot,
/// while subsequent reads see the new one.
struct FormatterState {
    /// Workspace-root resolver.
    /// Used as the fallback when no nested config matches.
    root_resolver: Arc<ConfigResolver>,
    /// Lazy nested-config probe cache.
    /// Each ancestor directory is loaded at most once for the lifetime of this state.
    nested_ctx: NestedConfigCtx,
}

pub struct ServerFormatter {
    root_path: PathBuf,
    source_formatter: SourceFormatter,
    js_config_loader: JsConfigLoaderCb,
    /// `.prettierignore` glob (workspace-level, shared across all scopes).
    prettierignore_glob: Option<Gitignore>,
    /// Explicit `fmt.configPath` from LSP settings. When set, disables nested
    /// config discovery; all files use this single config.
    explicit_config_path: Option<PathBuf>,
    /// Current config snapshot. Swapped wholesale on watched-file changes.
    state: RwLock<Arc<FormatterState>>,
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
                config_discovery()
                    .config_file_names()
                    .into_iter()
                    .map(|name| format!("**/{name}"))
                    .collect()
            };

        patterns.push(".editorconfig".to_string());
        patterns
    }

    fn handle_watched_file_change(
        &self,
        _builder: &dyn ToolBuilder,
        _changed_uri: &Uri,
        _root_uri: &Uri,
        _options: serde_json::Value,
    ) -> ToolRestartChanges {
        // Rebuild the snapshot wholesale.
        //
        // `NestedConfigCtx` has no per-entry invalidation API by design (its caches are walk-scoped).
        // Rebuilding is cheap: the ctx itself starts empty (lazy probes),
        // and only the root resolver does eager file IO.
        // The trade-off is over-eviction, a config change in one nested dir also drops cached probes elsewhere.
        // But format requests are sporadic enough that lazy re-population costs nothing observable.
        let new_state = Self::build_state(
            &self.root_path,
            self.explicit_config_path.as_deref(),
            &self.js_config_loader,
        );
        *self.state.write().expect("state rwlock poisoned") = Arc::new(new_state);

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
        prettierignore_glob: Option<Gitignore>,
        explicit_config_path: Option<PathBuf>,
    ) -> Self {
        let state =
            Self::build_state(&root_path, explicit_config_path.as_deref(), &js_config_loader);
        Self {
            root_path,
            source_formatter,
            js_config_loader,
            prettierignore_glob,
            explicit_config_path,
            state: RwLock::new(Arc::new(state)),
        }
    }

    /// Build a fresh [`FormatterState`] from scratch.
    ///
    /// Called once in [`Self::new`] and again on every watched-file change.
    /// `.editorconfig` is re-resolved here so add/remove events are picked up
    /// without a separate code path.
    fn build_state(
        root_path: &Path,
        explicit_config_path: Option<&Path>,
        js_config_loader: &JsConfigLoaderCb,
    ) -> FormatterState {
        let editorconfig_path = resolve_editorconfig_path(root_path);
        let root_resolver = Self::load_root_resolver(
            root_path,
            explicit_config_path,
            editorconfig_path.as_deref(),
            js_config_loader,
        );
        let nested_ctx = NestedConfigCtx::new(
            editorconfig_path.as_deref().map(Arc::from),
            Some(JsConfigLoaderCb::clone(js_config_loader)),
        );
        FormatterState { root_resolver: Arc::new(root_resolver), nested_ctx }
    }

    /// Load the workspace-root resolver,
    /// falling back to the default empty config on any load or validation error.
    ///
    /// LSP must keep editing usable even when the user's config is broken,
    /// so we surface a warning instead of bubbling the error up.
    fn load_root_resolver(
        root_path: &Path,
        explicit_config_path: Option<&Path>,
        editorconfig_path: Option<&Path>,
        js_config_loader: &JsConfigLoaderCb,
    ) -> ConfigResolver {
        let result = ConfigResolver::from_config(
            root_path,
            explicit_config_path,
            editorconfig_path,
            Some(js_config_loader),
        )
        .and_then(|mut resolver| {
            resolver.build_and_validate()?;
            Ok(resolver)
        });

        result.unwrap_or_else(|err| {
            warn!(
                "Failed to load config at {}: {err}, falling back to default",
                root_path.display()
            );
            let mut resolver = ConfigResolver::from_json_config(None, None)
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
        // Snapshot the current state.
        // In-flight reads survive a concurrent rebuild because the old `Arc` keeps the previous snapshot alive.
        let state = Arc::clone(&self.state.read().expect("state rwlock poisoned"));

        // Explicit config path applies uniformly to every file;
        // passing `None` tells `resolve_file_scope_config` to bypass nested probing.
        let nested_ctx = self.explicit_config_path.is_none().then_some(&state.nested_ctx);
        let resolver = match resolve_file_scope_config(path, &state.root_resolver, nested_ctx) {
            Ok(r) => r,
            Err(err) => {
                warn!(
                    "Failed to resolve nested config for {}: {err}, falling back to root",
                    path.display()
                );
                Arc::clone(&state.root_resolver)
            }
        };

        if resolver.is_path_ignored(path, path.is_dir()) {
            debug!("File is ignored by config ignorePatterns: {}", path.display());
            return None;
        }

        let Some(kind) = classify_file_kind(Arc::from(path)) else {
            debug!("Unsupported file type for formatting: {}", path.display());
            return None;
        };
        let strategy = match resolver.resolve(kind) {
            Ok(ResolveOutcome::Format(strategy)) => strategy,
            Ok(ResolveOutcome::MissingPlugin(plugin)) => {
                warn!(
                    "Skipping `.{plugin}`: `{plugin}` plugin is not enabled in resolved config: {}",
                    path.display()
                );
                return None;
            }
            Err(err) => {
                debug!("Config resolve error for {}: {err}", path.display());
                return None;
            }
        };
        debug!("strategy = {strategy:?}");

        Some(tokio::task::block_in_place(|| self.source_formatter.format(source_text, strategy)))
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
