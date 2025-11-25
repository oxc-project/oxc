use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use log::{debug, warn};
use oxc_allocator::Allocator;
use oxc_data_structures::rope::{Rope, get_line_column};
use oxc_formatter::{
    FormatOptions, Formatter, Oxfmtrc, enable_jsx_source_type, get_parse_options,
    get_supported_source_type,
};
use oxc_parser::Parser;
use tower_lsp_server::{
    UriExt,
    lsp_types::{Pattern, Position, Range, ServerCapabilities, TextEdit, Uri},
};

use crate::{
    formatter::{FORMAT_CONFIG_FILES, options::FormatOptions as LSPFormatOptions},
    tool::{Tool, ToolBuilder, ToolRestartChanges},
    utils::normalize_path,
};

pub struct ServerFormatterBuilder;

impl ServerFormatterBuilder {
    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    pub fn build(root_uri: &Uri, options: serde_json::Value) -> ServerFormatter {
        let options = match serde_json::from_value::<LSPFormatOptions>(options) {
            Ok(opts) => opts,
            Err(err) => {
                warn!(
                    "Failed to deserialize LSPFormatOptions from JSON: {err}, falling back to default options"
                );
                LSPFormatOptions::default()
            }
        };
        if options.experimental {
            debug!("experimental formatter enabled");
        }
        let root_path = root_uri.to_file_path().unwrap();
        let oxfmtrc = Self::get_config(&root_path, options.config_path.as_ref());

        let gitignore_glob = if options.experimental {
            match Self::create_ignore_globs(
                &root_path,
                oxfmtrc.ignore_patterns.as_deref().unwrap_or(&[]),
            ) {
                Ok(glob) => Some(glob),
                Err(err) => {
                    warn!(
                        "Failed to create gitignore globs: {err}, proceeding without ignore globs"
                    );
                    None
                }
            }
        } else {
            None
        };

        ServerFormatter::new(
            Self::get_format_options(oxfmtrc),
            options.experimental,
            gitignore_glob,
        )
    }
}

impl ToolBuilder for ServerFormatterBuilder {
    fn server_capabilities(&self, capabilities: &mut ServerCapabilities) {
        capabilities.document_formatting_provider =
            Some(tower_lsp_server::lsp_types::OneOf::Left(true));
    }
    fn build_boxed(&self, root_uri: &Uri, options: serde_json::Value) -> Box<dyn Tool> {
        Box::new(ServerFormatterBuilder::build(root_uri, options))
    }
}

impl ServerFormatterBuilder {
    fn get_config(root_path: &Path, config_path: Option<&String>) -> Oxfmtrc {
        if let Some(config) = Self::search_config_file(root_path, config_path) {
            if let Ok(oxfmtrc) = Oxfmtrc::from_file(&config) {
                oxfmtrc
            } else {
                warn!("Failed to initialize oxfmtrc config: {}", config.to_string_lossy());
                Oxfmtrc::default()
            }
        } else {
            warn!(
                "Config file not found: {}, fallback to default config",
                config_path.unwrap_or(&FORMAT_CONFIG_FILES.join(", "))
            );
            Oxfmtrc::default()
        }
    }
    fn get_format_options(oxfmtrc: Oxfmtrc) -> FormatOptions {
        match oxfmtrc.into_format_options() {
            Ok(options) => options,
            Err(err) => {
                warn!("Failed to parse oxfmtrc config: {err}, fallback to default config");
                FormatOptions::default()
            }
        }
    }

    fn search_config_file(root_path: &Path, config_path: Option<&String>) -> Option<PathBuf> {
        if let Some(config_path) = config_path {
            let config = normalize_path(root_path.join(config_path));
            if config.try_exists().is_ok_and(|exists| exists) {
                return Some(config);
            }

            warn!(
                "Config file not found: {}, searching for `{}` in the root path",
                config.to_string_lossy(),
                FORMAT_CONFIG_FILES.join(", ")
            );
        }

        FORMAT_CONFIG_FILES.iter().find_map(|&file| {
            let config = normalize_path(root_path.join(file));
            config.try_exists().is_ok_and(|exists| exists).then_some(config)
        })
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
pub struct ServerFormatter {
    options: FormatOptions,
    should_run: bool,
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
            return ToolRestartChanges {
                tool: None,
                diagnostic_reports: None,
                watch_patterns: None,
            };
        }

        let new_formatter = ServerFormatterBuilder::build(root_uri, new_options_json.clone());
        let watch_patterns = new_formatter.get_watcher_patterns(new_options_json);
        ToolRestartChanges {
            tool: Some(Box::new(new_formatter)),
            diagnostic_reports: None,
            watch_patterns: Some(watch_patterns),
        }
    }

