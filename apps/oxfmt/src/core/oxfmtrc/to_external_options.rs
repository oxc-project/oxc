use serde_json::Value;

use oxc_formatter::{FormatOptions, IndentStyle, LineEnding};

use crate::core::FormatFileStrategy;

/// Syncs resolved `FormatOptions` values into the raw config JSON.
/// This ensures `external_formatter`(Prettier) receives the same options that `oxc_formatter` uses.
///
/// Only options that meet one of these criteria need to be mapped:
/// - 1. Different defaults between Prettier and oxc_formatter
///   - e.g. `printWidth`: Prettier: 80, Oxfmt: 100
/// - 2. Can be set via `.editorconfig` (values won't be in raw config JSON)
///   - `max_line_length` -> `printWidth`
///   - `end_of_line` -> `endOfLine`
///   - `indent_style` -> `useTabs`
///   - `indent_size` -> `tabWidth`
///
/// This function should be called once during config caching.
/// For strategy-specific options (plugin flags), use [`finalize_external_options()`] separately.
pub fn sync_external_options(options: &FormatOptions, config: &mut Value) {
    let Some(obj) = config.as_object_mut() else {
        return;
    };

    // vs Prettier defaults and `.editorconfig` values
    obj.insert("printWidth".to_string(), Value::from(options.line_width.value()));

    // vs `.editorconfig` values
    obj.insert(
        "useTabs".to_string(),
        Value::from(match options.indent_style {
            IndentStyle::Tab => true,
            IndentStyle::Space => false,
        }),
    );
    obj.insert("tabWidth".to_string(), Value::from(options.indent_width.value()));
    obj.insert(
        "endOfLine".to_string(),
        Value::from(match options.line_ending {
            LineEnding::Lf => "lf",
            LineEnding::Crlf => "crlf",
            LineEnding::Cr => "cr",
        }),
    );

    // Any other fields are preserved as-is.
    // - e.g. `htmlWhitespaceSensitivity`, `vueIndentScriptAndStyle`, etc.
    //   - Defined in `Oxfmtrc`, but only used by Prettier
    // - e.g. `plugins`
    //   - It does not mean plugin works correctly with Oxfmt
    //   - Oxfmt still not aware of any plugin-defined languages
    // Other options defined independently by plugins are also left as they are.
}

/// Parsers(files) that benefit from Tailwind plugin
#[cfg(feature = "napi")]
static TAILWIND_PARSERS: phf::Set<&'static str> = phf::phf_set! {
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
static OXFMT_PARSERS: phf::Set<&'static str> = phf::phf_set! {
    // "html",
    "vue",
    // "markdown",
    // "mdx",
};

/// Finalizes external options by adding plugin-specific flags based on the formatting strategy.
/// This should be called during `resolve()` after getting cached config.
///
/// - `_useTailwindPlugin`: Flag for JS side to load Tailwind plugin
/// - `_oxfmtPluginOptionsJson`: Bundled options for `prettier-plugin-oxfmt`
///
/// Also removes Prettier-unaware options to minimize payload size.
pub fn finalize_external_options(config: &mut Value, strategy: &FormatFileStrategy) {
    let Some(obj) = config.as_object_mut() else {
        return;
    };

    // Determine if Tailwind plugin should be used based on config and strategy
    let use_tailwind = obj.contains_key("sortTailwindcss")
        && match strategy {
            FormatFileStrategy::OxcFormatter { .. } => true,
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatter { parser_name, .. } => {
                TAILWIND_PARSERS.contains(parser_name)
            }
            _ => false,
        };

    // Add Tailwind plugin flag and map options
    // See: https://github.com/tailwindlabs/prettier-plugin-tailwindcss#options
    if use_tailwind {
        if let Some(tailwind) = obj.get("sortTailwindcss").and_then(|v| v.as_object()).cloned() {
            for (src, dst) in [
                ("config", "tailwindConfig"),
                ("stylesheet", "tailwindStylesheet"),
                ("functions", "tailwindFunctions"),
                ("attributes", "tailwindAttributes"),
                ("preserveWhitespace", "tailwindPreserveWhitespace"),
                ("preserveDuplicates", "tailwindPreserveDuplicates"),
            ] {
                if let Some(value) = tailwind.get(src).cloned() {
                    obj.insert(dst.to_string(), value);
                }
            }
        }
        obj.insert("_useTailwindPlugin".to_string(), Value::Number(1.into()));
    }

    // Build oxfmt plugin options JSON for js-in-xxx parsers
    #[cfg(feature = "napi")]
    if let FormatFileStrategy::ExternalFormatter { path, parser_name } = strategy
        && OXFMT_PARSERS.contains(parser_name)
    {
        let mut oxfmt_plugin_options = serde_json::Map::new();

        for key in [
            "printWidth",
            "useTabs",
            "tabWidth",
            "endOfLine",
            "singleQuote",
            "bracketSpacing",
            "bracketSameLine",
            "semi",
            "trailingComma",
            "arrowParens",
            "quoteProps",
            "jsxSingleQuote",
            "sortImports",
            "sortTailwindcss",
        ] {
            if let Some(value) = obj.get(key) {
                oxfmt_plugin_options.insert(key.to_string(), value.clone());
            }
        }

        // NOTE: Pass the parent file path so embedded JS/TS formatting
        // uses the same path for Tailwind config resolution as the parent file.
        // This is needed for ts-in-(markdown|mdx),
        // which Prettier overrides the full path with a `dummy.ts(x)`...
        // This filepath roundtrips:
        // Rust → JS (Prettier plugin options) → Rust (text_to_doc_api),
        // where it becomes `filepath_override` in `ResolvedOptions::OxcFormatter`.
        oxfmt_plugin_options
            .insert("filepath".to_string(), Value::String(path.to_string_lossy().to_string()));

        if let Ok(json_str) = serde_json::to_string(&Value::Object(oxfmt_plugin_options)) {
            obj.insert("_oxfmtPluginOptionsJson".to_string(), Value::String(json_str));
        }
    }

    // To minimize payload size, remove Prettier unaware options
    for key in [
        "sortImports",
        "sortTailwindcss",
        "sortPackageJson",
        "insertFinalNewline",
        "overrides",
        "ignorePatterns",
    ] {
        obj.remove(key);
    }
}

// ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::oxfmtrc::{FormatConfig, Oxfmtrc, to_oxfmt_options};

    #[test]
    fn test_sync_external_options_defaults() {
        let json_string = r"{}";
        let mut raw_config: Value = serde_json::from_str(json_string).unwrap();
        let config: FormatConfig = serde_json::from_str(json_string).unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();

        sync_external_options(&oxfmt_options.format_options, &mut raw_config);

        let obj = raw_config.as_object().unwrap();
        assert_eq!(obj.get("printWidth").unwrap(), 100);
    }

    #[test]
    fn test_sync_external_options_with_user_values() {
        let json_string = r#"{
            "printWidth": 80,
            "ignorePatterns": ["*.min.js"],
            "experimentalSortImports": { "order": "asc" }
        }"#;
        let mut raw_config: Value = serde_json::from_str(json_string).unwrap();
        let config: FormatConfig = serde_json::from_str(json_string).unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();

        sync_external_options(&oxfmt_options.format_options, &mut raw_config);

        let obj = raw_config.as_object().unwrap();
        // User-specified value is preserved via FormatOptions
        assert_eq!(obj.get("printWidth").unwrap(), 80);
        // oxfmt extensions are preserved (for caching)
        // They will be removed later by `finalize_external_options()`
        assert!(obj.contains_key("ignorePatterns"));
        assert!(obj.contains_key("experimentalSortImports"));
    }

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
        assert_eq!(overrides[0].files, vec!["*.test.js"]);
        assert!(overrides[0].exclude_files.is_none());
        assert_eq!(overrides[0].options.tab_width, Some(4));

        // Second override: multiple file patterns with exclude
        assert_eq!(overrides[1].files, vec!["*.md", "*.html"]);
        assert_eq!(overrides[1].exclude_files, Some(vec!["*.min.js".to_string()]));
        assert_eq!(overrides[1].options.print_width, Some(80));
    }

    #[test]
    fn test_sync_external_options_preserves_overrides() {
        let json_string = r#"{
            "tabWidth": 2,
            "overrides": [
                { "files": ["*.test.js"], "options": { "tabWidth": 4 } }
            ]
        }"#;
        let mut raw_config: Value = serde_json::from_str(json_string).unwrap();
        let oxfmtrc: Oxfmtrc = serde_json::from_str(json_string).unwrap();
        let oxfmt_options = to_oxfmt_options(oxfmtrc.format_config).unwrap();

        sync_external_options(&oxfmt_options.format_options, &mut raw_config);

        let obj = raw_config.as_object().unwrap();
        // Overrides are preserved (for caching)
        // They will be removed later by `finalize_external_options()`
        assert!(obj.contains_key("overrides"));
    }

    #[test]
    fn test_finalize_external_options_removes_oxfmt_extensions() {
        use std::path::PathBuf;

        use oxc_span::SourceType;

        let json_string = r#"{
            "tabWidth": 2,
            "overrides": [
                { "files": ["*.test.js"], "options": { "tabWidth": 4 } }
            ],
            "ignorePatterns": ["*.min.js"],
            "sortImports": { "order": "asc" }
        }"#;
        let mut raw_config: Value = serde_json::from_str(json_string).unwrap();

        let strategy = FormatFileStrategy::OxcFormatter {
            path: PathBuf::from("test.js"),
            source_type: SourceType::mjs(),
        };
        finalize_external_options(&mut raw_config, &strategy);

        let obj = raw_config.as_object().unwrap();
        // oxfmt extensions are removed by finalize_external_options
        assert!(!obj.contains_key("overrides"));
        assert!(!obj.contains_key("ignorePatterns"));
        assert!(!obj.contains_key("sortImports"));
    }

    #[test]
    #[cfg(feature = "napi")]
    fn test_finalize_external_options_sets_oxfmt_plugin_filepath() {
        use std::path::PathBuf;

        let json_string = r#"{
            "printWidth": 100,
            "singleQuote": true,
            "experimentalSortImports": { "order": "asc" }
        }"#;
        let mut raw_config: Value = serde_json::from_str(json_string).unwrap();

        let strategy = FormatFileStrategy::ExternalFormatter {
            path: PathBuf::from("/tmp/foo/bar/App.vue"),
            parser_name: "vue",
        };
        finalize_external_options(&mut raw_config, &strategy);

        let obj = raw_config.as_object().unwrap();
        let plugin_options_json = obj
            .get("_oxfmtPluginOptionsJson")
            .and_then(Value::as_str)
            .expect("Expected `_oxfmtPluginOptionsJson` to be set");

        let plugin_options: Value = serde_json::from_str(plugin_options_json).unwrap();
        let plugin_obj = plugin_options.as_object().unwrap();
        assert_eq!(
            plugin_obj.get("filepath"),
            Some(&Value::String("/tmp/foo/bar/App.vue".to_string()))
        );
    }
}
