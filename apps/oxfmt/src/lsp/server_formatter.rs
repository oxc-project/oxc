use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use cow_utils::CowUtils;
use ignore::gitignore::Gitignore;
use tower_lsp_server::gen_lsp_types::{
    DocumentFormattingProvider, MessageType, Pattern, Range, ServerCapabilities, TextEdit, Uri,
};
use tracing::{debug, error, warn};

use oxc_language_server::{
    Capabilities, ClientMessage, LanguageId, TextDocument, Tool, ToolBuildResult, ToolBuilder,
    ToolRestartChanges, offset_to_position, uri_utils::uri_to_file_path,
    utils::normalize_user_config_path_to_watch_pattern,
};

use crate::core::{
    ConfigScopes, ExternalServices, FormatResult, JsConfigLoaderCb, ResolveOutcome,
    SourceFormatter, build_global_ignore_matchers, classify_file, config_discovery, is_ignored,
    resolve_ignore_paths, utils,
};
use crate::lsp::create_fake_file_path_from_language_id;
use crate::lsp::options::FormatOptions as LSPFormatOptions;

pub struct ServerFormatterBuilder {
    js_config_loader: JsConfigLoaderCb,
    external_services: ExternalServices,
}

impl ServerFormatterBuilder {
    pub fn new(js_config_loader: JsConfigLoaderCb, external_services: ExternalServices) -> Self {
        Self { js_config_loader, external_services }
    }

    /// Create a dummy `ServerFormatterBuilder` for testing.
    #[cfg(test)]
    pub fn dummy() -> Self {
        Self {
            js_config_loader: std::sync::Arc::new(|_| {
                Err("JS config not supported in tests".to_string())
            }),
            external_services: ExternalServices::dummy(),
        }
    }

    /// Creates a new `ServerFormatter` instance based on the provided root URI and options.
    /// Returns a tuple containing the `ServerFormatter` instance and optional messages to be sent to the client.
    /// These messages will be used to inform about misconfiguration.
    ///
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn build(
        &self,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> (ServerFormatter, Vec<ClientMessage>) {
        let options = deserialize_lsp_options(options);

        let root_path = uri_to_file_path(root_uri).unwrap();
        debug!("root_path = {:?}", root_path.display());

        // Resolve workspace-level concerns only here.
        // Per-file config resolution is deferred to format time.

        // If `configPath` is explicitly set, load it eagerly as the single config for all files.
        let use_nested_config = options.use_nested_configs();
        let explicit_config_path = options.config_path.as_ref().map(PathBuf::from);

        let num_of_threads = 1; // Single threaded for LSP
        // Use `block_in_place()` to avoid nested async runtime access
        if let Err(err) =
            tokio::task::block_in_place(|| self.external_services.init(num_of_threads))
        {
            error!("Failed to setup external services.\n{err}\n");
        }
        let source_formatter = SourceFormatter::new(num_of_threads)
            .with_external_services(Some(self.external_services.clone()));

        let (formatter, client_message) = ServerFormatter::new(
            root_path.to_path_buf(),
            source_formatter,
            JsConfigLoaderCb::clone(&self.js_config_loader),
            explicit_config_path,
            use_nested_config,
        );

        (formatter, client_message.into_iter().collect())
    }
}

impl ToolBuilder for ServerFormatterBuilder {
    fn server_capabilities(
        &self,
        capabilities: &mut ServerCapabilities,
        _backend_capabilities: &mut Capabilities,
    ) {
        capabilities.document_formatting_provider = Some(DocumentFormattingProvider::Bool(true));
    }

