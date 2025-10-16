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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_tabs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_width: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_of_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_width: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_quote: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_single_quote: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_props: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_comma: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semi: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrow_parens: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_spacing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_same_line: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_wrap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_attribute_per_line: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_operator_position: Option<String>,
    // TODO: experimental_ternaries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_sort_imports: Option<SortImportsConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SortImportsConfig {
    #[serde(default)]
    pub partition_by_newline: bool,
    #[serde(default)]
    pub partition_by_comment: bool,
    #[serde(default)]
    pub sort_side_effects: bool,
    #[serde(default = "default_order")]
    pub order: String,
    #[serde(default = "default_true")]
    pub ignore_case: bool,
}

fn default_order() -> String {
    "asc".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for SortImportsConfig {
    fn default() -> Self {
        Self {
            partition_by_newline: false,
            partition_by_comment: false,
            sort_side_effects: false,
            order: default_order(),
            ignore_case: default_true(),
        }
    }
}

impl Oxfmtrc {
    /// # Errors
    /// Returns error if file cannot be read or parsed
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
            options.line_ending =
                ending.parse::<LineEnding>().map_err(|e| format!("Invalid endOfLine: {e}"))?;
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
            options.quote_properties =
                props.parse::<QuoteProperties>().map_err(|e| format!("Invalid quoteProps: {e}"))?;
        }

        // [Prettier] trailingComma: "all" | "es5" | "none"
        if let Some(commas) = self.trailing_comma {
            options.trailing_commas = commas
                .parse::<TrailingCommas>()
                .map_err(|e| format!("Invalid trailingComma: {e}"))?;
        }

        // [Prettier] semi: boolean -> Semicolons
        if let Some(semi) = self.semi {
            options.semicolons = if semi { Semicolons::Always } else { Semicolons::AsNeeded };
        }

        // [Prettier] arrowParens: "avoid" | "always"
        if let Some(parens) = self.arrow_parens {
            let normalized = match parens.as_str() {
                "avoid" => "as-needed",
                _ => &parens,
            };
            options.arrow_parentheses = normalized
                .parse::<ArrowParentheses>()
                .map_err(|e| format!("Invalid arrowParens: {e}"))?;
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
        // NOTE: In addition to Prettier, we also support "always"
        if let Some(object_wrap) = self.object_wrap {
            let normalized = match object_wrap.as_str() {
                "preserve" => "auto",
                "collapse" => "never",
                _ => &object_wrap,
            };
            options.expand =
                normalized.parse::<Expand>().map_err(|e| format!("Invalid objectWrap: {e}"))?;
        }

        // [Prettier] experimentalOperatorPosition: "start" | "end"
        if let Some(position) = self.experimental_operator_position {
            options.experimental_operator_position = position
                .parse::<OperatorPosition>()
                .map_err(|e| format!("Invalid experimental_operator_position: {e}"))?;
        }

        // Below are our own extensions

        if let Some(sort_imports_config) = self.experimental_sort_imports {
            let order = sort_imports_config
                .order
                .parse::<SortOrder>()
                .map_err(|e| format!("Invalid sort_imports.order: {e}"))?;

            options.experimental_sort_imports = Some(SortImports {
                partition_by_newline: sort_imports_config.partition_by_newline,
                partition_by_comment: sort_imports_config.partition_by_comment,
                sort_side_effects: sort_imports_config.sort_side_effects,
                order,
                ignore_case: sort_imports_config.ignore_case,
            });
        }

        Ok(options)
    }
}

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
