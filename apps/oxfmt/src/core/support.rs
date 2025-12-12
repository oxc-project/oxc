use std::path::{Path, PathBuf};

use phf::phf_set;

use oxc_formatter::get_supported_source_type;
use oxc_span::SourceType;

pub enum FormatFileStrategy {
    OxcFormatter {
        path: PathBuf,
        source_type: SourceType,
    },
    ExternalFormatter {
        path: PathBuf,
        #[cfg_attr(not(feature = "napi"), expect(dead_code))]
        parser_name: &'static str,
    },
    /// `package.json` is special: sorted by `sort-package-json` then formatted by external formatter.
    ExternalFormatterPackageJson {
        path: PathBuf,
        #[cfg_attr(not(feature = "napi"), expect(dead_code))]
        parser_name: &'static str,
    },
}

impl TryFrom<PathBuf> for FormatFileStrategy {
    type Error = ();

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        // TODO: This logic should(can) move to this file, after LSP support is also moved here.
        if let Some(source_type) = get_supported_source_type(&path) {
            return Ok(Self::OxcFormatter { path, source_type });
        }

        if let Some(source) = get_external_format_source(path) {
            return Ok(source);
        }

        Err(())
    }
}

impl FormatFileStrategy {
    pub fn path(&self) -> &Path {
        match self {
            Self::OxcFormatter { path, .. }
            | Self::ExternalFormatter { path, .. }
            | Self::ExternalFormatterPackageJson { path, .. } => path,
        }
    }
}

// ---

/// Returns `FormatFileSource` for external formatter, if supported.
/// See also `prettier --support-info | jq '.languages[]'`
/// NOTE: The order matters: more specific matches (like `package.json`) must come before generic ones.
fn get_external_format_source(path: PathBuf) -> Option<FormatFileStrategy> {
    let file_name = path.file_name()?.to_str()?;
    let extension = path.extension().and_then(|ext| ext.to_str());

    // Excluded files like lock files
    if EXCLUDE_FILENAMES.contains(file_name) {
        return None;
    }

    // JSON and variants
    // `package.json` is special case
    if file_name == "package.json" {
        return Some(FormatFileStrategy::ExternalFormatterPackageJson {
            path,
            parser_name: "json-stringify",
        });
    }
    if JSON_STRINGIFY_FILENAMES.contains(file_name) || extension == Some("importmap") {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "json-stringify" });
    }
    if JSON_FILENAMES.contains(file_name) {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "json" });
    }
    if let Some(ext) = extension
        && JSON_EXTENSIONS.contains(ext)
    {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "json" });
    }
    if let Some(ext) = extension
        && JSONC_EXTENSIONS.contains(ext)
    {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "jsonc" });
    }
    if extension == Some("json5") {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "json5" });
    }

    // YAML
    if YAML_FILENAMES.contains(file_name) {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "yaml" });
    }
    if let Some(ext) = extension
        && YAML_EXTENSIONS.contains(ext)
    {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "yaml" });
    }

    // Markdown and variants
    if MARKDOWN_FILENAMES.contains(file_name) {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "markdown" });
    }
    if let Some(ext) = extension
        && MARKDOWN_EXTENSIONS.contains(ext)
    {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "markdown" });
    }
    if extension == Some("mdx") {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "mdx" });
    }

    // HTML and variants
    // Must be checked before generic HTML
    if file_name.ends_with(".component.html") {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "angular" });
    }
    if let Some(ext) = extension
        && HTML_EXTENSIONS.contains(ext)
    {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "html" });
    }
    if extension == Some("vue") {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "vue" });
    }
    if extension == Some("mjml") {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "mjml" });
    }

    // CSS and variants
    if let Some(ext) = extension
        && CSS_EXTENSIONS.contains(ext)
    {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "css" });
    }
    if extension == Some("less") {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "less" });
    }
    if extension == Some("scss") {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "scss" });
    }

    // GraphQL
    if let Some(ext) = extension
        && GRAPHQL_EXTENSIONS.contains(ext)
    {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "graphql" });
    }

    // Handlebars
    if let Some(ext) = extension
        && HANDLEBARS_EXTENSIONS.contains(ext)
    {
        return Some(FormatFileStrategy::ExternalFormatter { path, parser_name: "glimmer" });
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
    // NOTE: `package.json` is handled separately as `ExternalFormatterPackageJson`
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

    fn get_parser_name(file_name: &str) -> Option<&'static str> {
        match get_external_format_source(PathBuf::from(file_name)) {
            Some(
                FormatFileStrategy::ExternalFormatter { parser_name, .. }
                | FormatFileStrategy::ExternalFormatterPackageJson { parser_name, .. },
            ) => Some(parser_name),
            _ => None,
        }
    }

    #[test]
    fn test_get_external_format_source() {
        let test_cases = vec![
            // JSON
            ("package.json", Some("json-stringify")),
            ("config.importmap", Some("json-stringify")),
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
            let result = get_parser_name(file_name);
            assert_eq!(result, expected, "`{file_name}` should be parsed as {expected:?}");
        }
    }

    #[test]
    fn test_package_json_is_special() {
        let source = get_external_format_source(PathBuf::from("package.json")).unwrap();
        assert!(matches!(source, FormatFileStrategy::ExternalFormatterPackageJson { .. }));

        let source = get_external_format_source(PathBuf::from("composer.json")).unwrap();
        assert!(matches!(source, FormatFileStrategy::ExternalFormatter { .. }));
    }
}