    fn build(&self, root_uri: &Uri, options: serde_json::Value) -> ToolBuildResult {
        let (tool, client_messages) = self.build(root_uri, options);
        ToolBuildResult { tool: Box::new(tool), client_messages }
    }
}

// ---

pub struct ServerFormatter {
    root_path: PathBuf,
    source_formatter: SourceFormatter,
    js_config_loader: JsConfigLoaderCb,
    /// Explicit `fmt.configPath` from LSP settings.
    /// When set, all files use this single config.
    explicit_config_path: Option<PathBuf>,
    /// Whether nested config discovery is active.
    /// Disabled by an explicit `fmt.configPath` or `fmt.disableNestedConfig` in LSP settings.
    use_nested_config: bool,
    /// Current config snapshot. Swapped wholesale on watched-file changes.
    ///
    /// Held behind `RwLock<Arc<_>>` so a watch event can swap a fresh state in
    /// without blocking concurrent format requests:
    /// in-flight readers clone the `Arc` and continue using the old snapshot,
    /// while subsequent reads see the new one.
    state: RwLock<Arc<FormatterState>>,
}

/// Per-rebuild snapshot of everything read from workspace files.
struct FormatterState {
    scopes: ConfigScopes,
    /// `.prettierignore` matchers (workspace-level, shared across all scopes).
    ignore_matchers: Vec<Gitignore>,
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
            return ToolRestartChanges {
                tool: None,
                watch_patterns: None,
                client_messages: Vec::new(),
            };
        }

