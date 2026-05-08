use std::path::Path;

#[cfg(feature = "napi")]
use serde::Serialize;
use serde_json::Value;

use oxc_formatter::FormatOptions;

use super::super::oxfmtrc::{
    ArrowParensConfig, EmbeddedLanguageFormattingConfig, EndOfLineConfig, FormatConfig,
    HtmlWhitespaceSensitivityConfig, ObjectWrapConfig, ProseWrapConfig, QuotePropsConfig,
    SortTailwindcssUserConfig, TrailingCommaConfig,
};

/// Build base Prettier-compatible options from a typed `FormatConfig`.
///
/// Emits only Prettier-known keys.
/// - `printWidth` is always present because Prettier's default (80) differs from oxfmt's (100)
///   - We must send oxfmt's default explicitly
/// - Every other key is emitted only when the user (or `.editorconfig` via `apply_editorconfig`) set it
/// - oxfmt-specific keys are never emitted
///   - To reduce confusion and JSON size
///
/// `parser`, `filepath`, and plugin payloads are layered in via the dedicated `inject_*` helpers below.
pub fn to_prettier(config: &FormatConfig) -> Value {
    let mut obj = serde_json::Map::new();

    // `printWidth` is the one core option whose default genuinely differs
    // (Prettier 80 vs oxfmt 100), so we send oxfmt's default when unset.
    // TODO: Read the default from a neutral source (e.g. a shared `oxc_formatter_core`
    // / `ResolvedFormatConfig`) instead of the JS formatter's typed enum once available.
    obj.insert(
        "printWidth".to_string(),
        Value::from(config.print_width.unwrap_or(FormatOptions::default().line_width.value())),
    );

    // Other Prettier core options share defaults with oxfmt,
    // so we only emit when the user (or `.editorconfig` via `apply_editorconfig`) set them.
    if let Some(v) = config.use_tabs {
        obj.insert("useTabs".to_string(), Value::from(v));
    }
    if let Some(v) = config.tab_width {
        obj.insert("tabWidth".to_string(), Value::from(v));
    }
    if let Some(v) = config.end_of_line {
        obj.insert(
            "endOfLine".to_string(),
            Value::from(match v {
                EndOfLineConfig::Lf => "lf",
                EndOfLineConfig::Crlf => "crlf",
                EndOfLineConfig::Cr => "cr",
            }),
        );
    }
    if let Some(v) = config.single_quote {
        obj.insert("singleQuote".to_string(), Value::from(v));
    }
    if let Some(v) = config.jsx_single_quote {
        obj.insert("jsxSingleQuote".to_string(), Value::from(v));
    }
    if let Some(v) = config.quote_props {
        obj.insert(
            "quoteProps".to_string(),
            Value::from(match v {
                QuotePropsConfig::AsNeeded => "as-needed",
                QuotePropsConfig::Consistent => "consistent",
                QuotePropsConfig::Preserve => "preserve",
            }),
        );
    }
    if let Some(v) = config.trailing_comma {
        obj.insert(
            "trailingComma".to_string(),
            Value::from(match v {
                TrailingCommaConfig::All => "all",
                TrailingCommaConfig::Es5 => "es5",
                TrailingCommaConfig::None => "none",
            }),
        );
    }
    if let Some(v) = config.semi {
        obj.insert("semi".to_string(), Value::from(v));
    }
    if let Some(v) = config.arrow_parens {
        obj.insert(
            "arrowParens".to_string(),
            Value::from(match v {
                ArrowParensConfig::Always => "always",
                ArrowParensConfig::Avoid => "avoid",
            }),
        );
    }
    if let Some(v) = config.bracket_spacing {
        obj.insert("bracketSpacing".to_string(), Value::from(v));
    }
    if let Some(v) = config.bracket_same_line {
        obj.insert("bracketSameLine".to_string(), Value::from(v));
    }
    if let Some(v) = config.object_wrap {
        obj.insert(
            "objectWrap".to_string(),
            Value::from(match v {
                ObjectWrapConfig::Preserve => "preserve",
                ObjectWrapConfig::Collapse => "collapse",
            }),
        );
    }
    if let Some(v) = config.single_attribute_per_line {
        obj.insert("singleAttributePerLine".to_string(), Value::from(v));
    }
    if let Some(v) = config.embedded_language_formatting {
        obj.insert(
            "embeddedLanguageFormatting".to_string(),
            Value::from(match v {
                EmbeddedLanguageFormattingConfig::Auto => "auto",
                EmbeddedLanguageFormattingConfig::Off => "off",
            }),
        );
    }

    // Prettier-only options
    if let Some(v) = config.prose_wrap {
        obj.insert(
            "proseWrap".to_string(),
            Value::from(match v {
                ProseWrapConfig::Always => "always",
                ProseWrapConfig::Never => "never",
                ProseWrapConfig::Preserve => "preserve",
            }),
        );
    }
    if let Some(v) = config.html_whitespace_sensitivity {
        obj.insert(
            "htmlWhitespaceSensitivity".to_string(),
            Value::from(match v {
                HtmlWhitespaceSensitivityConfig::Css => "css",
                HtmlWhitespaceSensitivityConfig::Strict => "strict",
                HtmlWhitespaceSensitivityConfig::Ignore => "ignore",
            }),
        );
    }
    if let Some(v) = config.vue_indent_script_and_style {
        obj.insert("vueIndentScriptAndStyle".to_string(), Value::from(v));
    }

    Value::Object(obj)
}

