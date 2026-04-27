use std::path::{Path, PathBuf};

use phf::phf_set;

use oxc_span::SourceType;

pub enum FormatStrategy {
    OxcFormatter {
        path: PathBuf,
        source_type: SourceType,
    },
    /// TOML files formatted by taplo (Pure Rust).
    OxfmtToml {
        path: PathBuf,
    },
    ExternalFormatter {
        path: PathBuf,
        parser_name: &'static str,
    },
    /// `package.json` is special: sorted by `sort-package-json` then formatted by external formatter.
    ExternalFormatterPackageJson {
        path: PathBuf,
        parser_name: &'static str,
    },
}

/// Builder for creating [`FormatStrategy`] from file paths.
///
/// Carries per-scope configuration (e.g. experimental language flags)
/// that influences which files are supported and how they are formatted.
///
/// Created from [`super::ConfigResolver::strategy_builder()`] for CLI/LSP paths,
/// or via [`FormatStrategyBuilder::default()`] for the NAPI API path.
#[derive(Debug, Default)]
pub struct FormatStrategyBuilder {
    // Future: experimental_svelte, experimental_astro, etc.
    _private: (),
}

impl FormatStrategyBuilder {
    /// Build a [`FormatStrategy`] from a file path.
    ///
    /// Returns `Ok` if the file type is supported, `Err(())` otherwise.
    #[expect(clippy::unused_self)] // Will use `self` when experimental flags are added
    pub fn build(&self, path: PathBuf) -> Result<FormatStrategy, ()> {
        // Check JS/TS files first
        if let Some(source_type) = get_oxc_formatter_source_type(&path) {
            return Ok(FormatStrategy::OxcFormatter { path, source_type });
        }

        // Extract file_name and extension once for all subsequent checks
        let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else {
            return Err(());
        };

        // Excluded files like lock files
        if EXCLUDE_FILENAMES.contains(file_name) {
            return Err(());
        }

        // Then TOML files
        if is_toml_file(file_name) {
            return Ok(FormatStrategy::OxfmtToml { path });
        }

        // Then external formatter files
        // `package.json` is special: sorted then formatted
        if file_name == "package.json" {
            return Ok(FormatStrategy::ExternalFormatterPackageJson {
                path,
                parser_name: "json-stringify",
            });
        }

        let extension = path.extension().and_then(|ext| ext.to_str());
        if let Some(parser_name) = get_external_parser_name(file_name, extension) {
            return Ok(FormatStrategy::ExternalFormatter { path, parser_name });
        }

        Err(())
    }
}

impl FormatStrategy {
    #[cfg(not(feature = "napi"))]
    pub fn can_format_without_external(&self) -> bool {
        matches!(self, Self::OxcFormatter { .. } | Self::OxfmtToml { .. })
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::OxcFormatter { path, .. }
            | Self::OxfmtToml { path }
            | Self::ExternalFormatter { path, .. }
            | Self::ExternalFormatterPackageJson { path, .. } => path,
        }
    }

    /// Returns `true` if this strategy supports the Tailwind CSS sorting plugin.
    pub fn needs_tailwind_plugin(&self) -> bool {
        match self {
            Self::OxcFormatter { .. } => true,
            #[cfg(feature = "napi")]
            Self::ExternalFormatter { parser_name, .. } => TAILWIND_PARSERS.contains(parser_name),
            _ => false,
        }
    }

    /// Returns `true` if this strategy supports the `prettier-plugin-oxfmt` (js-in-xxx).
    #[cfg(feature = "napi")]
    pub fn needs_oxfmt_plugin(&self) -> bool {
        matches!(
            self,
            Self::ExternalFormatter { parser_name, .. } if OXFMT_PARSERS.contains(parser_name)
        )
    }
}

/// Parsers(files) that benefit from Tailwind plugin.
#[cfg(feature = "napi")]
static TAILWIND_PARSERS: phf::Set<&'static str> = phf_set! {
    "html",
    "vue",
    "angular",
    "glimmer",
    "css",
    "scss",
    "less",
};

/// Parsers(files) that can embed JS/TS code and benefit from oxfmt plugin.
/// For now, expressions are not supported.
/// - e.g. `__vue_expression` in `vue`, `__ng_directive` in `angular`
#[cfg(feature = "napi")]
static OXFMT_PARSERS: phf::Set<&'static str> = phf_set! {
    // "html",
    "vue",
    // "markdown",
    // "mdx",
};

static EXCLUDE_FILENAMES: phf::Set<&'static str> = phf_set! {
    // JSON, YAML lock files
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
    // TOML lock files
    "Cargo.lock",
    "Gopkg.lock",
    "pdm.lock",
    "poetry.lock",
    "uv.lock",
};

// ---

/// Returns `true` if this is a TOML file.
fn is_toml_file(file_name: &str) -> bool {
    if TOML_FILENAMES.contains(file_name) {
        return true;
    }

    #[expect(clippy::case_sensitive_file_extension_comparisons)]
    if file_name.ends_with(".toml.example") || file_name.ends_with(".toml") {
        return true;
    }

    false
}

