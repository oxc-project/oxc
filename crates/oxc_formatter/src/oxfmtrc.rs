//! This file handles the `Oxfmtrc` struct, which ideally should be defined under `apps/oxfmt`.
//!
//! The reason it is not done so at this time is that `oxc_language_server` uses this struct,
//! and `apps/oxfmt` also depends on `oxc_language_server`, creating a circular reference.
//!
//! While it is possible to define a separate crate for `Oxfmtrc`, we compromise with this method for now.

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{
    ArrowParentheses, AttributePosition, BracketSameLine, BracketSpacing,
    EmbeddedLanguageFormatting, Expand, FormatOptions, IndentStyle, IndentWidth, LineEnding,
    LineWidth, QuoteProperties, QuoteStyle, Semicolons, SortImportsOptions, SortOrder,
    TailwindcssOptions, TrailingCommas,
};

/// Configuration options for the Oxfmt.
///
/// Most options are the same as Prettier's options.
/// See also <https://prettier.io/docs/options>
///
/// In addition, some options are our own extensions.
// NOTE: All fields are typed as `Option` to distinguish between user-specified values and defaults.
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Oxfmtrc {
    /// Use tabs for indentation or spaces. (Default: `false`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_tabs: Option<bool>,
    /// Number of spaces per indentation level. (Default: `2`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_width: Option<u8>,
    /// Which end of line characters to apply. (Default: `"lf"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_of_line: Option<EndOfLineConfig>,
    /// The line length that the printer will wrap on. (Default: `100`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_width: Option<u16>,
    /// Use single quotes instead of double quotes. (Default: `false`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_quote: Option<bool>,
    /// Use single quotes instead of double quotes in JSX. (Default: `false`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_single_quote: Option<bool>,
    /// Change when properties in objects are quoted. (Default: `"as-needed"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_props: Option<QuotePropsConfig>,
    /// Print trailing commas wherever possible. (Default: `"all"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_comma: Option<TrailingCommaConfig>,
    /// Print semicolons at the ends of statements. (Default: `true`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semi: Option<bool>,
    /// Include parentheses around a sole arrow function parameter. (Default: `"always"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrow_parens: Option<ArrowParensConfig>,
    /// Print spaces between brackets in object literals. (Default: `true`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_spacing: Option<bool>,
    /// Put the `>` of a multi-line JSX element at the end of the last line
    /// instead of being alone on the next line. (Default: `false`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_same_line: Option<bool>,
    /// How to wrap object literals when they could fit on one line or span multiple lines. (Default: `"preserve"`)
    /// NOTE: In addition to Prettier's `"preserve"` and `"collapse"`, we also support `"always"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_wrap: Option<ObjectWrapConfig>,
    /// Put each attribute on a new line in JSX. (Default: `false`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_attribute_per_line: Option<bool>,

    // NOTE: These experimental options are not yet supported.
    // Just be here to report error if they are used.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(skip)]
    pub experimental_operator_position: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(skip)]
    pub experimental_ternaries: Option<bool>,

    /// Control whether to format embedded parts in the file. (Default: `"off"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedded_language_formatting: Option<EmbeddedLanguageFormattingConfig>,

    /// Whether to insert a final newline at the end of the file. (Default: `true`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_final_newline: Option<bool>,

    /// Experimental: Sort import statements. Disabled by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_sort_imports: Option<SortImportsConfig>,

    /// Experimental: Sort `package.json` keys. (Default: `true`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_sort_package_json: Option<bool>,

    /// Experimental: Enable Tailwind CSS class sorting in JSX class/className attributes.
    /// When enabled, class strings will be collected and passed to a callback for sorting.
    /// Pass `true` or an object with options from `prettier-plugin-tailwindcss`.
    /// (Default: disabled)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub experimental_tailwindcss: Option<TailwindcssConfig>,

    /// Ignore files matching these glob patterns. Current working directory is used as the root.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_patterns: Option<Vec<String>>,
}