// ---

/// Unwrap a [`Value`] produced by [`to_prettier`] back to its underlying object map.
///
/// Used by every `inject_*` helper below
/// so the invariant ("`to_prettier` returns [`Value::Object`]") lives in exactly one place.
fn as_object_mut(opts: &mut Value) -> &mut serde_json::Map<String, Value> {
    opts.as_object_mut().expect("`to_prettier` returns `Value::Object`")
}

/// Inject `parser` key.
/// Set explicitly to skip Prettier's parser inference.
pub fn inject_parser(opts: &mut Value, parser_name: &str) {
    as_object_mut(opts).insert("parser".to_string(), Value::String(parser_name.to_string()));
}

/// Inject `filepath` key.
///
/// Some plugins (Tailwind sorter, etc.) depend on it.
/// Shipped explicitly because Prettier replaces it with `dummy.{ts,tsx}`
/// for some embedded contexts (e.g., js-in-mdx).
pub fn inject_filepath(opts: &mut Value, path: &Path) {
    as_object_mut(opts)
        .insert("filepath".to_string(), Value::String(path.to_string_lossy().to_string()));
}

/// Inject Tailwind plugin keys derived from `config.sort_tailwindcss`.
///
/// No-ops when `sortTailwindcss` is disabled in config (activation check).
/// The caller gates this on capability (`supports_tailwind`).
///
/// See: <https://github.com/tailwindlabs/prettier-plugin-tailwindcss#options>
pub fn inject_tailwind_plugin_payload(opts: &mut Value, config: &FormatConfig) {
    let Some(tw) = config.sort_tailwindcss.clone().and_then(SortTailwindcssUserConfig::into_config)
    else {
        return;
    };
    let map = as_object_mut(opts);

    if let Some(v) = tw.config {
        map.insert("tailwindConfig".to_string(), Value::from(v));
    }
    if let Some(v) = tw.stylesheet {
        map.insert("tailwindStylesheet".to_string(), Value::from(v));
    }
    if let Some(v) = tw.functions {
        map.insert("tailwindFunctions".to_string(), Value::from(v));
    }
    if let Some(v) = tw.attributes {
        map.insert("tailwindAttributes".to_string(), Value::from(v));
    }
    if let Some(v) = tw.preserve_whitespace {
        map.insert("tailwindPreserveWhitespace".to_string(), Value::from(v));
    }
    if let Some(v) = tw.preserve_duplicates {
        map.insert("tailwindPreserveDuplicates".to_string(), Value::from(v));
    }
    map.insert("_useTailwindPlugin".to_string(), Value::Number(1.into()));
}

