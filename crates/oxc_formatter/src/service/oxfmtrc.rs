use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ArrowParentheses, AttributePosition, BracketSameLine, BracketSpacing, Expand, FormatOptions,
    IndentStyle, IndentWidth, LineEnding, LineWidth, OperatorPosition, QuoteProperties, QuoteStyle,
    Semicolons, SortImports, SortOrder, TrailingCommas,
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
    /// The line length that the printer will wrap on. (Default: 80)
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
    /// Experimental: Position of operators in expressions. (Default: "end")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_operator_position: Option<OperatorPositionConfig>,
    // TODO: Experimental: Use curious ternaries which move `?` after the condition. (Default: false)
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub experimental_ternaries: Option<bool>,
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
pub enum OperatorPositionConfig {
    Start,
    #[default]
    End,
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
}

fn default_true() -> bool {
    true
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

        let json = serde_json::from_str::<serde_json::Value>(&string)
            .map_err(|err| format!("Failed to parse config {}: {err}", path.display()))?;

        // NOTE: String enum deserialization errors are handled here
        Self::deserialize(&json)
            .map_err(|err| format!("Failed to deserialize config {}: {err}", path.display()))
    }

    /// # Errors
    /// Returns error if any option value is invalid
    pub fn into_format_options(self) -> Result<FormatOptions, String> {
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

        // [Prettier] experimentalOperatorPosition: "start" | "end"
        if let Some(position) = self.experimental_operator_position {
            options.experimental_operator_position = match position {
                OperatorPositionConfig::Start => OperatorPosition::Start,
                OperatorPositionConfig::End => OperatorPosition::End,
            };
        }

        // Below are our own extensions

        if let Some(sort_imports_config) = self.experimental_sort_imports {
            options.experimental_sort_imports = Some(SortImports {
                partition_by_newline: sort_imports_config.partition_by_newline,
                partition_by_comment: sort_imports_config.partition_by_comment,
                sort_side_effects: sort_imports_config.sort_side_effects,
                order: sort_imports_config.order.map_or(SortOrder::Asc, |o| match o {
                    SortOrderConfig::Asc => SortOrder::Asc,
                    SortOrderConfig::Desc => SortOrder::Desc,
                }),

                ignore_case: sort_imports_config.ignore_case,
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
                "ignoreCase": false
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
        assert_eq!(options.line_width.value(), 80);
        assert_eq!(options.experimental_sort_imports, None);
    }

    #[test]
    fn test_empty_config() {
        let config: Oxfmtrc = serde_json::from_str("{}").unwrap();
        let options = config.into_format_options().unwrap();

        // Should use defaults
        assert!(options.indent_style.is_space());
        assert_eq!(options.indent_width.value(), 2);
        assert_eq!(options.line_width.value(), 80);
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
}