        builder.shutdown(root_uri);
        let ToolBuildResult { tool, client_messages } =
            builder.build(root_uri, new_options_json.clone());
        let watch_patterns = tool.get_watcher_patterns(new_options_json);
        ToolRestartChanges {
            tool: Some(tool),
            watch_patterns: Some(watch_patterns),
            client_messages,
        }
    }

    fn get_watcher_patterns(&self, options: serde_json::Value) -> Vec<Pattern> {
        let options = deserialize_lsp_options(options);

        let mut patterns: Vec<Pattern> = if let Some(config_path) = options.config_path.as_deref() {
            vec![normalize_user_config_path_to_watch_pattern(config_path)]
        } else {
            // Watch subdirectories too for nested config support;
            // with `disableNestedConfig`, only the workspace-root config is used
            let prefix = if options.use_nested_configs() { "**/" } else { "" };
            config_discovery()
                .config_file_names()
                .into_iter()
                .map(|name| format!("{prefix}{name}"))
                .collect()
        };

        patterns.push(".editorconfig".to_string());
        patterns.push(".prettierignore".to_string());
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
        // `NestedConfigCtx` has no per-entry invalidation API by design (its caches live as long as the `ConfigScopes`).
        // Rebuilding is cheap: the ctx itself starts empty (lazy probes),
        // and only the root resolver, `.editorconfig` and `.prettierignore` do eager file IO.
        // The trade-off is over-eviction, a config change in one nested dir also drops cached probes elsewhere.
        // But format requests are sporadic enough that lazy re-population costs nothing observable.
        let (new_state, client_message) = Self::build_state(
            &self.root_path,
            self.explicit_config_path.as_deref(),
            self.use_nested_config,
            &self.js_config_loader,
        );
        *self.state.write().expect("state rwlock poisoned") = Arc::new(new_state);

        ToolRestartChanges {
            tool: None,
            watch_patterns: None,
            client_messages: client_message.into_iter().collect(),
        }
    }

    fn run_format(&self, document: TextDocument) -> Result<Vec<TextEdit>, String> {
        let file_content;
        let (result, source_text) = if document.uri.scheme().as_str() == "file" {
            let Some(path) = uri_to_file_path(document.uri) else {
                return Err("Invalid file URI".to_string());
            };

            let source_text = if let Some(c) = document.text.as_deref() {
                c
            } else {
                file_content = utils::read_to_string(&path)
                    .map_err(|e| format!("Failed to read file: {e}"))?;
                &file_content
            };

            let Some(result) = self.format_file(&path, source_text)? else {
                return Ok(vec![]); // No formatting for this file (unsupported or ignored)
            };

            (result, source_text)
        } else {
            let source_text = document
                .text
                .as_deref()
                .ok_or_else(|| "In-memory formatting requires content".to_string())?;

            let Some(result) =
                self.format_in_memory(document.uri, source_text, &document.language_id)?
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
                let start_position = offset_to_position(source_text, start);
                let end_position = offset_to_position(source_text, end);

                Ok(vec![TextEdit::new(
                    Range::new(start_position, end_position),
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
        explicit_config_path: Option<PathBuf>,
        use_nested_config: bool,
    ) -> (Self, Vec<ClientMessage>) {
        let (state, client_message) = Self::build_state(
            &root_path,
            explicit_config_path.as_deref(),
            use_nested_config,
            &js_config_loader,
        );
        (
            Self {
                root_path,
                source_formatter,
                js_config_loader,
                explicit_config_path,
                use_nested_config,
                state: RwLock::new(Arc::new(state)),
            },
            client_message,
        )
    }

    /// Build a fresh [`FormatterState`] from scratch.
    ///
    /// Called once in [`Self::new`] and again on every watched-file change.
    /// `.editorconfig` and `.prettierignore` are re-resolved here so add/remove events are picked up
    /// without a separate code path.
    ///
    /// LSP must keep editing usable even when the user's config is broken,
    /// so the root falls back to the default empty config with a warning instead of bubbling the error up.
    /// Likewise, a broken `.prettierignore` is skipped with a warning.
    fn build_state(
        root_path: &Path,
        explicit_config_path: Option<&Path>,
        use_nested_config: bool,
        js_config_loader: &JsConfigLoaderCb,
    ) -> (FormatterState, Vec<ClientMessage>) {
        let mut client_message = vec![];
        let scopes = match ConfigScopes::load(
            root_path,
            explicit_config_path,
            use_nested_config,
            Some(js_config_loader),
        ) {
            Ok(scopes) => scopes,
            Err(err) => {
                client_message.push(ClientMessage {
                    message: format!(
                        "{}: {err}",
                        root_path.to_string_lossy().cow_replace('\\', "/")
                    ),
                    r#type: MessageType::Error,
                });
                ConfigScopes::with_default_root(
                    root_path,
                    use_nested_config,
                    Some(js_config_loader),
                )
            }
        };

        // NOTE: `.gitignore` is intentionally NOT included here.
        // An LSP document is explicitly formatted by the user.
        // (CLI also formats explicitly specified git ignored files.)
        let ignore_matchers = match resolve_ignore_paths(root_path, &[])
            .and_then(|paths| build_global_ignore_matchers(root_path, &[], &paths))
        {
            Ok(matchers) => matchers,
            Err(err) => {
                client_message.push(ClientMessage {
                    message: format!("Failed to load .prettierignore: {err}"),
                    r#type: MessageType::Error,
                });
                vec![]
            }
        };

        (FormatterState { scopes, ignore_matchers }, client_message)
    }

    /// Snapshot the current state.
    /// In-flight reads survive a concurrent rebuild because the old `Arc` keeps the previous snapshot alive.
    fn snapshot(&self) -> Arc<FormatterState> {
        Arc::clone(&self.state.read().expect("state rwlock poisoned"))
    }

    /// Resolve config and format a file at the given path.
    /// Returns `None` if the file is unsupported or ignored.
    fn resolve_and_format(
        &self,
        scopes: &ConfigScopes,
        path: &Path,
        source_text: &str,
    ) -> Result<Option<FormatResult>, String> {
        let resolver = match scopes.resolve(path) {
            Ok(r) => r,
            Err(err) => {
                warn!(
                    "Failed to resolve nested config for {}: {err}, falling back to root",
                    path.display()
                );
                Arc::clone(scopes.root())
            }
        };

        if resolver.is_path_ignored(path, false) {
            debug!("File is ignored by config ignorePatterns: {}", path.display());
            return Ok(None);
        }

        let Some(strategy) = classify_file(path) else {
            debug!("Unsupported file type for formatting: {}", path.display());
            return Ok(None);
        };
        let plan = match resolver.resolve(path, strategy) {
            Ok(ResolveOutcome::Format(plan)) => plan,
            Ok(ResolveOutcome::MissingPlugin(plugin)) => {
                return Err(format!(
                    "`{plugin}` plugin is not enabled in resolved config: {}",
                    path.display()
                ));
            }
            Err(err) => {
                return Err(format!("Config resolve error for {}: {err}", path.display()));
            }
        };
        debug!("plan = {plan:?}");

        Ok(Some(tokio::task::block_in_place(|| self.source_formatter.format(source_text, plan))))
    }

    fn format_file(&self, path: &Path, source_text: &str) -> Result<Option<FormatResult>, String> {
        let state = self.snapshot();
        if is_ignored(&state.ignore_matchers, path, false, true) {
            debug!("File is ignored by .prettierignore: {}", path.display());
            return Ok(None);
        }
        self.resolve_and_format(&state.scopes, path, source_text)
    }

    fn format_in_memory(
        &self,
        uri: &Uri,
        source_text: &str,
        language_id: &LanguageId,
    ) -> Result<Option<FormatResult>, String> {
        let Some(path) = create_fake_file_path_from_language_id(language_id, &self.root_path, uri)
        else {
            debug!("Unsupported language id for in-memory formatting: {language_id:?}");
            return Ok(None);
        };
        self.resolve_and_format(&self.snapshot().scopes, &path, source_text)
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
///
/// Respects EOL characters as a whole edit. Appending `\r` before `\n` is invalid and should be treated as a single edit replacing `\n` with `\r\n`.
#[expect(clippy::cast_possible_truncation)]
fn compute_minimal_text_edit<'a>(
    source_text: &str,
    formatted_text: &'a str,
) -> (u32, u32, &'a str) {
    debug_assert_ne!(
        source_text, formatted_text,
        "compute_minimal_text_edit: source_text and formatted_text must be different"
    );

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

    while suffix_byte < src_len - prefix_byte && suffix_byte < fmt_len - prefix_byte {
        let src_idx = src_len - 1 - suffix_byte;
        let fmt_idx = fmt_len - 1 - suffix_byte;

        // Prevents appending `\r` to a line ending in `\n`, instead we want to replace `\n` with `\r\n` in this case,
        // aligning how LSP text documents are represented (line / column editing, instead of bytes).
        if src_bytes[src_idx] == b'\n' && fmt_bytes[fmt_idx] == b'\n' {
            let src_has_cr = src_idx > 0 && src_bytes[src_idx - 1] == b'\r';
            let fmt_has_cr = fmt_idx > 0 && fmt_bytes[fmt_idx - 1] == b'\r';

            if src_has_cr != fmt_has_cr {
                break;
            }
        }

        if src_bytes[src_idx] != fmt_bytes[fmt_idx] {
            break;
        }

        suffix_byte += 1;
    }

    let start = prefix_byte as u32;
    let end = (src_len - suffix_byte) as u32;
    let replacement_start = prefix_byte;
    let replacement_end = fmt_len - suffix_byte;
    let replacement = &formatted_text[replacement_start..replacement_end];

    (start, end, replacement)
}

// ---

#[cfg(test)]
mod tests_builder {
    use crate::lsp::server_formatter::ServerFormatterBuilder;
    use oxc_language_server::{Capabilities, ToolBuilder};
    use tower_lsp_server::gen_lsp_types::DocumentFormattingProvider;

    #[test]
    fn test_server_capabilities() {
        use tower_lsp_server::gen_lsp_types::ServerCapabilities;

        let builder = ServerFormatterBuilder::dummy();
        let mut capabilities = ServerCapabilities::default();

        builder.server_capabilities(&mut capabilities, &mut Capabilities::default());

        assert_eq!(
            capabilities.document_formatting_provider,
            Some(DocumentFormattingProvider::Bool(true))
        );
    }
}

#[cfg(test)]
mod tests {
    use super::compute_minimal_text_edit;
    use oxc_language_server::offset_to_position;

    #[test]
    #[should_panic(
        expected = "compute_minimal_text_edit: source_text and formatted_text must be different"
    )]
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
    fn test_lsp_edit_range_ignores_unicode_line_separators() {
        let src = "a\u{2028}b\u{2029}c \nnext";
        let formatted = "a\u{2028}b\u{2029}c\nnext";
        let (start, end, replacement) = compute_minimal_text_edit(src, formatted);

        let start_position = offset_to_position(src, start);
        let end_position = offset_to_position(src, end);

        assert_eq!((start_position.line, start_position.character), (0, 5));
        assert_eq!((end_position.line, end_position.character), (0, 6));
        assert_eq!(replacement, "");
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

    #[test]
    #[expect(clippy::cast_possible_truncation)]
    fn test_replacing_line_breaks() {
        let src_lf = "line1\nline2\nline3";
        let src_crlf = "line1\r\nline2\r\nline3";

        // from LF to CRLF
        {
            let (start, end, replacement) = compute_minimal_text_edit(src_lf, src_crlf);
            assert_eq!((start, end, replacement), (5, src_lf.len() as u32 - 5, "\r\nline2\r\n"));
        }

        // from CRLF to LF
        {
            let (start, end, replacement) = compute_minimal_text_edit(src_crlf, src_lf);
            assert_eq!((start, end, replacement), (5, src_crlf.len() as u32 - 5, "\nline2\n"));
        }
    }
}
