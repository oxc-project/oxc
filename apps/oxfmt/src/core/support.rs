use std::path::{Path, PathBuf};

use phf::phf_set;

use oxc_formatter::get_supported_source_type;
use oxc_span::SourceType;

pub enum FormatFileSource {
    OxcFormatter { path: PathBuf, source_type: SourceType },
    ExternalFormatter { path: PathBuf, parser_name: &'static str },
}

impl TryFrom<PathBuf> for FormatFileSource {
    type Error = ();

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        // TODO: This logic should(can) move to this file, after LSP support is also moved here.
        if let Some(source_type) = get_supported_source_type(&path) {
            return Ok(Self::OxcFormatter { path, source_type });
        }

        if let Some(parser_name) = get_external_parser_name(&path) {
            return Ok(Self::ExternalFormatter { path, parser_name });
        }

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

// ---

/// Returns the Prettier parser name for file at `path`, if supported.
/// See also `prettier --support-info | jq '.languages[]'`
/// NOTE: The order matters: more specific matches (like `package.json`) must come before generic ones.
fn get_external_parser_name(path: &Path) -> Option<&'static str> {
    let file_name = path.file_name()?.to_str()?;
    let extension = path.extension().and_then(|ext| ext.to_str());

    // O(1) file name check goes first
    if JSON_STRINGIFY_FILENAMES.contains(file_name) || extension == Some("importmap") {
        return Some("json-stringify");
    }
    if JSON_FILENAMES.contains(file_name) {
        return Some("json");
    }

    // Then, check by extension
    if (file_name.starts_with("tsconfig.") || file_name.starts_with("jsconfig."))
        && extension == Some("json")
    {
        return Some("jsonc");
    }
    if let Some(ext) = extension
        && JSON_EXTENSIONS.contains(ext)
    {
        return Some("json");
    }
    if let Some(ext) = extension
        && JSONC_EXTENSIONS.contains(ext)
    {
        return Some("jsonc");
    }
    if extension == Some("json5") {
        return Some("json5");
    }

    // TODO: Support more default supported file types
    // {
    //   "extensions": [".html", ".hta", ".htm", ".html.hl", ".inc", ".xht", ".xhtml"],
    //   "name": "HTML",
    //   "parsers": ["html"]
    // },
    // {
    //   "extensions": [".mjml"],
    //   "name": "MJML",
    //   "parsers": ["mjml"]
    // },
    // {
    //   "extensions": [".component.html"],
    //   "name": "Angular",
    //   "parsers": ["angular"]
    // },
    // {
    //   "extensions": [".vue"],
    //   "name": "Vue",
    //   "parsers": ["vue"]
    // },
    //
    // {
    //   "extensions": [".css", ".wxss"],
    //   "name": "CSS",
    //   "parsers": ["css"]
    // },
    // {
    //   "extensions": [".less"],
    //   "name": "Less",
    //   "parsers": ["less"]
    // },
    // {
    //   "extensions": [".pcss", ".postcss"],
    //   "group": "CSS",
    //   "name": "PostCSS",
    //   "parsers": ["css"]
    // },
    // {
    //   "extensions": [".scss"],
    //   "name": "SCSS",
    //   "parsers": ["scss"]
    // },
    //
    // {
    //   "extensions": [".graphql", ".gql", ".graphqls"],
    //   "name": "GraphQL",
    //   "parsers": ["graphql"]
    // },
    //
    // {
    //   "extensions": [".handlebars", ".hbs"],
    //   "name": "Handlebars",
    //   "parsers": ["glimmer"]
    // },
    //
    // {
    //   "extensions": [".md", ".livemd", ".markdown", ".mdown", ".mdwn", ".mkd", ".mkdn", ".mkdown", ".ronn", ".scd", ".workbook"],
    //   "filenames": ["contents.lr", "README"],
    //   "name": "Markdown",
    //   "parsers": ["markdown"]
    // },
    // {
    //   "extensions": [".mdx"],
    //   "name": "MDX",
    //   "parsers": ["mdx"]
    // },

    None
}

static JSON_STRINGIFY_FILENAMES: phf::Set<&'static str> = phf_set! {
    "package.json",
    "package-lock.json",
    "composer.json",
};

static JSON_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "json",
    "4DForm",
    "4DProject",
    "avsc",
    "geojson",
    "gltf",
    "har",
    "ice",
    "JSON-tmLanguage",
    "json.example",
    "mcmeta",
    "sarif",
    "tact",
    "tfstate",
    "tfstate.backup",
    "topojson",
    "webapp",
    "webmanifest",
    "yy",
    "yyp",
};

static JSON_FILENAMES: phf::Set<&'static str> = phf_set! {
    ".all-contributorsrc",
    ".arcconfig",
    ".auto-changelog",
    ".c8rc",
    ".htmlhintrc",
    ".imgbotconfig",
    ".nycrc",
    ".tern-config",
    ".tern-project",
    ".watchmanconfig",
    ".babelrc",
    ".jscsrc",
    ".jshintrc",
    ".jslintrc",
    ".swcrc",
};

static JSONC_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "jsonc",
    "code-snippets",
    "code-workspace",
    "sublime-build",
    "sublime-color-scheme",
    "sublime-commands",
    "sublime-completions",
    "sublime-keymap",
    "sublime-macro",
    "sublime-menu",
    "sublime-mousemap",
    "sublime-project",
    "sublime-settings",
    "sublime-theme",
    "sublime-workspace",
    "sublime_metrics",
    "sublime_session",
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_external_parser_name() {
        let test_cases = vec![
            ("package.json", Some("json-stringify")),
            ("package-lock.json", Some("json-stringify")),
            ("config.importmap", Some("json-stringify")),
            ("tsconfig.json", Some("jsonc")),
            ("jsconfig.dev.json", Some("jsonc")),
            ("data.json", Some("json")),
            ("schema.avsc", Some("json")),
            ("config.code-workspace", Some("jsonc")),
            ("unknown.txt", None),
            ("prof.png", None),
            ("foo", None),
        ];

        for (file_name, expected) in test_cases {
            let result = get_external_parser_name(Path::new(file_name));
            assert_eq!(result, expected, "`{file_name}` should be parsed as {expected:?}");
        }
    }
}
