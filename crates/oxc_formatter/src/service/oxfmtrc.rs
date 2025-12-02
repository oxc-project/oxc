use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    ArrowParentheses, AttributePosition, BracketSameLine, BracketSpacing,
    EmbeddedLanguageFormatting, Expand, FormatOptions, IndentStyle, IndentWidth, LineEnding,
    LineWidth, QuoteProperties, QuoteStyle, Semicolons, SortImportsOptions, SortOrder,
    TrailingCommas, default_groups, default_internal_patterns,
};

/// Configuration options for the formatter.
/// Most options are the same as Prettier's options.
/// See also <https://prettier.io/docs/options>
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Oxfmtrc {
    /// Use tabs for indentation or spaces. (Default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_tabs: Option<bool>,
    /// Number of spaces per indentation level. (Default: 2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_width: Option<u8>,
    /// Which end of line characters to apply. (Default: "lf")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_of_line: Option<EndOfLineConfig>,
    /// The line length that the printer will wrap on. (Default: 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_width: Option<u16>,
    /// Use single quotes instead of double quotes. (Default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_quote: Option<bool>,
    /// Use single quotes instead of double quotes in JSX. (Default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_single_quote: Option<bool>,
    /// Change when properties in objects are quoted. (Default: "as-needed")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_props: Option<QuotePropsConfig>,
    /// Print trailing commas wherever possible. (Default: "all")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_comma: Option<TrailingCommaConfig>,
    /// Print semicolons at the ends of statements. (Default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semi: Option<bool>,
    /// Include parentheses around a sole arrow function parameter. (Default: "always")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrow_parens: Option<ArrowParensConfig>,
    /// Print spaces between brackets in object literals. (Default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_spacing: Option<bool>,
    /// Put the > of a multi-line JSX element at the end of the last line instead of being alone on the next line. (Default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_same_line: Option<bool>,
    /// How to wrap object literals when they could fit on one line or span multiple lines. (Default: "preserve")
    /// NOTE: In addition to Prettier's "preserve" and "collapse", we also support "always".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_wrap: Option<ObjectWrapConfig>,
    /// Put each attribute on a new line in JSX. (Default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_attribute_per_line: Option<bool>,

    // NOTE: These experimental options are not yet supported.
    // Just be here to report error if they are used.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(skip)]
    pub experimental_operator_position: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(skip)]
    pub experimental_ternaries: Option<serde_json::Value>,

    /// Control whether formats quoted code embedded in the file. (Default: "auto")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedded_language_formatting: Option<EmbeddedLanguageFormattingConfig>,

    /// Experimental: Sort import statements. Disabled by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_sort_imports: Option<SortImportsConfig>,

    /// Ignore files matching these glob patterns. Current working directory is used as the root.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_patterns: Option<Vec<String>>,
}

