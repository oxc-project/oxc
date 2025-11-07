use std::path::{Path, PathBuf};

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
    lsp_types::{Pattern, Position, Range, TextEdit, Uri},
};

use crate::{
    FORMAT_CONFIG_FILES, formatter::options::FormatOptions as LSPFormatOptions,
    utils::normalize_path, worker::ToolRestartChanges,
};

pub struct ServerFormatterBuilder {
    root_uri: Uri,
    options: LSPFormatOptions,
}

impl ServerFormatterBuilder {
    pub fn new(root_uri: Uri, options: serde_json::Value) -> Self {
        let options = match serde_json::from_value::<LSPFormatOptions>(options) {
            Ok(opts) => opts,
            Err(err) => {
                warn!(
                    "Failed to deserialize LSPFormatOptions from JSON: {err}, falling back to default options"
                );
                LSPFormatOptions::default()
            }
        };
        Self { root_uri, options }
    }

    pub fn build(self) -> ServerFormatter {
        if self.options.experimental {
            debug!("experimental formatter enabled");
        }
        let root_path = self.root_uri.to_file_path().unwrap();

        ServerFormatter::new(
            Self::get_format_options(&root_path, self.options.config_path.as_ref()),
            self.options.experimental,
        )
    }

    fn get_format_options(root_path: &Path, config_path: Option<&String>) -> FormatOptions {
        let oxfmtrc = if let Some(config) = Self::search_config_file(root_path, config_path) {
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
        };

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
}
pub struct ServerFormatter {
    options: FormatOptions,
    should_run: bool,
}

impl ServerFormatter {
    pub fn new(options: FormatOptions, should_run: bool) -> Self {
        Self { options, should_run }
    }

    pub fn run_single(&self, uri: &Uri, content: Option<String>) -> Option<Vec<TextEdit>> {
        // Formatter is disabled
        if !self.should_run {
            return None;
        }

        let path = uri.to_file_path()?;
        let source_type = get_supported_source_type(&path).map(enable_jsx_source_type)?;
        let source_text = if let Some(content) = content {
            content
        } else {
            #[cfg(not(all(test, windows)))]
            let source_text = std::fs::read_to_string(&path).ok()?;
            #[cfg(all(test, windows))]
            #[expect(clippy::disallowed_methods)] // no `cow_replace` in tests are fine
            // On Windows, convert CRLF to LF for consistent formatting results
            let source_text = std::fs::read_to_string(&path).ok()?.replace("\r\n", "\n");
            source_text
        };

        let allocator = Allocator::new();
        let ret = Parser::new(&allocator, &source_text, source_type)
            .with_options(get_parse_options())
            .parse();

        if !ret.errors.is_empty() {
            return None;
        }

        let code = Formatter::new(&allocator, self.options.clone()).build(&ret.program);

        // nothing has changed
        if code == source_text {
            return Some(vec![]);
        }

        let (start, end, replacement) = compute_minimal_text_edit(&source_text, &code);
        let rope = Rope::from(source_text.as_str());
        let (start_line, start_character) = get_line_column(&rope, start, &source_text);
        let (end_line, end_character) = get_line_column(&rope, end, &source_text);

        Some(vec![TextEdit::new(
            Range::new(
                Position::new(start_line, start_character),
                Position::new(end_line, end_character),
            ),
            replacement.to_string(),
        )])
    }

    /// # Panics
    /// Panics if the root URI cannot be converted to a file path.
    #[expect(clippy::unused_self)]
    pub fn handle_configuration_change(
        &self,
        root_uri: &Uri,
        old_options_json: &serde_json::Value,
        new_options_json: serde_json::Value,
    ) -> ToolRestartChanges<ServerFormatter> {
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

        let new_formatter =
            ServerFormatterBuilder::new(root_uri.clone(), new_options_json.clone()).build();
        let watch_patterns = new_formatter.get_watcher_patterns(new_options_json);
        ToolRestartChanges {
            tool: Some(new_formatter),
            diagnostic_reports: None,
            watch_patterns: Some(watch_patterns),
        }
    }

    pub fn get_watcher_patterns(&self, options: serde_json::Value) -> Vec<Pattern> {
        let options = match serde_json::from_value::<LSPFormatOptions>(options) {
            Ok(opts) => opts,
            Err(e) => {
                warn!(
                    "Failed to deserialize LSPFormatOptions from JSON: {e}. Falling back to default options."
                );
                LSPFormatOptions::default()
            }
        };

        if !self.should_run {
            return vec![];
        }

        if let Some(config_path) = options.config_path.as_ref() {
            return vec![config_path.clone()];
        }

        FORMAT_CONFIG_FILES.iter().map(|file| (*file).to_string()).collect()
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
}
