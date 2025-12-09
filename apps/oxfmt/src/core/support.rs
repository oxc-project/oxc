use std::path::{Path, PathBuf};

use phf::phf_set;

use oxc_formatter::get_supported_source_type;
use oxc_span::SourceType;

pub enum FormatFileSource {
    OxcFormatter {
        path: PathBuf,
        source_type: SourceType,
    },
    ExternalFormatter {
        path: PathBuf,
        #[cfg_attr(not(feature = "napi"), expect(dead_code))]
        parser_name: &'static str,
    },
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

    // Excluded files like lock files
    if EXCLUDE_FILENAMES.contains(file_name) {
        return None;
    }

    // JSON and variants
    if JSON_STRINGIFY_FILENAMES.contains(file_name) || extension == Some("importmap") {
        return Some("json-stringify");
    }
    if JSON_FILENAMES.contains(file_name) {
        return Some("json");
    }
    // Must be checked before generic JSON/JSONC
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

    // YAML
    if YAML_FILENAMES.contains(file_name) {
        return Some("yaml");
    }
    if let Some(ext) = extension
        && YAML_EXTENSIONS.contains(ext)
    {
        return Some("yaml");
    }

    // Markdown and variants
    if MARKDOWN_FILENAMES.contains(file_name) {
        return Some("markdown");
    }
    if let Some(ext) = extension
        && MARKDOWN_EXTENSIONS.contains(ext)
    {
        return Some("markdown");
    }
    if extension == Some("mdx") {
        return Some("mdx");
    }

    // HTML and variants
    // Must be checked before generic HTML
    if file_name.ends_with(".component.html") {
        return Some("angular");
    }
    if let Some(ext) = extension
        && HTML_EXTENSIONS.contains(ext)
    {
        return Some("html");
    }
    if extension == Some("vue") {
        return Some("vue");
    }
    if extension == Some("mjml") {
        return Some("mjml");
    }

    // CSS and variants
    if let Some(ext) = extension
        && CSS_EXTENSIONS.contains(ext)
    {
        return Some("css");
    }
    if extension == Some("less") {
        return Some("less");
    }
    if extension == Some("scss") {
        return Some("scss");
    }

    // GraphQL
    if let Some(ext) = extension
        && GRAPHQL_EXTENSIONS.contains(ext)
    {
        return Some("graphql");
    }

    // Handlebars
    if let Some(ext) = extension
        && HANDLEBARS_EXTENSIONS.contains(ext)
    {
        return Some("glimmer");
    }

    None
}

static EXCLUDE_FILENAMES: phf::Set<&'static str> = phf_set! {
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "MODULE.bazel.lock",
    "bun.lock",
    "deno.lock",
    "composer.lock",
    "Package.resolved",
    "Pipfile.lock",
    "flake.lock",
    "mcmod.info",
};

static JSON_STRINGIFY_FILENAMES: phf::Set<&'static str> = phf_set! {
    "package.json",
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

static HTML_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "html",
    "hta",
    "htm",
    "inc",
    "xht",
    "xhtml",
};

static CSS_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "css",
    "wxss",
    "pcss",
    "postcss",
};

static GRAPHQL_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "graphql",
    "gql",
    "graphqls",
};

static HANDLEBARS_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "handlebars",
    "hbs",
};

static MARKDOWN_FILENAMES: phf::Set<&'static str> = phf_set! {
    "contents.lr",
    "README",
};

static MARKDOWN_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "md",
    "livemd",
    "markdown",
    "mdown",
    "mdwn",
    "mkd",
    "mkdn",
    "mkdown",
    "ronn",
    "scd",
    "workbook",
};

static YAML_FILENAMES: phf::Set<&'static str> = phf_set! {
    ".clang-format",
    ".clang-tidy",
    ".clangd",
    ".gemrc",
    "CITATION.cff",
    "glide.lock",
    "pixi.lock",
    ".prettierrc",
    ".stylelintrc",
    ".lintstagedrc",
};

static YAML_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "yml",
    "mir",
    "reek",
    "rviz",
    "sublime-syntax",
    "syntax",
    "yaml",
    "yaml-tmlanguage",
};

// ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_external_parser_name() {
        let test_cases = vec![
            // JSON
            ("package.json", Some("json-stringify")),
            ("config.importmap", Some("json-stringify")),
            ("tsconfig.json", Some("jsonc")),
            ("jsconfig.dev.json", Some("jsonc")),
            ("data.json", Some("json")),
            ("schema.avsc", Some("json")),
            ("config.code-workspace", Some("jsonc")),
            ("settings.json5", Some("json5")),
            // HTML
            ("index.html", Some("html")),
            ("page.htm", Some("html")),
            ("template.xhtml", Some("html")),
            // Angular (must be detected before HTML)
            ("app.component.html", Some("angular")),
            // MJML
            ("email.mjml", Some("mjml")),
            // Vue
            ("App.vue", Some("vue")),
            // CSS
            ("styles.css", Some("css")),
            ("app.wxss", Some("css")),
            ("styles.pcss", Some("css")),
            ("styles.postcss", Some("css")),
            ("theme.less", Some("less")),
            ("main.scss", Some("scss")),
            // GraphQL
            ("schema.graphql", Some("graphql")),
            ("query.gql", Some("graphql")),
            ("types.graphqls", Some("graphql")),
            // Handlebars
            ("template.handlebars", Some("glimmer")),
            ("partial.hbs", Some("glimmer")),
            // Markdown
            ("README", Some("markdown")),
            ("contents.lr", Some("markdown")),
            ("docs.md", Some("markdown")),
            ("guide.markdown", Some("markdown")),
            ("notes.mdown", Some("markdown")),
            ("page.mdx", Some("mdx")),
            // YAML
            (".clang-format", Some("yaml")),
            (".prettierrc", Some("yaml")),
            ("config.yml", Some("yaml")),
            ("settings.yaml", Some("yaml")),
            ("grammar.sublime-syntax", Some("yaml")),
            // Excluded lock files
            ("package-lock.json", None),
            ("pnpm-lock.yaml", None),
            ("yarn.lock", None),
            // Unknown
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
