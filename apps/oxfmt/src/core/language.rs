//! User-facing language identifiers, mapped to their formatter in one place.
//!
//! Each language is named by its Shiki id (what Markdown code fences use, <https://shiki.style/languages>).
//! The aliases used by embedded code, the LSP `languageId`s and the fake path extension all live here,
//! so adding a language never needs syncing several tables.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_formatter_css::CssVariant;
use oxc_formatter_json::JsonVariant;
use oxc_span::{FileExtension, SourceType};

use super::support::{NativeLanguage, PrettierLanguage};

/// "Which formatter serves this language?" for a name as written (e.g. a code fence),
/// via the [`Language`] table plus the JS/TS extension fallback.
/// [`build_dispatcher`](super::embed::dispatcher::build_dispatcher) and the napi string channel's fence routing both consult it,
/// so their notions of who formats what can never drift.
pub fn route_embedded(name: &str) -> Route {
    if let Some(language) = Language::from_embedded(name) {
        return language.route();
    }
    // JS / TS extensions carrying a module kind (`mjs`, `cjs`, `mts`, `cts`) keep it.
    SourceType::from_extension(name)
        .map_or(Route::Unsupported, |source_type| Route::Native(NativeLanguage::Js(source_type)))
}

/// Where a language identifier routes.
pub enum Route {
    /// A Rust formatter branch in [`build_dispatcher`](super::embed::dispatcher::build_dispatcher);
    /// never re-routed to Prettier.
    Native(NativeLanguage),
    /// Prettier serves it (napi Doc→IR fallback / string channel);
    /// the pure build preserves it as-is.
    Prettier(PrettierLanguage),
    /// No formatter anywhere: the part deliberately stays as-is in every build.
    Unsupported,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    Javascript,
    Jsx,
    Typescript,
    Tsx,
    Json,
    Jsonc,
    Json5,
    Css,
    Scss,
    Less,
    Graphql,
    Yaml,
    Markdown,
    Toml,
    Html,
    AngularHtml,
    Vue,
    Svelte,
    Astro,
    Handlebars,
    Mdx,
}

impl Language {
    /// Resolve the name of an embedded language (Markdown code fences, xxx-in-js, front matter, ...):
    /// the Shiki id of each language, plus the aliases listed here.
    ///
    /// JS / TS extensions carrying a module kind (`mjs`, `cjs`, `mts`, `cts`) are NOT resolved here,
    /// [`route_embedded`] maps them to a `SourceType` directly to keep the module kind.
    pub fn from_embedded(name: &str) -> Option<Self> {
        Some(match name {
            "javascript" | "js" => Self::Javascript,
            "jsx" => Self::Jsx,
            // `angular-ts` is listed because Shiki defines it, but nothing special is done for it:
            // it is plain TypeScript, `oxc_formatter` finds a component's inline template from its `@Component` decorator.
            "typescript" | "ts" | "angular-ts" => Self::Typescript,
            "tsx" => Self::Tsx,
            "json" => Self::Json,
            "jsonc" => Self::Jsonc,
            "json5" => Self::Json5,
            "css" | "postcss" => Self::Css,
            "scss" => Self::Scss,
            "less" => Self::Less,
            "graphql" | "gql" => Self::Graphql,
            "yaml" | "yml" => Self::Yaml,
            "markdown" | "md" => Self::Markdown,
            "toml" => Self::Toml,
            "html" => Self::Html,
            // NOTE: `angular-html` is the Shiki id, but `angular` is not.
            // It is not known to GitHub's linguist (so not highlighted there), and rarely written.
            // Kept only for Prettier compat, which resolves a fence by its `parser` name.
            "angular-html" | "angular" => Self::AngularHtml,
            "vue" => Self::Vue,
            "svelte" => Self::Svelte,
            "astro" => Self::Astro,
            "handlebars" | "hbs" => Self::Handlebars,
            "mdx" => Self::Mdx,
            _ => return None,
        })
    }

    /// Resolve an LSP `languageId`: VS Code's ids, which match the embedded names except for the React ones.
    /// `angular` is not a VS Code id, accepted for editors naming Angular templates so (#20242).
    #[cfg(feature = "napi")]
    pub fn from_lsp_id(id: &str) -> Option<Self> {
        match id {
            "javascriptreact" => Some(Self::Jsx),
            "typescriptreact" => Some(Self::Tsx),
            _ => Self::from_embedded(id),
        }
    }