static TOML_FILENAMES: phf::Set<&'static str> = phf_set! {
    "Pipfile",
    "Cargo.toml.orig",
};

// ---

/// Returns parser name for external formatter, if supported.
/// See also `prettier --support-info | jq '.languages[]'`
fn get_external_parser_name(file_name: &str, extension: Option<&str>) -> Option<&'static str> {
    // JSON and variants
    // NOTE: `package.json` is handled separately in `FormatStrategyBuilder::build()`
    if file_name == "composer.json" || extension == Some("importmap") {
        return Some("json-stringify");
    }
    if JSON_FILENAMES.contains(file_name) {
        return Some("json");
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

// Additional extensions from linguist-languages, which Prettier also supports
// - https://github.com/ikatyang-collab/linguist-languages/blob/d1dc347c7ced0f5b42dd66c7d1c4274f64a3eb6b/data/JavaScript.js
// No special extensions for TypeScript
// - https://github.com/ikatyang-collab/linguist-languages/blob/d1dc347c7ced0f5b42dd66c7d1c4274f64a3eb6b/data/TypeScript.js
// And on top of this data, Prettier adds its own checks.
// Ultimately, it can be confirmed with the following command.
// `prettier --support-info | jq '.languages[] | select(.name == "JavaScript")'`
static ADDITIONAL_JS_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "_js",
    "bones",
    "es",
    "es6",
    "gs",
    "jake",
    "javascript",
    "jsb",
    "jscad",
    "jsfl",
    "jslib",
    "jsm",
    "jspre",
    "jss",
    "njs",
    "pac",
    "sjs",
    "ssjs",
    "xsjs",
    "xsjslib",
};

// Special filenames that are valid JS files
static SPECIAL_JS_FILENAMES: phf::Set<&'static str> = phf_set! {
    "Jakefile",
    "start.frag",
    "end.frag",
};

fn get_oxc_formatter_source_type(path: &Path) -> Option<SourceType> {
    // Standard extensions, also supported by `oxc_span::VALID_EXTENSIONS`
    // NOTE: Use `path` directly for `.d.ts` detection
    if let Ok(source_type) = SourceType::from_path(path) {
        return Some(source_type);
    }

    // Check special filenames first
    if let Some(file_name) = path.file_name()
        && SPECIAL_JS_FILENAMES.contains(file_name.to_str()?)
    {
        return Some(SourceType::default());
    }

    let extension = path.extension()?.to_string_lossy();
    // Additional extensions Prettier also supports
    if ADDITIONAL_JS_EXTENSIONS.contains(extension.as_ref()) {
        return Some(SourceType::default());
    }
    // Special handling for `.frag` files: only allow `*.start.frag` and `*.end.frag`
    if extension == "frag" {
        let stem = path.file_stem()?.to_str()?;
        #[expect(clippy::case_sensitive_file_extension_comparisons)]
        return (stem.ends_with(".start") || stem.ends_with(".end"))
            .then_some(SourceType::default());
    }

    None
}

// ---

#[cfg(test)]
mod tests {
    use super::*;

    fn get_parser_name(file_name: &str) -> Option<&'static str> {
        let path = Path::new(file_name);
        let extension = path.extension().and_then(|ext| ext.to_str());
        get_external_parser_name(file_name, extension)
    }

    #[test]
    fn test_get_external_parser_name() {
        let test_cases = vec![
            // JSON (NOTE: `package.json` is handled in FormatStrategyBuilder::build, not here)
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
        let source = FormatStrategyBuilder::default().build(PathBuf::from("package.json")).unwrap();
        assert!(matches!(source, FormatStrategy::ExternalFormatterPackageJson { .. }));

        let source =
            FormatStrategyBuilder::default().build(PathBuf::from("composer.json")).unwrap();
        assert!(matches!(source, FormatStrategy::ExternalFormatter { .. }));
    }

    #[test]
    fn test_toml_files() {
        // Files that should be detected as TOML
        let toml_files = vec![
            "Cargo.toml",
            "pyproject.toml",
            "config.toml",
            "config.toml.example",
            "Pipfile",
            "Cargo.toml.orig",
        ];

        for file_name in toml_files {
            let result = FormatStrategyBuilder::default().build(PathBuf::from(file_name));
            assert!(
                matches!(result, Ok(FormatStrategy::OxfmtToml { .. })),
                "`{file_name}` should be detected as TOML"
            );
        }

        // Lock files that should be excluded
        let excluded_files = vec!["Cargo.lock", "poetry.lock", "pdm.lock", "uv.lock", "Gopkg.lock"];

        for file_name in excluded_files {
            let result = FormatStrategyBuilder::default().build(PathBuf::from(file_name));
            assert!(result.is_err(), "`{file_name}` should be excluded (lock file)");
        }
    }
}