    fn get_watcher_patterns(&self, options: serde_json::Value) -> Vec<Pattern> {
        if !self.should_run {
            return vec![];
        }

        let options = match serde_json::from_value::<LSPFormatOptions>(options) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPFormatOptions from JSON: {e}. Falling back to default options."
                );
                LSPFormatOptions::default()
            }
        };

        if let Some(config_path) = options.config_path.as_ref() {
            return vec![config_path.clone()];
        }

        FORMAT_CONFIG_FILES.iter().map(|file| (*file).to_string()).collect()
    }

    fn handle_watched_file_change(
        &self,
        _changed_uri: &Uri,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> ToolRestartChanges {
        if !self.should_run {
            return ToolRestartChanges {
                tool: None,
                diagnostic_reports: None,
                watch_patterns: None,
            };
        }

        // TODO: Check if the changed file is actually a config file

        let new_formatter = ServerFormatterBuilder::build(root_uri, options);

        ToolRestartChanges {
            tool: Some(Box::new(new_formatter)),
            diagnostic_reports: None,
            // TODO: update watch patterns if config_path changed
            watch_patterns: None,
        }
    }

    fn run_format(&self, uri: &Uri, content: Option<&str>) -> Option<Vec<TextEdit>> {
        // Formatter is disabled
        if !self.should_run {
            return None;
        }

        let path = uri.to_file_path()?;

        if self.is_ignored(&path) {
            debug!("File is ignored: {}", path.display());
            return None;
        }

        let source_type = get_supported_source_type(&path).map(enable_jsx_source_type)?;
        // Declaring Variable to satisfy borrow checker
        let file_content;
        let source_text = if let Some(content) = content {
            content
        } else {
            #[cfg(not(all(test, windows)))]
            {
                file_content = std::fs::read_to_string(&path).ok()?;
            }
            #[cfg(all(test, windows))]
            #[expect(clippy::disallowed_methods)] // no `cow_replace` in tests are fine
            // On Windows, convert CRLF to LF for consistent formatting results
            {
                file_content = std::fs::read_to_string(&path).ok()?.replace("\r\n", "\n");
            }
            &file_content
        };

        let allocator = Allocator::new();
        let ret = Parser::new(&allocator, source_text, source_type)
            .with_options(get_parse_options())
            .parse();

        if !ret.errors.is_empty() {
            return None;
        }

        let code = Formatter::new(&allocator, self.options.clone()).build(&ret.program);

        // nothing has changed
        if code == *source_text {
            return Some(vec![]);
        }

        let (start, end, replacement) = compute_minimal_text_edit(source_text, &code);
        let rope = Rope::from(source_text);
        let (start_line, start_character) = get_line_column(&rope, start, source_text);
        let (end_line, end_character) = get_line_column(&rope, end, source_text);

        Some(vec![TextEdit::new(
            Range::new(
                Position::new(start_line, start_character),
                Position::new(end_line, end_character),
            ),
            replacement.to_string(),
        )])
    }
}