/// Inject `_oxfmtPluginOptionsJson` carrying the typed [`FormatConfig`] plus
/// the parent filepath for the embedded callback to recover.
///
/// `filepath` is shipped explicitly because Prettier replaces it with
/// `dummy.{ts,tsx}` for some embedded contexts (e.g., js-in-mdx).
///
/// The caller gates this on capability (`supports_oxfmt`).
///
/// # Panics
///
/// Panics if payload serialization fails;
/// Unreachable in practice because the payload is plainly-serializable data.
#[cfg(feature = "napi")]
pub fn inject_oxfmt_plugin_payload(opts: &mut Value, config: &FormatConfig, path: &Path) {
    #[derive(Serialize)]
    struct Payload<'a> {
        config: &'a FormatConfig,
        filepath: &'a str,
    }
    let filepath = path.to_string_lossy();
    let payload_json = serde_json::to_string(&Payload { config, filepath: &filepath })
        .expect("oxfmt plugin payload serialization should not fail");
    as_object_mut(opts).insert("_oxfmtPluginOptionsJson".to_string(), Value::String(payload_json));
}

// ---

#[cfg(test)]
mod tests_overrides_parsing {
    use oxc_config::GlobSet;

    use crate::core::oxfmtrc::Oxfmtrc;

    #[test]
    fn test_overrides_parsing() {
        let json = r#"{
            "tabWidth": 2,
            "overrides": [
                {
                    "files": ["*.test.js"],
                    "options": { "tabWidth": 4 }
                },
                {
                    "files": ["*.md", "*.html"],
                    "excludeFiles": ["*.min.js"],
                    "options": { "printWidth": 80 }
                }
            ]
        }"#;

        let config: Oxfmtrc = serde_json::from_str(json).unwrap();
        assert!(config.overrides.is_some());

        let overrides = config.overrides.unwrap();
        assert_eq!(overrides.len(), 2);

        // First override: single file pattern
        assert_eq!(overrides[0].files, GlobSet::new(["*.test.js"]));
        assert_eq!(overrides[0].exclude_files, GlobSet::default());
        assert_eq!(overrides[0].options.tab_width, Some(4));

        // Second override: multiple file patterns with exclude
        assert_eq!(overrides[1].files, GlobSet::new(["*.md", "*.html"]));
        assert_eq!(overrides[1].exclude_files, GlobSet::new(["*.min.js"]));
        assert_eq!(overrides[1].options.print_width, Some(80));
    }
}

// ---

#[cfg(test)]
mod tests_to_prettier {
    use super::*;
    use crate::core::oxfmtrc::FormatConfig;

