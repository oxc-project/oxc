use std::path::Path;

use phf::phf_set;

use oxc_formatter_css::CssVariant;
use oxc_formatter_json::JsonVariant;
use oxc_span::SourceType;

use super::language::{Language, Route};
#[cfg(feature = "napi")]
use super::oxfmtrc::FormatConfig;

/// Classify a file path into a [`FormatStrategy`].
///
/// Returns `None` when the file type is not a formatting target.
pub fn classify_file(path: &Path) -> Option<FormatStrategy> {
    // PERF: Standard JS/TS extensions are by far the most common case, so resolve them first.
    // This relies on `EXCLUDE_FILENAMES` containing no JS/TS file, see `exclude_filenames_are_not_js_or_ts` test.
    if let Ok(source_type) = SourceType::from_path(path) {
        return Some(FormatStrategy::Native(NativeLanguage::Js(source_type)));
    }

    let file_name = path.file_name()?.to_str()?;
    // Machine-generated files like lock files must NEVER be formatted,
    // regardless of how they reach us (CLI, API, LSP, etc).
    if EXCLUDE_FILENAMES.contains(file_name) {
        return None;
    }
    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or_default();

    if is_extra_js_file(file_name, ext) {
        return Some(FormatStrategy::Native(NativeLanguage::Js(SourceType::default())));
    }
    if TOML_FILENAMES.contains(file_name) || ext == "toml" || file_name.ends_with(".toml.example") {
        return Some(FormatStrategy::Native(NativeLanguage::Toml));
    }
    if file_name == "package.json" {
        return Some(FormatStrategy::PackageJson);
    }
    if let Some(variant) = json_variant(file_name, ext) {
        return Some(FormatStrategy::Native(NativeLanguage::Json(variant)));
    }
    if GRAPHQL_EXTENSIONS.contains(ext) {
        return Some(FormatStrategy::Native(NativeLanguage::Graphql));
    }
    if let Some(variant) = css_variant(ext) {
        return Some(FormatStrategy::Native(NativeLanguage::Css(variant)));
    }
    // Before the generic YAML check, since Prettier tries to format them as JSON first
    if YAML_RC_FILENAMES.contains(file_name) {
        return Some(FormatStrategy::YamlRc);
    }
    if YAML_FILENAMES.contains(file_name) || YAML_EXTENSIONS.contains(ext) {
        return Some(FormatStrategy::Native(NativeLanguage::Yaml));
    }
    if MARKDOWN_FILENAMES.contains(file_name) || MARKDOWN_EXTENSIONS.contains(ext) {
        return Some(FormatStrategy::Native(NativeLanguage::Markdown));
    }

    #[cfg(feature = "napi")]
    if let Some(language) = prettier_language(file_name, ext) {
        return Some(FormatStrategy::Prettier(language));
    }

    None
}

/// Classify a file explicitly assigned to `language` (e.g. by `associations`), regardless of its name.
///
/// Still `None` for excluded files (lock files), and in the pure Rust build for languages Prettier serves.
pub fn classify_as(language: Language, path: &Path) -> Option<FormatStrategy> {
    // Lock files must NEVER be formatted, even when explicitly assigned
    // (e.g. `pnpm-lock.yaml` matched by a broad `associations` glob).
    // This bypasses `classify_file`, so the same rule is checked here.
    if EXCLUDE_FILENAMES.contains(path.file_name()?.to_str()?) {
        return None;
    }
    match language.route() {
        Route::Native(language) => Some(FormatStrategy::Native(language)),
        #[cfg(feature = "napi")]
        Route::Prettier(language) => Some(FormatStrategy::Prettier(language)),
        _ => None,
    }
}