impl ServerFormatter {
    pub fn new(
        options: FormatOptions,
        should_run: bool,
        gitignore_glob: Option<Gitignore>,
    ) -> Self {
        Self { options, should_run, gitignore_glob }
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

// Almost the same as `oxfmt::walk::load_ignore_paths`, but does not handle custom ignore files.
fn load_ignore_paths(cwd: &Path) -> Vec<PathBuf> {
    [".gitignore", ".prettierignore"]
        .iter()
        .filter_map(|file_name| {
            let path = cwd.join(file_name);
            if path.exists() { Some(path) } else { None }
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests_builder {
    use crate::{ServerFormatterBuilder, ToolBuilder};

    #[test]
    fn test_server_capabilities() {
        use tower_lsp_server::lsp_types::{OneOf, ServerCapabilities};

        let builder = ServerFormatterBuilder;
        let mut capabilities = ServerCapabilities::default();

        builder.server_capabilities(&mut capabilities);

        assert_eq!(capabilities.document_formatting_provider, Some(OneOf::Left(true)));
    }
}

#[cfg(test)]
mod test_watchers {
    // formatter file watcher-system does not depend on the actual file system,
    // so we can use a fake directory for testing.
    const FAKE_DIR: &str = "fixtures/formatter/watchers";

    mod init_watchers {
        use crate::formatter::{server_formatter::test_watchers::FAKE_DIR, tester::Tester};
        use serde_json::json;

        #[test]
        fn test_default_options() {
            let patterns = Tester::new(FAKE_DIR, json!({})).get_watcher_patterns();
            assert!(patterns.is_empty());
        }

        #[test]
        fn test_formatter_experimental_enabled() {
            let patterns = Tester::new(
                FAKE_DIR,
                json!({
                    "fmt.experimental": true
                }),
            )
            .get_watcher_patterns();

            assert_eq!(patterns.len(), 2);
            assert_eq!(patterns[0], ".oxfmtrc.json");
            assert_eq!(patterns[1], ".oxfmtrc.jsonc");
        }

        #[test]
        fn test_formatter_custom_config_path() {
            let patterns = Tester::new(
                FAKE_DIR,
                json!({
                    "fmt.experimental": true,
                    "fmt.configPath": "configs/formatter.json"
                }),
            )
            .get_watcher_patterns();
            assert_eq!(patterns.len(), 1);
            assert_eq!(patterns[0], "configs/formatter.json");
        }
    }

    mod handle_configuration_change {
        use crate::{
            ToolRestartChanges,
            formatter::{server_formatter::test_watchers::FAKE_DIR, tester::Tester},
        };
        use serde_json::json;

        #[test]
        fn test_no_change() {
            let ToolRestartChanges { watch_patterns, .. } =
                Tester::new(FAKE_DIR, json!({})).handle_configuration_change(json!({}));

            assert!(watch_patterns.is_none());
        }

        #[test]
        fn test_no_changes_with_experimental() {
            let options = json!({
                "fmt.experimental": true
            });
            let ToolRestartChanges { watch_patterns, .. } =
                Tester::new(FAKE_DIR, options.clone()).handle_configuration_change(options);

            assert!(watch_patterns.is_none());
        }

        #[test]
        fn test_formatter_experimental_enabled() {
            let ToolRestartChanges { watch_patterns, .. } = Tester::new(FAKE_DIR, json!({}))
                .handle_configuration_change(json!({
                    "fmt.experimental": true
                }));

            assert!(watch_patterns.is_some());
            assert_eq!(watch_patterns.as_ref().unwrap().len(), 2);
            assert_eq!(watch_patterns.as_ref().unwrap()[0], ".oxfmtrc.json");
            assert_eq!(watch_patterns.as_ref().unwrap()[1], ".oxfmtrc.jsonc");
        }

        #[test]
        fn test_formatter_custom_config_path() {
            let ToolRestartChanges { watch_patterns, .. } = Tester::new(
                FAKE_DIR,
                json!({
                    "fmt.experimental": true,
                }),
            )
            .handle_configuration_change(json!({
                "fmt.experimental": true,
                "fmt.configPath": "configs/formatter.json"
            }));

            assert!(watch_patterns.is_some());
            assert_eq!(watch_patterns.as_ref().unwrap().len(), 1);
            assert_eq!(watch_patterns.as_ref().unwrap()[0], "configs/formatter.json");
        }

        #[test]
        fn test_formatter_disabling() {
            let ToolRestartChanges { watch_patterns, .. } = Tester::new(
                FAKE_DIR,
                json!({
                    "fmt.experimental": true
                }),
            )
            .handle_configuration_change(json!({
                "fmt.experimental": false
            }));

            assert!(watch_patterns.is_some());
            assert!(watch_patterns.unwrap().is_empty());
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::compute_minimal_text_edit;
    use crate::formatter::tester::Tester;

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

    #[test]
    fn test_formatter() {
        Tester::new(
            "fixtures/formatter/basic",
            json!({
                "fmt.experimental": true
            }),
        )
        .format_and_snapshot_single_file("basic.ts");
    }

    #[test]
    fn test_root_config_detection() {
        Tester::new(
            "fixtures/formatter/root_config",
            json!({
                "fmt.experimental": true
            }),
        )
        .format_and_snapshot_single_file("semicolons-as-needed.ts");
    }

    #[test]
    fn test_custom_config_path() {
        Tester::new(
            "fixtures/formatter/custom_config_path",
            json!({
                "fmt.experimental": true,
                "fmt.configPath": "./format.json",
            }),
        )
        .format_and_snapshot_single_file("semicolons-as-needed.ts");
    }

    #[test]
    fn test_ignore_files() {
        Tester::new(
            "fixtures/formatter/ignore-file",
            json!({
                "fmt.experimental": true
            }),
        )
        .format_and_snapshot_multiple_file(&["ignored.ts", "not-ignored.js"]);
    }

    #[test]
    fn test_ignore_pattern() {
        Tester::new(
            "fixtures/formatter/ignore-pattern",
            json!({
                "fmt.experimental": true
            }),
        )
        .format_and_snapshot_multiple_file(&["ignored.ts", "not-ignored.js"]);
    }
}