    fn config_from(json: &str) -> FormatConfig {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn emits_only_print_width_for_empty_config() {
        let config = config_from("{}");
        let value = to_prettier(&config);
        let obj = value.as_object().unwrap();

        // `printWidth` is always emitted because oxfmt's default (100) differs
        // from Prettier's (80). The other core options share defaults and are
        // omitted when unset.
        assert_eq!(obj.get("printWidth"), Some(&Value::from(100)));
        assert!(!obj.contains_key("useTabs"));
        assert!(!obj.contains_key("tabWidth"));
        assert!(!obj.contains_key("endOfLine"));
    }

    #[test]
    fn emits_user_set_core_options() {
        let config = config_from(
            r#"{ "printWidth": 80, "useTabs": true, "tabWidth": 4, "endOfLine": "crlf" }"#,
        );
        let value = to_prettier(&config);
        let obj = value.as_object().unwrap();

        assert_eq!(obj.get("printWidth"), Some(&Value::from(80)));
        assert_eq!(obj.get("useTabs"), Some(&Value::from(true)));
        assert_eq!(obj.get("tabWidth"), Some(&Value::from(4)));
        assert_eq!(obj.get("endOfLine"), Some(&Value::from("crlf")));
    }

    #[test]
    fn omits_unset_optional_prettier_options() {
        let config = config_from("{}");
        let value = to_prettier(&config);
        let obj = value.as_object().unwrap();

        // Optional Prettier options are omitted when not set
        assert!(!obj.contains_key("singleQuote"));
        assert!(!obj.contains_key("trailingComma"));
        assert!(!obj.contains_key("semi"));
        assert!(!obj.contains_key("vueIndentScriptAndStyle"));
        assert!(!obj.contains_key("htmlWhitespaceSensitivity"));
    }

    #[test]
    fn emits_user_set_optional_prettier_options() {
        let config = config_from(
            r#"{
                "singleQuote": true,
                "trailingComma": "es5",
                "semi": false,
                "arrowParens": "avoid",
                "quoteProps": "consistent",
                "objectWrap": "collapse",
                "embeddedLanguageFormatting": "off",
                "proseWrap": "always",
                "htmlWhitespaceSensitivity": "ignore",
                "vueIndentScriptAndStyle": true
            }"#,
        );
        let value = to_prettier(&config);
        let obj = value.as_object().unwrap();

        assert_eq!(obj.get("singleQuote"), Some(&Value::from(true)));
        assert_eq!(obj.get("trailingComma"), Some(&Value::from("es5")));
        assert_eq!(obj.get("semi"), Some(&Value::from(false)));
        assert_eq!(obj.get("arrowParens"), Some(&Value::from("avoid")));
        assert_eq!(obj.get("quoteProps"), Some(&Value::from("consistent")));
        assert_eq!(obj.get("objectWrap"), Some(&Value::from("collapse")));
        assert_eq!(obj.get("embeddedLanguageFormatting"), Some(&Value::from("off")));
        assert_eq!(obj.get("proseWrap"), Some(&Value::from("always")));
        assert_eq!(obj.get("htmlWhitespaceSensitivity"), Some(&Value::from("ignore")));
        assert_eq!(obj.get("vueIndentScriptAndStyle"), Some(&Value::from(true)));
    }

    #[test]
    fn never_emits_oxfmt_specific_keys() {
        // Even with all oxfmt-specific options set, none should appear in the Prettier options
        let config = config_from(
            r#"{
                "insertFinalNewline": false,
                "sortImports": true,
                "sortPackageJson": true,
                "sortTailwindcss": true,
                "jsdoc": true
            }"#,
        );
        let value = to_prettier(&config);
        let obj = value.as_object().unwrap();

        for key in [
            "insertFinalNewline",
            "sortImports",
            "experimentalSortImports",
            "sortPackageJson",
            "experimentalSortPackageJson",
            "sortTailwindcss",
            "experimentalTailwindcss",
            "jsdoc",
            "overrides",
            "ignorePatterns",
            "experimentalOperatorPosition",
            "experimentalTernaries",
        ] {
            assert!(!obj.contains_key(key), "Key `{key}` must NOT be in Prettier options");
        }
    }

    #[test]
    fn to_prettier_never_injects_tailwind_keys() {
        // Tailwind keys are always opt-in via `inject_tailwind_plugin_payload`.
        let config = config_from(r#"{ "sortTailwindcss": true }"#);
        let value = to_prettier(&config);
        let obj = value.as_object().unwrap();

        assert!(!obj.contains_key("_useTailwindPlugin"));
        assert!(!obj.contains_key("tailwindConfig"));
    }

    #[test]
    fn does_not_inject_filepath_or_plugin_options_json() {
        // These belong to the format step, not to_prettier
        let config = config_from(r#"{ "printWidth": 80 }"#);
        let value = to_prettier(&config);
        let obj = value.as_object().unwrap();

        assert!(!obj.contains_key("filepath"));
        assert!(!obj.contains_key("parser"));
        assert!(!obj.contains_key("_oxfmtPluginOptionsJson"));
    }
}
