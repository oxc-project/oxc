use std::path::{Path, PathBuf};

use oxc_formatter::get_supported_source_type;
use oxc_span::SourceType;

pub enum FormatFileSource {
    OxcFormatter {
        path: PathBuf,
        source_type: SourceType,
    },
    #[expect(dead_code)]
    ExternalFormatter {
        path: PathBuf,
        parser_name: String,
    },
}

impl TryFrom<&Path> for FormatFileSource {
    type Error = ();

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        // TODO: This logic should(can) move to this file, after LSP support is also moved here.
        if let Some(source_type) = get_supported_source_type(path) {
            return Ok(Self::OxcFormatter { path: path.to_path_buf(), source_type });
        }

        // TODO: Support more files with `ExternalFormatter`
        // - JSON
        // - HTML(include .vue)
        // - CSS
        // - GraphQL
        // - Markdown
        // - YAML
        // - Handlebars

        Err(())
    }
}

impl FormatFileSource {
    pub fn path(&self) -> &Path {
        match self {
            Self::OxcFormatter { path, .. } | Self::ExternalFormatter { path, .. } => path,
        }
    }
}