// ---

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EndOfLineConfig {
    #[default]
    Lf,
    Crlf,
    Cr,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum QuotePropsConfig {
    #[default]
    AsNeeded,
    Preserve,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TrailingCommaConfig {
    #[default]
    All,
    Es5,
    None,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ArrowParensConfig {
    #[default]
    Always,
    Avoid,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ObjectWrapConfig {
    #[default]
    Preserve,
    Collapse,
    Always,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddedLanguageFormattingConfig {
    Auto,
    // Disable by default at alpha release, synced with `options.rs`
    #[default]
    Off,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SortImportsConfig {
    #[serde(default)]
    pub partition_by_newline: bool,
    #[serde(default)]
    pub partition_by_comment: bool,
    #[serde(default)]
    pub sort_side_effects: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrderConfig>,
    #[serde(default = "default_true")]
    pub ignore_case: bool,
    #[serde(default = "default_true")]
    pub newlines_between: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_pattern: Option<Vec<String>>,
    /// Custom groups configuration for organizing imports.
    /// Each array element represents a group, and multiple group names in the same array are treated as one.
    /// Accepts both `string` and `string[]` as group elements.
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_groups")]
    pub groups: Option<Vec<Vec<String>>>,
}

fn default_true() -> bool {
    true
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

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrderConfig {
    #[default]
    Asc,
    Desc,
}

// ---

impl Oxfmtrc {
    /// # Errors
    /// Returns error if:
    /// - file cannot be found or read
    /// - file content is not valid JSONC
    /// - deserialization fails for string enum values
    pub fn from_file(path: &Path) -> Result<Self, String> {
        // TODO: Use `simdutf8` like `oxc_linter`?
        let mut string = std::fs::read_to_string(path)
            // Do not include OS error, it differs between platforms
            .map_err(|_| format!("Failed to read config {}: File not found", path.display()))?;

        // JSONC support - strip comments
        json_strip_comments::strip(&mut string)
            .map_err(|err| format!("Failed to strip comments from {}: {err}", path.display()))?;

        // NOTE: String enum deserialization errors are handled here
        serde_json::from_str(&string)
            .map_err(|err| format!("Failed to deserialize config {}: {err}", path.display()))
    }

    /// # Errors
    /// Returns error if any option value is invalid
    pub fn into_format_options(self) -> Result<FormatOptions, String> {
        // Not yet supported options:
        // [Prettier] experimentalOperatorPosition: "start" | "end"
        // [Prettier] experimentalTernaries: boolean
        if self.experimental_operator_position.is_some() {
            return Err("Unsupported option: `experimentalOperatorPosition`".to_string());
        }
        if self.experimental_ternaries.is_some() {
            return Err("Unsupported option: `experimentalTernaries`".to_string());
        }

        let mut options = FormatOptions::default();

        // [Prettier] useTabs: boolean
        if let Some(use_tabs) = self.use_tabs {
            options.indent_style = if use_tabs { IndentStyle::Tab } else { IndentStyle::Space };
        }

        // [Prettier] tabWidth: number
        if let Some(width) = self.tab_width {
            options.indent_width =
                IndentWidth::try_from(width).map_err(|e| format!("Invalid tabWidth: {e}"))?;
        }

        // [Prettier] endOfLine: "lf" | "cr" | "crlf" | "auto"
        // NOTE: "auto" is not supported
        if let Some(ending) = self.end_of_line {
            options.line_ending = match ending {
                EndOfLineConfig::Lf => LineEnding::Lf,
                EndOfLineConfig::Crlf => LineEnding::Crlf,
                EndOfLineConfig::Cr => LineEnding::Cr,
            };
        }

        // [Prettier] printWidth: number
        if let Some(width) = self.print_width {
            options.line_width =
                LineWidth::try_from(width).map_err(|e| format!("Invalid printWidth: {e}"))?;
        }

        // [Prettier] singleQuote: boolean
        if let Some(single_quote) = self.single_quote {
            options.quote_style =
                if single_quote { QuoteStyle::Single } else { QuoteStyle::Double };
        }

        // [Prettier] jsxSingleQuote: boolean
        if let Some(jsx_single_quote) = self.jsx_single_quote {
            options.jsx_quote_style =
                if jsx_single_quote { QuoteStyle::Single } else { QuoteStyle::Double };
        }

        // [Prettier] quoteProps: "as-needed" | "consistent" | "preserve"
        // NOTE: "consistent" is not supported
        if let Some(props) = self.quote_props {
            options.quote_properties = match props {
                QuotePropsConfig::AsNeeded => QuoteProperties::AsNeeded,
                QuotePropsConfig::Preserve => QuoteProperties::Preserve,
            };
        }

        // [Prettier] trailingComma: "all" | "es5" | "none"
        if let Some(commas) = self.trailing_comma {
            options.trailing_commas = match commas {
                TrailingCommaConfig::All => TrailingCommas::All,
                TrailingCommaConfig::Es5 => TrailingCommas::Es5,
                TrailingCommaConfig::None => TrailingCommas::None,
            };
        }

        // [Prettier] semi: boolean
        if let Some(semi) = self.semi {
            options.semicolons = if semi { Semicolons::Always } else { Semicolons::AsNeeded };
        }

        // [Prettier] arrowParens: "avoid" | "always"
        if let Some(parens) = self.arrow_parens {
            options.arrow_parentheses = match parens {
                ArrowParensConfig::Avoid => ArrowParentheses::AsNeeded,
                ArrowParensConfig::Always => ArrowParentheses::Always,
            };
        }

        // [Prettier] bracketSpacing: boolean
        if let Some(spacing) = self.bracket_spacing {
            options.bracket_spacing = BracketSpacing::from(spacing);
        }

        // [Prettier] bracketSameLine: boolean
        if let Some(same_line) = self.bracket_same_line {
            options.bracket_same_line = BracketSameLine::from(same_line);
        }

        // [Prettier] singleAttributePerLine: boolean
        if let Some(single_attribute_per_line) = self.single_attribute_per_line {
            options.attribute_position = if single_attribute_per_line {
                AttributePosition::Multiline
            } else {
                AttributePosition::Auto
            };
        }

        // [Prettier] objectWrap: "preserve" | "collapse"
        if let Some(object_wrap) = self.object_wrap {
            options.expand = match object_wrap {
                ObjectWrapConfig::Preserve => Expand::Auto,
                ObjectWrapConfig::Collapse => Expand::Never,
                // NOTE: Our own extension
                ObjectWrapConfig::Always => Expand::Always,
            };
        }

        if let Some(embedded_language_formatting) = self.embedded_language_formatting {
            options.embedded_language_formatting = match embedded_language_formatting {
                EmbeddedLanguageFormattingConfig::Auto => EmbeddedLanguageFormatting::Auto,
                EmbeddedLanguageFormattingConfig::Off => EmbeddedLanguageFormatting::Off,
            };
        }

        // Below are our own extensions

        if let Some(sort_imports_config) = self.experimental_sort_imports {
            // `partition_by_newline: true` and `newlines_between` cannot be used together
            if sort_imports_config.partition_by_newline && sort_imports_config.newlines_between {
                return Err("Invalid `sortImports` configuration: `partitionByNewline: true` and `newlinesBetween: true` cannot be used together".to_string());
            }

            options.experimental_sort_imports = Some(SortImportsOptions {
                partition_by_newline: sort_imports_config.partition_by_newline,
                partition_by_comment: sort_imports_config.partition_by_comment,
                sort_side_effects: sort_imports_config.sort_side_effects,
                order: sort_imports_config.order.map_or(SortOrder::default(), |o| match o {
                    SortOrderConfig::Asc => SortOrder::Asc,
                    SortOrderConfig::Desc => SortOrder::Desc,
                }),
                ignore_case: sort_imports_config.ignore_case,
                newlines_between: sort_imports_config.newlines_between,
                internal_pattern: sort_imports_config
                    .internal_pattern
                    .unwrap_or_else(default_internal_patterns),
                groups: sort_imports_config.groups.unwrap_or_else(default_groups),
            });
        }

        Ok(options)
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
        let options = config.into_format_options().unwrap();

        assert!(options.indent_style.is_tab());
        assert_eq!(options.indent_width.value(), 4);
        assert_eq!(options.line_width.value(), 100);
        assert!(!options.quote_style.is_double());
        assert!(options.semicolons.is_as_needed());

        let sort_imports = options.experimental_sort_imports.unwrap();
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
        let options = config.into_format_options().unwrap();

        // Should use defaults
        assert!(options.indent_style.is_space());
        assert_eq!(options.indent_width.value(), 2);
        assert_eq!(options.line_width.value(), 100);
        assert_eq!(options.experimental_sort_imports, None);
    }

    #[test]
    fn test_empty_config() {
        let config: Oxfmtrc = serde_json::from_str("{}").unwrap();
        let options = config.into_format_options().unwrap();

        // Should use defaults
        assert!(options.indent_style.is_space());
        assert_eq!(options.indent_width.value(), 2);
        assert_eq!(options.line_width.value(), 100);
        assert_eq!(options.experimental_sort_imports, None);
    }

    #[test]
    fn test_arrow_parens_normalization() {
        // Test "avoid" -> "as-needed" normalization
        let config: Oxfmtrc = serde_json::from_str(r#"{"arrowParens": "avoid"}"#).unwrap();
        let options = config.into_format_options().unwrap();
        assert!(options.arrow_parentheses.is_as_needed());

        // Test "always" remains unchanged
        let config: Oxfmtrc = serde_json::from_str(r#"{"arrowParens": "always"}"#).unwrap();
        let options = config.into_format_options().unwrap();
        assert!(options.arrow_parentheses.is_always());
    }

    #[test]
    fn test_object_wrap_normalization() {
        // Test "preserve" -> "auto" normalization
        let config: Oxfmtrc = serde_json::from_str(r#"{"objectWrap": "preserve"}"#).unwrap();
        let options = config.into_format_options().unwrap();
        assert_eq!(options.expand, Expand::Auto);

        // Test "collapse" -> "never" normalization
        let config: Oxfmtrc = serde_json::from_str(r#"{"objectWrap": "collapse"}"#).unwrap();
        let options = config.into_format_options().unwrap();
        assert_eq!(options.expand, Expand::Never);

        // Test "always" remains unchanged
        let config: Oxfmtrc = serde_json::from_str(r#"{"objectWrap": "always"}"#).unwrap();
        let options = config.into_format_options().unwrap();
        assert_eq!(options.expand, Expand::Always);
    }

    #[test]
    fn test_sort_imports_config() {
        let config: Oxfmtrc = serde_json::from_str(
            r#"{
            "experimentalSortImports": {}
        }"#,
        )
        .unwrap();
        let sort_imports = config.into_format_options().unwrap().experimental_sort_imports.unwrap();
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
        let sort_imports = config.into_format_options().unwrap().experimental_sort_imports.unwrap();
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
        let sort_imports = config.into_format_options().unwrap().experimental_sort_imports.unwrap();
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
        assert!(config.into_format_options().is_ok());
        let config: Oxfmtrc = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "partitionByNewline": true,
                    "newlinesBetween": true
                }
            }"#,
        )
        .unwrap();
        assert!(config.into_format_options().is_err_and(|e| e.contains("newlinesBetween")));

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
        let sort_imports = config.into_format_options().unwrap().experimental_sort_imports.unwrap();
        assert_eq!(sort_imports.groups.len(), 5);
        assert_eq!(sort_imports.groups[0], vec!["builtin".to_string()]);
        assert_eq!(sort_imports.groups[1], vec!["external".to_string(), "internal".to_string()]);
        assert_eq!(sort_imports.groups[4], vec!["index".to_string()]);
    }
}