/// How a whole file is formatted: which formatter, with any pre-process.
/// The whole-file counterpart of [`Route`] for embedded parts,
/// with `PackageJson` / `YamlRc` as file-only pre-processes.
///
/// Consumed by the resolver to construct a [`super::FormatPlan`] with the resolved config.
#[derive(Debug)]
pub enum FormatStrategy {
    /// Files formatted by a Rust formatter (`oxc_formatter_*`).
    Native(NativeLanguage),
    /// `package.json`: sorted by `sort-package-json`,
    /// then formatted by `oxc_formatter_json` with the `json-stringify` variant.
    PackageJson,
    /// Files like `.prettierrc`: mirroring Prettier's yaml embed,
    /// formatted as JSON first, then as YAML if that fails.
    YamlRc,
    /// Files formatted by delegating to Prettier (Tier 3/4).
    #[cfg(feature = "napi")]
    Prettier(PrettierLanguage),
}

impl FormatStrategy {
    /// Label for tracing spans and logs, the Prettier parser name for `Prettier`.
    pub fn trace_label(&self) -> &'static str {
        match self {
            Self::Native(language) => language.trace_label(),
            Self::PackageJson => "package_json",
            Self::YamlRc => "yaml_rc",
            #[cfg(feature = "napi")]
            Self::Prettier(language) => language.parser(),
        }
    }
}

/// Languages formatted by a Rust formatter (`oxc_formatter_*`),
/// both embedded parts ([`route_embedded`](super::language::route_embedded)) and whole files ([`FormatStrategy::Native`]).
#[derive(Debug)]
pub enum NativeLanguage {
    Js(SourceType),
    Graphql,
    /// The fence-derived variant;
    /// the css-in-js typed context overrides it to Scss + placeholders at dispatch time (see the css branch).
    Css(CssVariant),
    Yaml,
    Json(JsonVariant),
    Markdown,
    Toml,
}

impl NativeLanguage {
    /// Label for tracing spans and logs.
    pub fn trace_label(&self) -> &'static str {
        match self {
            Self::Js(_) => "js",
            Self::Graphql => "graphql",
            Self::Css(_) => "css",
            Self::Yaml => "yaml",
            Self::Json(_) => "json",
            Self::Markdown => "markdown",
            Self::Toml => "toml",
        }
    }
}

/// Languages Prettier still formats for us (no Rust formatter yet),
/// both embedded parts ([`route_embedded`](super::language::route_embedded)) and whole files ([`FormatStrategy::Prettier`]).
///
/// The Prettier paths receive this instead of a raw string,
/// so they can never be handed an unknown language.
/// The set shrinks as Rust ports land, and the type disappears with the last port.
#[derive(Debug, Clone, Copy)]
pub enum PrettierLanguage {
    Mdx,
    Html,
    Angular,
    Vue,
    /// Formatted only when `prettier-plugin-svelte` is enabled (`svelte` config key).
    Svelte,
    /// Formatted only when `prettier-plugin-astro` is enabled (`astro` config key).
    Astro,
    /// Whole files only: [`route_embedded`](super::language::route_embedded) never returns it.
    #[cfg(feature = "napi")]
    Mjml,
    /// Handlebars files (`.hbs` / `.handlebars`), following Prettier's convention of formatting them with its `glimmer` parser.
    /// They are parsed as classic Ember (Glimmer) templates,
    /// so loose Handlebars outside that subset (e.g. partials `{{> name}}`) is not supported.
    /// Glimmer's template tag components (`.gjs` / `.gts`) are not supported either.
    Glimmer,
}

#[cfg(feature = "napi")]
impl PrettierLanguage {
    /// The Prettier `parser` name injected into the options JSON.
    pub fn parser(self) -> &'static str {
        match self {
            Self::Mdx => "mdx",
            Self::Html => "html",
            Self::Angular => "angular",
            Self::Vue => "vue",
            Self::Svelte => "svelte",
            Self::Astro => "astro",
            Self::Mjml => "mjml",
            Self::Glimmer => "glimmer",
        }
    }

    /// The config key of the opt-in plugin this language requires, when `config` does NOT enable it.
    ///
    /// `svelte` / `astro` cannot be formatted without `prettier-plugin-svelte` / `prettier-plugin-astro`,
    /// which are enabled by the `svelte` / `astro` config keys.
    pub fn missing_plugin(self, config: &FormatConfig) -> Option<&'static str> {
        match self {
            Self::Svelte if !config.is_svelte_enabled() => Some("svelte"),
            Self::Astro if !config.is_astro_enabled() => Some("astro"),
            _ => None,
        }
    }

    /// Whether the Doc→IR conversion must surface `HtmlEmbedMeta`
    /// (`htmlHasMultipleRootElements`) to the embed site.
    pub fn wants_html_meta(self) -> bool {
        matches!(self, Self::Html | Self::Angular)
    }
}