    /// LSP only: the extension of the fake path for an in-memory document,
    /// so that config scopes, `ignorePatterns` and `overrides` see it as a file of this language.
    #[cfg(feature = "napi")]
    pub fn to_lsp_extension(self) -> &'static str {
        match self {
            Self::Javascript => "js",
            Self::Jsx => "jsx",
            Self::Typescript => "ts",
            Self::Tsx => "tsx",
            Self::Json => "json",
            Self::Jsonc => "jsonc",
            Self::Json5 => "json5",
            Self::Css => "css",
            Self::Scss => "scss",
            Self::Less => "less",
            Self::Graphql => "graphql",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Markdown => "md",
            Self::Mdx => "mdx",
            Self::Html => "html",
            Self::AngularHtml => "component.html",
            Self::Vue => "vue",
            Self::Svelte => "svelte",
            Self::Astro => "astro",
            Self::Handlebars => "handlebars",
        }
    }

    /// Which formatter serves this language.
    pub fn route(self) -> Route {
        match self {
            Self::Javascript => Route::Native(NativeLanguage::Js(FileExtension::Js.into())),
            Self::Jsx => Route::Native(NativeLanguage::Js(FileExtension::Jsx.into())),
            Self::Typescript => Route::Native(NativeLanguage::Js(FileExtension::Ts.into())),
            Self::Tsx => Route::Native(NativeLanguage::Js(FileExtension::Tsx.into())),
            Self::Json => Route::Native(NativeLanguage::Json(JsonVariant::Json)),
            Self::Jsonc => Route::Native(NativeLanguage::Json(JsonVariant::Jsonc)),
            Self::Json5 => Route::Native(NativeLanguage::Json(JsonVariant::Json5)),
            Self::Css => Route::Native(NativeLanguage::Css(CssVariant::Css)),
            Self::Scss => Route::Native(NativeLanguage::Css(CssVariant::Scss)),
            Self::Less => Route::Native(NativeLanguage::Css(CssVariant::Less)),
            Self::Graphql => Route::Native(NativeLanguage::Graphql),
            Self::Yaml => Route::Native(NativeLanguage::Yaml),
            Self::Markdown => Route::Native(NativeLanguage::Markdown),
            Self::Toml => Route::Native(NativeLanguage::Toml),
            Self::Html => Route::Prettier(PrettierLanguage::Html),
            Self::AngularHtml => Route::Prettier(PrettierLanguage::Angular),
            Self::Vue => Route::Prettier(PrettierLanguage::Vue),
            Self::Svelte => Route::Prettier(PrettierLanguage::Svelte),
            Self::Astro => Route::Prettier(PrettierLanguage::Astro),
            Self::Handlebars => Route::Prettier(PrettierLanguage::Glimmer),
            Self::Mdx => Route::Prettier(PrettierLanguage::Mdx),
        }
    }
}

#[cfg(all(test, feature = "napi"))]
mod tests {
    use super::*;

    /// The config ids (serde names) are the canonical embedded names, so both stay one vocabulary.
    #[test]
    fn serde_names_are_embedded_names() {
        let schema = serde_json::to_value(schemars::schema_for!(Language)).unwrap();
        let names = schema["enum"].as_array().expect("`Language` is a plain string enum");
        for name in names {
            let name = name.as_str().unwrap();
            let language = Language::from_embedded(name)
                .unwrap_or_else(|| panic!("`{name}` is not an embedded name"));
            assert_eq!(serde_json::to_value(language).unwrap(), name);
        }
    }

    /// Prettier-served fences (native ones are covered by `every_native_language_dispatches`)
    /// and the LSP ids that differ from embedded names or extensions.
    #[test]
    fn prettier_fences_and_lsp_ids() {
        for (fence, parser) in [
            ("html", "html"),
            ("angular", "angular"),
            ("angular-html", "angular"),
            ("vue", "vue"),
            ("svelte", "svelte"),
            ("astro", "astro"),
            ("handlebars", "glimmer"),
            ("hbs", "glimmer"),
            ("mdx", "mdx"),
        ] {
            let Some(Route::Prettier(language)) =
                Language::from_embedded(fence).map(Language::route)
            else {
                panic!("`{fence}` should route to Prettier");
            };
            assert_eq!(language.parser(), parser, "`{fence}`");
        }

        for (id, extension) in [
            ("javascript", "js"),
            ("javascriptreact", "jsx"),
            ("typescriptreact", "tsx"),
            ("markdown", "md"),
            ("angular", "component.html"),
        ] {
            assert_eq!(
                Language::from_lsp_id(id).map(Language::to_lsp_extension),
                Some(extension),
                "`{id}`"
            );
        }
    }
}
