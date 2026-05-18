use std::path::{Path, PathBuf};

use editorconfig_parser::{
    EditorConfig, EditorConfigProperties, EditorConfigProperty, EndOfLine, IndentStyle,
    MaxLineLength, QuoteType,
};

use crate::core::{
    oxfmtrc::{EndOfLineConfig, FormatConfig},
    utils,
};

/// Find the nearest `.editorconfig` walking up from `cwd`.
pub fn resolve_editorconfig_path(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors().map(|dir| dir.join(".editorconfig")).find(|p| p.exists())
}

/// Load `.editorconfig` from a path if provided.
///
/// Section patterns like `[src/*.ts]` are anchored at the `.editorconfig`'s
/// own directory, so `path.parent()` is used as the base. Real callers always
/// pass an absolute path (via `resolve_editorconfig_path`), making the `.` fallback
/// only a theoretical safety net for a bare `.editorconfig` filename.
pub fn load_editorconfig(editorconfig_path: Option<&Path>) -> Result<Option<EditorConfig>, String> {
    match editorconfig_path {
        Some(path) => {
            let str = utils::read_to_string(path)
                .map_err(|_| format!("Failed to read {}: File not found", path.display()))?;
            let cwd = path.parent().unwrap_or_else(|| Path::new("."));
            Ok(Some(EditorConfig::parse(&str).with_cwd(cwd)))
        }
        None => Ok(None),
    }
}

/// Check if `.editorconfig` has per-file overrides for this path.
///
/// Returns `true` if the resolved properties differ from the root `[*]` section.
///
/// Currently, only the following properties are considered for overrides:
/// - max_line_length
/// - end_of_line
/// - indent_style
/// - indent_size
/// - tab_width
/// - insert_final_newline
/// - quote_type
pub fn has_editorconfig_overrides(editorconfig: &EditorConfig, path: &Path) -> bool {
    let sections = editorconfig.sections();

    // No sections, or only root `[*]` section → no overrides
    if sections.is_empty() || matches!(sections, [s] if s.name == "*") {
        return false;
    }

    let resolved = editorconfig.resolve(path);

    // Get the root `[*]` section properties
    let root_props = sections.iter().find(|s| s.name == "*").map(|s| &s.properties);

    // Compare only the properties we care about
    match root_props {
        Some(root) => {
            resolved.max_line_length != root.max_line_length
                || resolved.end_of_line != root.end_of_line
                || resolved.indent_style != root.indent_style
                || resolved.indent_size != root.indent_size
                || resolved.tab_width != root.tab_width
                || resolved.insert_final_newline != root.insert_final_newline
                || resolved.quote_type != root.quote_type
        }
        // No `[*]` section means any resolved property is an override
        None => {
            resolved.max_line_length != EditorConfigProperty::Unset
                || resolved.end_of_line != EditorConfigProperty::Unset
                || resolved.indent_style != EditorConfigProperty::Unset
                || resolved.indent_size != EditorConfigProperty::Unset
                || resolved.tab_width != EditorConfigProperty::Unset
                || resolved.insert_final_newline != EditorConfigProperty::Unset
                || resolved.quote_type != EditorConfigProperty::Unset
        }
    }
}

/// Apply `.editorconfig` properties to `FormatConfig`.
///
/// Only applies values that are not already set in the user's config.
/// NOTE: Only properties checked by [`has_editorconfig_overrides`] are applied here.
pub fn apply_editorconfig(config: &mut FormatConfig, props: &EditorConfigProperties) {
    #[expect(clippy::cast_possible_truncation)]
    if config.print_width.is_none()
        && let EditorConfigProperty::Value(MaxLineLength::Number(v)) = props.max_line_length
    {
        config.print_width = Some(v as u16);
    }

    if config.end_of_line.is_none()
        && let EditorConfigProperty::Value(eol) = props.end_of_line
    {
        config.end_of_line = Some(match eol {
            EndOfLine::Lf => EndOfLineConfig::Lf,
            EndOfLine::Cr => EndOfLineConfig::Cr,
            EndOfLine::Crlf => EndOfLineConfig::Crlf,
        });
    }

    if config.use_tabs.is_none()
        && let EditorConfigProperty::Value(style) = props.indent_style
    {
        config.use_tabs = Some(match style {
            IndentStyle::Tab => true,
            IndentStyle::Space => false,
        });
    }

    if config.tab_width.is_none() {
        // Match Prettier's behavior: Only use `indent_size` when `useTabs: false`.
        // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/config/editorconfig/editorconfig-to-prettier.js#L25-L30
        #[expect(clippy::cast_possible_truncation)]
        if config.use_tabs == Some(false)
            && let EditorConfigProperty::Value(size) = props.indent_size
        {
            config.tab_width = Some(size as u8);
        } else if let EditorConfigProperty::Value(size) = props.tab_width {
            config.tab_width = Some(size as u8);
        }
    }

    if config.insert_final_newline.is_none()
        && let EditorConfigProperty::Value(v) = props.insert_final_newline
    {
        config.insert_final_newline = Some(v);
    }

    if config.single_quote.is_none() {
        match props.quote_type {
            EditorConfigProperty::Value(QuoteType::Single) => {
                config.single_quote = Some(true);
            }
            EditorConfigProperty::Value(QuoteType::Double) => {
                config.single_quote = Some(false);
            }
            _ => {}
        }
    }
}