// ---

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

static TOML_FILENAMES: phf::Set<&'static str> = phf_set! {
    "Pipfile",
    "Cargo.toml.orig",
};

// ---

fn json_variant(file_name: &str, ext: &str) -> Option<JsonVariant> {
    if file_name == "composer.json" || ext == "importmap" {
        return Some(JsonVariant::JsonStringify);
    }
    if JSON_FILENAMES.contains(file_name)
        || JSON_EXTENSIONS.contains(ext)
        || file_name.ends_with(".json.example")
        || file_name.ends_with(".tfstate.backup")
    {
        return Some(JsonVariant::Json);
    }
    if JSONC_EXTENSIONS.contains(ext) {
        return Some(JsonVariant::Jsonc);
    }
    if ext == "json5" {
        return Some(JsonVariant::Json5);
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
    "mcmeta",
    "sarif",
    "tact",
    "tfstate",
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

// ---

static GRAPHQL_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "graphql",
    "gql",
    "graphqls",
};

// ---

fn css_variant(ext: &str) -> Option<CssVariant> {
    match ext {
        "css" | "wxss" | "pcss" | "postcss" => Some(CssVariant::Css),
        "scss" => Some(CssVariant::Scss),
        "less" => Some(CssVariant::Less),
        _ => None,
    }
}

// ---

static YAML_RC_FILENAMES: phf::Set<&'static str> = phf_set! {
    ".prettierrc",
    ".stylelintrc",
    ".lintstagedrc",
};

static YAML_FILENAMES: phf::Set<&'static str> = phf_set! {
    ".clang-format",
    ".clang-tidy",
    ".clangd",
    ".gemrc",
    "CITATION.cff",
    "glide.lock",
    "pixi.lock",
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

// ---

/// Returns the [`PrettierLanguage`] for the file, if supported.
/// See also `prettier --support-info | jq '.languages[]'`
#[cfg(feature = "napi")]
fn prettier_language(file_name: &str, ext: &str) -> Option<PrettierLanguage> {
    if file_name.ends_with(".component.html") {
        return Some(PrettierLanguage::Angular);
    }
    Some(match ext {
        "html" | "hta" | "htm" | "inc" | "xht" | "xhtml" => PrettierLanguage::Html,
        "vue" => PrettierLanguage::Vue,
        // Formatting is gated by `ResolveOutcome::MissingPlugin` (requires `svelte` / `astro` config),
        // classified here so that each caller can surface a friendly error or skip.
        "svelte" => PrettierLanguage::Svelte,
        "astro" => PrettierLanguage::Astro,
        "mdx" => PrettierLanguage::Mdx,
        "mjml" => PrettierLanguage::Mjml,
        "handlebars" | "hbs" => PrettierLanguage::Glimmer,
        _ => return None,
    })
}

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

static SPECIAL_JS_FILENAMES: phf::Set<&'static str> = phf_set! {
    "Jakefile",
    "start.frag",
    "end.frag",
};

/// Non-standard JS files that `SourceType::from_path` does not recognize, but Prettier does.
fn is_extra_js_file(file_name: &str, ext: &str) -> bool {
    SPECIAL_JS_FILENAMES.contains(file_name)
        || ADDITIONAL_JS_EXTENSIONS.contains(ext)
        || file_name.ends_with(".start.frag")
        || file_name.ends_with(".end.frag")
}

// ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclude_filenames_are_not_js_or_ts() {
        // `classify_file` resolves standard JS/TS extensions (via `SourceType::from_path`)
        // before checking `EXCLUDE_FILENAMES` for perf,
        // so an excluded file with a JS/TS extension would bypass the exclusion entirely.
        for name in &EXCLUDE_FILENAMES {
            assert!(
                SourceType::from_path(Path::new(name)).is_err(),
                "`{name}` in EXCLUDE_FILENAMES must not be a standard JS/TS file, \
                 otherwise it bypasses the exclusion check in `classify_file`"
            );
        }
    }

    #[test]
    fn test_js_or_ts_files() {
        // Standard extensions, special filenames, and Prettier's additional extensions.
        let js_or_ts_files = vec![
            // Standard (via `SourceType::from_path`)
            "index.js",
            "app.tsx",
            "types.d.ts",
            "module.mjs",
            // Special filenames
            "Jakefile",
            "start.frag",
            "end.frag",
            // Additional extensions
            "legacy.es6",
            "script._js",
            // `.frag` is only valid as `*.start.frag` / `*.end.frag`
            "shader.start.frag",
            "shader.end.frag",
        ];
        for file_name in js_or_ts_files {
            let result = classify_file(Path::new(file_name));
            assert!(
                matches!(result, Some(FormatStrategy::Native(NativeLanguage::Js(_)))),
                "`{file_name}` should be routed to oxc_formatter"
            );
        }

        // Plain `.frag` files (not `*.start.frag` / `*.end.frag`) are not JS.
        for file_name in ["shader.frag", "random.frag", "xstart.frag"] {
            let result = classify_file(Path::new(file_name));
            assert!(
                !matches!(result, Some(FormatStrategy::Native(NativeLanguage::Js(_)))),
                "`{file_name}` should NOT be routed to oxc_formatter"
            );
        }
    }

    #[test]
    #[cfg(feature = "napi")]
    fn test_prettier_language() {
        fn get_parser_name(file_name: &str) -> Option<&'static str> {
            let ext = Path::new(file_name).extension().and_then(|ext| ext.to_str());
            prettier_language(file_name, ext.unwrap_or_default()).map(PrettierLanguage::parser)
        }

        let test_cases = vec![
            // JSON variants (e.g. `data.json`, `package.json`, `config.importmap`) are
            // all routed to `oxc_formatter_json` in `classify_file` and excluded from this map.
            ("package.json", None),
            ("composer.json", None),
            ("config.importmap", None),
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
            // CSS files are routed to `oxc_formatter_css` in `classify_file`
            // and excluded from this map.
            ("styles.css", None),
            ("theme.less", None),
            ("main.scss", None),
            // GraphQL files are routed to `oxc_formatter_graphql` in `classify_file`
            // and excluded from this map.
            ("schema.graphql", None),
            ("query.gql", None),
            ("types.graphqls", None),
            // Handlebars
            ("template.handlebars", Some("glimmer")),
            ("partial.hbs", Some("glimmer")),
            // Markdown files are routed to `oxc_formatter_markdown` in `classify_file`
            // and excluded from this map.
            ("README", None),
            ("contents.lr", None),
            ("docs.md", None),
            ("guide.markdown", None),
            ("notes.mdown", None),
            // MDX
            ("page.mdx", Some("mdx")),
            // YAML files are routed to `oxc_formatter_yaml` in `classify_file`
            // and excluded from this map.
            (".clang-format", None),
            (".prettierrc", None),
            ("config.yml", None),
            ("settings.yaml", None),
            ("grammar.sublime-syntax", None),
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
    fn test_json_files_route_to_oxc_formatter_json() {
        let test_cases = vec![
            // JSON_EXTENSIONS
            ("data.json", JsonVariant::Json),
            ("config.webmanifest", JsonVariant::Json),
            ("infra.tfstate", JsonVariant::Json),
            // Compound extensions (`Path::extension()` only sees the last segment)
            ("config.json.example", JsonVariant::Json),
            ("infra.tfstate.backup", JsonVariant::Json),
            // JSON_FILENAMES
            (".babelrc", JsonVariant::Json),
            (".eslintrc.json", JsonVariant::Json),
            // tsconfig (handled via standard `.json` extension)
            ("tsconfig.json", JsonVariant::Json),
            // JSONC_EXTENSIONS
            ("settings.jsonc", JsonVariant::Jsonc),
            ("project.code-workspace", JsonVariant::Jsonc),
            // JSON5
            ("settings.json5", JsonVariant::Json5),
            // `JSON.stringify`-style files
            ("composer.json", JsonVariant::JsonStringify),
            ("config.importmap", JsonVariant::JsonStringify),
        ];

        for (file_name, expected) in test_cases {
            let result = classify_file(Path::new(file_name));
            assert!(
                matches!(result, Some(FormatStrategy::Native(NativeLanguage::Json(variant))) if variant == expected),
                "`{file_name}` should be routed to oxc_formatter_json ({expected:?})"
            );
        }

        // `package.json` also uses the `json-stringify` variant,
        // but is the lone dedicated strategy for the sorting pre-process
        let strategy = classify_file(Path::new("package.json")).unwrap();
        assert!(matches!(strategy, FormatStrategy::PackageJson));
    }

    #[test]
    fn test_graphql_files_route_to_oxc_formatter_graphql() {
        for file_name in ["schema.graphql", "query.gql", "types.graphqls"] {
            let result = classify_file(Path::new(file_name));
            assert!(
                matches!(result, Some(FormatStrategy::Native(NativeLanguage::Graphql))),
                "`{file_name}` should be routed to oxc_formatter_graphql"
            );
        }
    }

    #[test]
    fn test_css_files_route_to_oxc_formatter_css() {
        let test_cases = vec![
            ("styles.css", CssVariant::Css),
            ("app.wxss", CssVariant::Css),
            ("styles.pcss", CssVariant::Css),
            ("styles.postcss", CssVariant::Css),
            ("main.scss", CssVariant::Scss),
            ("theme.less", CssVariant::Less),
        ];

        for (file_name, expected) in test_cases {
            let result = classify_file(Path::new(file_name));
            assert!(
                matches!(result, Some(FormatStrategy::Native(NativeLanguage::Css(variant))) if variant == expected),
                "`{file_name}` should be routed to oxc_formatter_css ({expected:?})"
            );
        }
    }

    #[test]
    fn test_yaml_files_route_to_oxc_formatter_yaml() {
        // YAML_EXTENSIONS and YAML_FILENAMES
        for file_name in [
            "config.yml",
            "settings.yaml",
            "grammar.sublime-syntax",
            ".clang-format",
            "CITATION.cff",
        ] {
            let result = classify_file(Path::new(file_name));
            assert!(
                matches!(result, Some(FormatStrategy::Native(NativeLanguage::Yaml))),
                "`{file_name}` should be routed to oxc_formatter_yaml"
            );
        }

        // rc files Prettier tries as JSON first
        for file_name in [".prettierrc", ".stylelintrc", ".lintstagedrc"] {
            let result = classify_file(Path::new(file_name));
            assert!(
                matches!(result, Some(FormatStrategy::YamlRc)),
                "`{file_name}` should be routed to the JSON-first YAML rc strategy"
            );
        }

        // YAML lock files are excluded, not formatted
        let result = classify_file(Path::new("pnpm-lock.yaml"));
        assert!(result.is_none(), "`pnpm-lock.yaml` should be excluded");
    }

    #[test]
    fn test_markdown_files_route_to_oxc_formatter_markdown() {
        // MARKDOWN_EXTENSIONS and MARKDOWN_FILENAMES
        for file_name in ["docs.md", "guide.markdown", "notes.mdown", "README", "contents.lr"] {
            let result = classify_file(Path::new(file_name));
            assert!(
                matches!(result, Some(FormatStrategy::Native(NativeLanguage::Markdown))),
                "`{file_name}` should be routed to oxc_formatter_markdown"
            );
        }
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
            let result = classify_file(Path::new(file_name));
            assert!(
                matches!(result, Some(FormatStrategy::Native(NativeLanguage::Toml))),
                "`{file_name}` should be detected as TOML"
            );
        }

        // Lock files that should be excluded
        let excluded_files = vec!["Cargo.lock", "poetry.lock", "pdm.lock", "uv.lock", "Gopkg.lock"];

        for file_name in excluded_files {
            let result = classify_file(Path::new(file_name));
            assert!(result.is_none(), "`{file_name}` should be excluded (lock file)");
        }
    }
}