// ---

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EndOfLineConfig {
    Lf,
    Crlf,
    Cr,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum QuotePropsConfig {
    AsNeeded,
    Consistent,
    Preserve,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TrailingCommaConfig {
    All,
    Es5,
    None,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ArrowParensConfig {
    Always,
    Avoid,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ObjectWrapConfig {
    Preserve,
    Collapse,
    Always,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddedLanguageFormattingConfig {
    Auto,
    Off,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SortImportsConfig {
    /// Partition imports by newlines. (Default: `false`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_by_newline: Option<bool>,
    /// Partition imports by comments. (Default: `false`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_by_comment: Option<bool>,
    /// Sort side-effect imports. (Default: `false`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_side_effects: Option<bool>,
    /// Sort order. (Default: `"asc"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrderConfig>,
    /// Ignore case when sorting. (Default: `true`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_case: Option<bool>,
    /// Add newlines between import groups. (Default: `true`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newlines_between: Option<bool>,
    /// Glob patterns to identify internal imports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_pattern: Option<Vec<String>>,
    /// Custom groups configuration for organizing imports.
    /// Each array element represents a group, and multiple group names in the same array are treated as one.
    /// Accepts both `string` and `string[]` as group elements.
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_groups")]
    pub groups: Option<Vec<Vec<String>>>,
}

/// Custom deserializer for groups field to support both `string` and `string[]` as group elements
fn deserialize_groups<'de, D>(deserializer: D) -> Result<Option<Vec<Vec<String>>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value: Option<Value> = Option::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(Value::Array(arr)) => {
            let mut groups = Vec::new();
            for item in arr {
                match item {
                    // Single string becomes a single-element group
                    Value::String(s) => {
                        groups.push(vec![s]);
                    }
                    // Array of strings becomes a group
                    Value::Array(group_arr) => {
                        let mut group = Vec::new();
                        for g in group_arr {
                            if let Value::String(s) = g {
                                group.push(s);
                            } else {
                                return Err(D::Error::custom(
                                    "groups array elements must contain only strings",
                                ));
                            }
                        }
                        groups.push(group);
                    }
                    _ => {
                        return Err(D::Error::custom(
                            "groups must be an array of strings or arrays of strings",
                        ));
                    }
                }
            }
            Ok(Some(groups))
        }
        Some(_) => Err(D::Error::custom("groups must be an array")),
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrderConfig {
    Asc,
    Desc,
}

/// Configuration for Tailwind CSS class sorting.
/// See <https://github.com/tailwindlabs/prettier-plugin-tailwindcss#options>
#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[expect(clippy::struct_field_names)] // Field names match the plugin's documented options
pub struct TailwindcssConfig {
    /// Path to Tailwind config file (v3).
    /// e.g., `"./tailwind.config.js"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tailwind_config: Option<String>,

    /// Path to Tailwind stylesheet (v4).
    /// e.g., `"./src/app.css"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tailwind_stylesheet: Option<String>,

    /// List of custom function names whose arguments should be sorted.
    /// e.g., `["clsx", "cva", "tw"]`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tailwind_functions: Option<Vec<String>>,

    /// List of additional HTML/JSX attributes to sort (beyond `class` and `className`).
    /// e.g., `["myClassProp", ":class"]`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tailwind_attributes: Option<Vec<String>>,

    /// Preserve whitespace around classes.
    /// Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tailwind_preserve_whitespace: Option<bool>,

    /// Preserve duplicate classes.
    /// Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tailwind_preserve_duplicates: Option<bool>,
}

// ---

/// Additional options specific to Oxfmt.
/// These options are not part of Prettier's configuration,
/// and `oxc_formatter` also does not use these options.
#[derive(Debug, Clone)]
pub struct OxfmtOptions {
    pub ignore_patterns: Vec<String>,
    pub sort_package_json: bool,
    pub insert_final_newline: bool,
}

impl Default for OxfmtOptions {
    fn default() -> Self {
        Self { ignore_patterns: vec![], sort_package_json: true, insert_final_newline: true }
    }
}

// ---

impl Oxfmtrc {
    /// Converts the `Oxfmtrc` into `FormatOptions` and `OxfmtOptions`.
    /// With resolving default values for unspecified options.
    ///
    /// # Errors
    /// Returns error if any option value is invalid
    pub fn into_options(self) -> Result<(FormatOptions, OxfmtOptions), String> {
        // Not yet supported options:
        // [Prettier] experimentalOperatorPosition: "start" | "end"
        // [Prettier] experimentalTernaries: boolean
        if self.experimental_operator_position.is_some() {
            return Err("Unsupported option: `experimentalOperatorPosition`".to_string());
        }
        if self.experimental_ternaries.is_some() {
            return Err("Unsupported option: `experimentalTernaries`".to_string());
        }

        // All values are based on defaults from `FormatOptions::default()`
        let mut format_options = FormatOptions::default();

        // [Prettier] useTabs: boolean
        if let Some(use_tabs) = self.use_tabs {
            format_options.indent_style =
                if use_tabs { IndentStyle::Tab } else { IndentStyle::Space };
        }

        // [Prettier] tabWidth: number
        if let Some(width) = self.tab_width {
            format_options.indent_width =
                IndentWidth::try_from(width).map_err(|e| format!("Invalid tabWidth: {e}"))?;
        }

        // [Prettier] endOfLine: "lf" | "cr" | "crlf" | "auto"
        // NOTE: "auto" is not supported
        if let Some(ending) = self.end_of_line {
            format_options.line_ending = match ending {
                EndOfLineConfig::Lf => LineEnding::Lf,
                EndOfLineConfig::Crlf => LineEnding::Crlf,
                EndOfLineConfig::Cr => LineEnding::Cr,
            };
        }

        // [Prettier] printWidth: number
        if let Some(width) = self.print_width {
            format_options.line_width =
                LineWidth::try_from(width).map_err(|e| format!("Invalid printWidth: {e}"))?;
        }

        // [Prettier] singleQuote: boolean
        if let Some(single_quote) = self.single_quote {
            format_options.quote_style =
                if single_quote { QuoteStyle::Single } else { QuoteStyle::Double };
        }

        // [Prettier] jsxSingleQuote: boolean
        if let Some(jsx_single_quote) = self.jsx_single_quote {
            format_options.jsx_quote_style =
                if jsx_single_quote { QuoteStyle::Single } else { QuoteStyle::Double };
        }

        // [Prettier] quoteProps: "as-needed" | "consistent" | "preserve"
        if let Some(props) = self.quote_props {
            format_options.quote_properties = match props {
                QuotePropsConfig::AsNeeded => QuoteProperties::AsNeeded,
                QuotePropsConfig::Consistent => QuoteProperties::Consistent,
                QuotePropsConfig::Preserve => QuoteProperties::Preserve,
            };
        }

        // [Prettier] trailingComma: "all" | "es5" | "none"
        if let Some(commas) = self.trailing_comma {
            format_options.trailing_commas = match commas {
                TrailingCommaConfig::All => TrailingCommas::All,
                TrailingCommaConfig::Es5 => TrailingCommas::Es5,
                TrailingCommaConfig::None => TrailingCommas::None,
            };
        }

        // [Prettier] semi: boolean
        if let Some(semi) = self.semi {
            format_options.semicolons =
                if semi { Semicolons::Always } else { Semicolons::AsNeeded };
        }

        // [Prettier] arrowParens: "avoid" | "always"
        if let Some(parens) = self.arrow_parens {
            format_options.arrow_parentheses = match parens {
                ArrowParensConfig::Avoid => ArrowParentheses::AsNeeded,
                ArrowParensConfig::Always => ArrowParentheses::Always,
            };
        }

        // [Prettier] bracketSpacing: boolean
        if let Some(spacing) = self.bracket_spacing {
            format_options.bracket_spacing = BracketSpacing::from(spacing);
        }

        // [Prettier] bracketSameLine: boolean
        if let Some(same_line) = self.bracket_same_line {
            format_options.bracket_same_line = BracketSameLine::from(same_line);
        }

        // [Prettier] singleAttributePerLine: boolean
        if let Some(single_attribute_per_line) = self.single_attribute_per_line {
            format_options.attribute_position = if single_attribute_per_line {
                AttributePosition::Multiline
            } else {
                AttributePosition::Auto
            };
        }

        // [Prettier] objectWrap: "preserve" | "collapse"
        if let Some(object_wrap) = self.object_wrap {
            format_options.expand = match object_wrap {
                ObjectWrapConfig::Preserve => Expand::Auto,
                ObjectWrapConfig::Collapse => Expand::Never,
                // NOTE: Our own extension
                ObjectWrapConfig::Always => Expand::Always,
            };
        }

        // [Prettier] embeddedLanguageFormatting: "auto" | "off"
        if let Some(embedded_language_formatting) = self.embedded_language_formatting {
            format_options.embedded_language_formatting = match embedded_language_formatting {
                EmbeddedLanguageFormattingConfig::Auto => EmbeddedLanguageFormatting::Auto,
                EmbeddedLanguageFormattingConfig::Off => EmbeddedLanguageFormatting::Off,
            };
        }

        // Below are our own extensions

        if let Some(config) = self.experimental_sort_imports {
            let mut sort_imports = SortImportsOptions::default();

            if let Some(v) = config.partition_by_newline {
                sort_imports.partition_by_newline = v;
            }
            if let Some(v) = config.partition_by_comment {
                sort_imports.partition_by_comment = v;
            }
            if let Some(v) = config.sort_side_effects {
                sort_imports.sort_side_effects = v;
            }
            if let Some(v) = config.order {
                sort_imports.order = match v {
                    SortOrderConfig::Asc => SortOrder::Asc,
                    SortOrderConfig::Desc => SortOrder::Desc,
                };
            }
            if let Some(v) = config.ignore_case {
                sort_imports.ignore_case = v;
            }
            if let Some(v) = config.newlines_between {
                sort_imports.newlines_between = v;
            }
            if let Some(v) = config.internal_pattern {
                sort_imports.internal_pattern = v;
            }
            if let Some(v) = config.groups {
                sort_imports.groups = v;
            }

            // `partition_by_newline: true` and `newlines_between: true` cannot be used together
            if sort_imports.partition_by_newline && sort_imports.newlines_between {
                return Err("Invalid `sortImports` configuration: `partitionByNewline: true` and `newlinesBetween: true` cannot be used together".to_string());
            }

            format_options.experimental_sort_imports = Some(sort_imports);
        }

        // [Oxfmt] experimentalTailwindcss: object | null
        if let Some(config) = self.experimental_tailwindcss {
            format_options.experimental_tailwindcss = Some(TailwindcssOptions {
                tailwind_config: config.tailwind_config,
                tailwind_stylesheet: config.tailwind_stylesheet,
                tailwind_functions: config.tailwind_functions,
                tailwind_attributes: config.tailwind_attributes,
                tailwind_preserve_whitespace: config.tailwind_preserve_whitespace,
                tailwind_preserve_duplicates: config.tailwind_preserve_duplicates,
            });
        }

        let mut oxfmt_options = OxfmtOptions::default();
        if let Some(patterns) = self.ignore_patterns {
            oxfmt_options.ignore_patterns = patterns;
        }
        if let Some(sort_package_json) = self.experimental_sort_package_json {
            oxfmt_options.sort_package_json = sort_package_json;
        }
        if let Some(insert_final_newline) = self.insert_final_newline {
            oxfmt_options.insert_final_newline = insert_final_newline;
        }

        Ok((format_options, oxfmt_options))
    }

    /// Populates the raw config JSON with resolved `FormatOptions` values.
    /// This ensures `external_formatter`(Prettier) receives the same options that `oxc_formatter` uses.
    /// Roughly the reverse of `into_format_options`.
    pub fn populate_prettier_config(options: &FormatOptions, config: &mut Value) {
        let Some(obj) = config.as_object_mut() else {
            return;
        };

        // [Prettier] useTabs: boolean
        obj.insert(
            "useTabs".to_string(),
            Value::from(match options.indent_style {
                IndentStyle::Tab => true,
                IndentStyle::Space => false,
            }),
        );

        // [Prettier] tabWidth: number
        obj.insert("tabWidth".to_string(), Value::from(options.indent_width.value()));

        // [Prettier] endOfLine: "lf" | "cr" | "crlf" | "auto"
        // NOTE: "auto" is not supported by `oxc_formatter`
        obj.insert(
            "endOfLine".to_string(),
            Value::from(match options.line_ending {
                LineEnding::Lf => "lf",
                LineEnding::Crlf => "crlf",
                LineEnding::Cr => "cr",
            }),
        );

        // [Prettier] printWidth: number
        obj.insert("printWidth".to_string(), Value::from(options.line_width.value()));

        // [Prettier] singleQuote: boolean
        obj.insert(
            "singleQuote".to_string(),
            Value::from(match options.quote_style {
                QuoteStyle::Single => true,
                QuoteStyle::Double => false,
            }),
        );

        // [Prettier] jsxSingleQuote: boolean
        obj.insert(
            "jsxSingleQuote".to_string(),
            Value::from(match options.jsx_quote_style {
                QuoteStyle::Single => true,
                QuoteStyle::Double => false,
            }),
        );

        // [Prettier] quoteProps: "as-needed" | "consistent" | "preserve"
        obj.insert(
            "quoteProps".to_string(),
            Value::from(match options.quote_properties {
                QuoteProperties::AsNeeded => "as-needed",
                QuoteProperties::Consistent => "consistent",
                QuoteProperties::Preserve => "preserve",
            }),
        );

        // [Prettier] trailingComma: "all" | "es5" | "none"
        obj.insert(
            "trailingComma".to_string(),
            Value::from(match options.trailing_commas {
                TrailingCommas::All => "all",
                TrailingCommas::Es5 => "es5",
                TrailingCommas::None => "none",
            }),
        );

        // [Prettier] semi: boolean
        obj.insert(
            "semi".to_string(),
            Value::from(match options.semicolons {
                Semicolons::Always => true,
                Semicolons::AsNeeded => false,
            }),
        );

        // [Prettier] arrowParens: "avoid" | "always"
        obj.insert(
            "arrowParens".to_string(),
            Value::from(match options.arrow_parentheses {
                ArrowParentheses::AsNeeded => "avoid",
                ArrowParentheses::Always => "always",
            }),
        );

        // [Prettier] bracketSpacing: boolean
        obj.insert("bracketSpacing".to_string(), Value::from(options.bracket_spacing.value()));

        // [Prettier] bracketSameLine: boolean
        obj.insert("bracketSameLine".to_string(), Value::from(options.bracket_same_line.value()));

        // [Prettier] singleAttributePerLine: boolean
        obj.insert(
            "singleAttributePerLine".to_string(),
            Value::from(match options.attribute_position {
                AttributePosition::Multiline => true,
                AttributePosition::Auto => false,
            }),
        );

        // [Prettier] objectWrap: "preserve" | "collapse"
        // NOTE: "always" is our extension and not supported by Prettier, fallback to "preserve" for now
        obj.insert(
            "objectWrap".to_string(),
            Value::from(match options.expand {
                Expand::Auto | Expand::Always => "preserve",
                Expand::Never => "collapse",
            }),
        );

        // [Prettier] embeddedLanguageFormatting: "auto" | "off"
        obj.insert(
            "embeddedLanguageFormatting".to_string(),
            Value::from(match options.embedded_language_formatting {
                EmbeddedLanguageFormatting::Auto => "auto",
                EmbeddedLanguageFormatting::Off => "off",
            }),
        );

        // Below are our own extensions, just remove them
        obj.remove("ignorePatterns");
        obj.remove("insertFinalNewline");
        obj.remove("experimentalSortImports");
        obj.remove("experimentalSortPackageJson");
        obj.remove("experimentalTailwindcss");

        // Any other unknown fields are preserved as-is.
        // e.g. `plugins`, `htmlWhitespaceSensitivity`, `vueIndentScriptAndStyle`, etc.
        // Other options defined independently by plugins are also left as they are.
    }

    /// Generates the JSON schema for Oxfmtrc configuration files.
    ///
    /// # Panics
    /// Panics if the schema generation fails.
    pub fn generate_schema_json() -> String {
        let mut schema = schema_for!(Oxfmtrc);

        // Allow comments and trailing commas for vscode-json-languageservice
        // NOTE: This is NOT part of standard JSON Schema specification
        // https://github.com/microsoft/vscode-json-languageservice/blob/fb83547762901f32d8449d57e24666573016b10c/src/jsonLanguageTypes.ts#L151-L159
        schema.schema.extensions.insert("allowComments".to_string(), serde_json::Value::Bool(true));
        schema
            .schema
            .extensions
            .insert("allowTrailingCommas".to_string(), serde_json::Value::Bool(true));

        // Inject markdownDescription fields for better editor support (e.g., VS Code)
        let mut json = serde_json::to_value(&schema).unwrap();
        Self::inject_markdown_descriptions(&mut json);

        // Sort keys for deterministic output across different environments.
        // Without this, CI and local environments may produce different key orders,
        // causing snapshot tests to fail.
        let sorted_json = Self::sort_json_keys(&json);

        serde_json::to_string_pretty(&sorted_json).unwrap()
    }

    /// Recursively sort all object keys in the JSON value for deterministic output.
    fn sort_json_keys(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut sorted: Vec<_> = map.iter().collect();
                sorted.sort_by(|(a, _), (b, _)| a.cmp(b));
                serde_json::Value::Object(
                    sorted.into_iter().map(|(k, v)| (k.clone(), Self::sort_json_keys(v))).collect(),
                )
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(Self::sort_json_keys).collect())
            }
            _ => value.clone(),
        }
    }

    /// Recursively inject `markdownDescription` fields into the JSON schema.
    /// This is a non-standard field that some editors (like VS Code) use to render
    /// markdown in hover tooltips.
    fn inject_markdown_descriptions(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                // If this object has a `description` field, copy it to `markdownDescription`
                if let Some(serde_json::Value::String(desc_str)) = map.get("description") {
                    map.insert(
                        "markdownDescription".to_string(),
                        serde_json::Value::String(desc_str.clone()),
                    );
                }

                // Recursively process all values in the object
                for value in map.values_mut() {
                    Self::inject_markdown_descriptions(value);
                }
            }
            serde_json::Value::Array(items) => {
                // Recursively process all items in the array
                for item in items {
                    Self::inject_markdown_descriptions(item);
                }
            }
            _ => {
                // Primitive values don't need processing
            }
        }
    }
}

// ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let json = r#"{
            "useTabs": true,
            "tabWidth": 4,
            "printWidth": 100,
            "singleQuote": true,
            "semi": false,
            "experimentalSortImports": {
                "partitionByNewline": true,
                "order": "desc",
                "ignoreCase": false,
                "newlinesBetween": false
            }
        }"#;

        let config: Oxfmtrc = serde_json::from_str(json).unwrap();
        let (format_options, _) = config.into_options().unwrap();

        assert!(format_options.indent_style.is_tab());
        assert_eq!(format_options.indent_width.value(), 4);
        assert_eq!(format_options.line_width.value(), 100);
        assert!(!format_options.quote_style.is_double());
        assert!(format_options.semicolons.is_as_needed());

        let sort_imports = format_options.experimental_sort_imports.unwrap();
        assert!(sort_imports.partition_by_newline);
        assert!(sort_imports.order.is_desc());
        assert!(!sort_imports.ignore_case);
        assert!(!sort_imports.newlines_between);
    }

    #[test]
    fn test_ignore_unknown_fields() {
        let config: Oxfmtrc = serde_json::from_str(
            r#"{
                "unknownField": "someValue",
                "anotherUnknown": 123
            }"#,
        )
        .unwrap();
        let (format_options, _) = config.into_options().unwrap();

        // Should use defaults
        assert!(format_options.indent_style.is_space());
        assert_eq!(format_options.indent_width.value(), 2);
        assert_eq!(format_options.line_width.value(), 100);
        assert_eq!(format_options.experimental_sort_imports, None);
    }

    #[test]
    fn test_empty_config() {
        let config: Oxfmtrc = serde_json::from_str("{}").unwrap();
        let (format_options, _) = config.into_options().unwrap();

        // Should use defaults
        assert!(format_options.indent_style.is_space());
        assert_eq!(format_options.indent_width.value(), 2);
        assert_eq!(format_options.line_width.value(), 100);
        assert_eq!(format_options.experimental_sort_imports, None);
    }

    #[test]
    fn test_arrow_parens_normalization() {
        // Test "avoid" -> "as-needed" normalization
        let config: Oxfmtrc = serde_json::from_str(r#"{"arrowParens": "avoid"}"#).unwrap();
        let (format_options, _) = config.into_options().unwrap();
        assert!(format_options.arrow_parentheses.is_as_needed());

        // Test "always" remains unchanged
        let config: Oxfmtrc = serde_json::from_str(r#"{"arrowParens": "always"}"#).unwrap();
        let (format_options, _) = config.into_options().unwrap();
        assert!(format_options.arrow_parentheses.is_always());
    }

    #[test]
    fn test_object_wrap_normalization() {
        // Test "preserve" -> "auto" normalization
        let config: Oxfmtrc = serde_json::from_str(r#"{"objectWrap": "preserve"}"#).unwrap();
        let (format_options, _) = config.into_options().unwrap();
        assert_eq!(format_options.expand, Expand::Auto);

        // Test "collapse" -> "never" normalization
        let config: Oxfmtrc = serde_json::from_str(r#"{"objectWrap": "collapse"}"#).unwrap();
        let (format_options, _) = config.into_options().unwrap();
        assert_eq!(format_options.expand, Expand::Never);

        // Test "always" remains unchanged
        let config: Oxfmtrc = serde_json::from_str(r#"{"objectWrap": "always"}"#).unwrap();
        let (format_options, _) = config.into_options().unwrap();
        assert_eq!(format_options.expand, Expand::Always);
    }

    #[test]
    fn test_sort_imports_config() {
        let config: Oxfmtrc = serde_json::from_str(
            r#"{
            "experimentalSortImports": {}
        }"#,
        )
        .unwrap();
        let (format_options, _) = config.into_options().unwrap();
        let sort_imports = format_options.experimental_sort_imports.unwrap();
        assert!(sort_imports.newlines_between);
        assert!(!sort_imports.partition_by_newline);

        // Test explicit false
        let config: Oxfmtrc = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "newlinesBetween": false
                }
            }"#,
        )
        .unwrap();
        let (format_options, _) = config.into_options().unwrap();
        let sort_imports = format_options.experimental_sort_imports.unwrap();
        assert!(!sort_imports.newlines_between);
        assert!(!sort_imports.partition_by_newline);

        // Test explicit true
        let config: Oxfmtrc = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "newlinesBetween": true
                }
            }"#,
        )
        .unwrap();
        let (format_options, _) = config.into_options().unwrap();
        let sort_imports = format_options.experimental_sort_imports.unwrap();
        assert!(sort_imports.newlines_between);
        assert!(!sort_imports.partition_by_newline);

        let config: Oxfmtrc = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "partitionByNewline": true,
                    "newlinesBetween": false
                }
            }"#,
        )
        .unwrap();
        assert!(config.into_options().is_ok());
        let config: Oxfmtrc = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "partitionByNewline": true,
                    "newlinesBetween": true
                }
            }"#,
        )
        .unwrap();
        assert!(config.into_options().is_err_and(|e| e.contains("newlinesBetween")));

        let config: Oxfmtrc = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "groups": [
                        "builtin",
                        ["external", "internal"],
                        "parent",
                        "sibling",
                        "index"
                    ]
                }
            }"#,
        )
        .unwrap();
        let (format_options, _) = config.into_options().unwrap();
        let sort_imports = format_options.experimental_sort_imports.unwrap();
        assert_eq!(sort_imports.groups.len(), 5);
        assert_eq!(sort_imports.groups[0], vec!["builtin".to_string()]);
        assert_eq!(sort_imports.groups[1], vec!["external".to_string(), "internal".to_string()]);
        assert_eq!(sort_imports.groups[4], vec!["index".to_string()]);
    }

    #[test]
    fn test_populate_prettier_config_defaults() {
        let json_string = r"{}";
        let mut raw_config: Value = serde_json::from_str(json_string).unwrap();
        let oxfmtrc: Oxfmtrc = serde_json::from_str(json_string).unwrap();
        let (format_options, _) = oxfmtrc.into_options().unwrap();

        Oxfmtrc::populate_prettier_config(&format_options, &mut raw_config);

        let obj = raw_config.as_object().unwrap();
        assert_eq!(obj.get("printWidth").unwrap(), 100);
    }

    #[test]
    fn test_populate_prettier_config_with_user_values() {
        let json_string = r#"{
            "printWidth": 80,
            "ignorePatterns": ["*.min.js"],
            "experimentalSortImports": { "order": "asc" }
        }"#;
        let mut raw_config: Value = serde_json::from_str(json_string).unwrap();
        let oxfmtrc: Oxfmtrc = serde_json::from_str(json_string).unwrap();
        let (format_options, _) = oxfmtrc.into_options().unwrap();

        Oxfmtrc::populate_prettier_config(&format_options, &mut raw_config);

        let obj = raw_config.as_object().unwrap();
        // User-specified value is preserved via FormatOptions
        assert_eq!(obj.get("printWidth").unwrap(), 80);
        // oxfmt extensions are removed
        assert!(!obj.contains_key("ignorePatterns"));
        assert!(!obj.contains_key("experimentalSortImports"));
    }
}
