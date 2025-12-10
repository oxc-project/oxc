use std::path::Path;

use editorconfig_parser::{EditorConfig, EditorConfigProperties};
use oxc_formatter::FormatOptions;

use crate::core::utils::read_to_string;

/// Find root `.editorconfig`.
/// <https://github.com/prettier/prettier/blob/v3.7/src/config/editorconfig/index.js>
pub fn find_root_editorconfig(cwd: &Path) -> Option<EditorConfig> {
    let root =
        cwd.ancestors().find(|path| path.join(".git").exists() || path.join(".hg").exists())?;
    let source = read_to_string(&root.join(".editorconfig")).ok()?;
    Some(EditorConfig::parse(&source))
}

pub fn merge_editorconfig_and_format_options(
    properties: EditorConfigProperties,
    options: &FormatOptions,
) -> FormatOptions {
}
