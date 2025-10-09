use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    ArrowParentheses, AttributePosition, BracketSameLine, BracketSpacing, Expand, FormatOptions,
    IndentStyle, IndentWidth, LineEnding, LineWidth, OperatorPosition, QuoteProperties, QuoteStyle,
    Semicolons, SortImports, SortOrder, TrailingCommas,
};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Oxfmtrc {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_width: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_ending: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_width: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_quote_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_properties: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_commas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semicolons: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrow_parentheses: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_spacing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_same_line: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_operator_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_sort_imports: Option<SortImportsConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
            .map_err(|err| format!("Failed to read config {}: {err}", path.display()))?;

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

        if let Some(style) = self.indent_style {
            options.indent_style =
                style.parse::<IndentStyle>().map_err(|e| format!("Invalid indent_style: {e}"))?;
        }

        if let Some(width) = self.indent_width {
            options.indent_width =
                IndentWidth::try_from(width).map_err(|e| format!("Invalid indent_width: {e}"))?;
        }

        if let Some(ending) = self.line_ending {
            options.line_ending =
                ending.parse::<LineEnding>().map_err(|e| format!("Invalid line_ending: {e}"))?;
        }

        if let Some(width) = self.line_width {
            options.line_width =
                LineWidth::try_from(width).map_err(|e| format!("Invalid line_width: {e}"))?;
        }

        if let Some(style) = self.quote_style {
            options.quote_style =
                style.parse::<QuoteStyle>().map_err(|e| format!("Invalid quote_style: {e}"))?;
        }

        if let Some(style) = self.jsx_quote_style {
            options.jsx_quote_style =
                style.parse::<QuoteStyle>().map_err(|e| format!("Invalid jsx_quote_style: {e}"))?;
        }

        if let Some(props) = self.quote_properties {
            options.quote_properties = props
                .parse::<QuoteProperties>()
                .map_err(|e| format!("Invalid quote_properties: {e}"))?;
        }

        if let Some(commas) = self.trailing_commas {
            options.trailing_commas = commas
                .parse::<TrailingCommas>()
                .map_err(|e| format!("Invalid trailing_commas: {e}"))?;
        }

        if let Some(semis) = self.semicolons {
            options.semicolons =
                semis.parse::<Semicolons>().map_err(|e| format!("Invalid semicolons: {e}"))?;
        }

        if let Some(parens) = self.arrow_parentheses {
            options.arrow_parentheses = parens
                .parse::<ArrowParentheses>()
                .map_err(|e| format!("Invalid arrow_parentheses: {e}"))?;
        }

        if let Some(spacing) = self.bracket_spacing {
            options.bracket_spacing = BracketSpacing::from(spacing);
        }

        if let Some(same_line) = self.bracket_same_line {
            options.bracket_same_line = BracketSameLine::from(same_line);
        }

        if let Some(position) = self.attribute_position {
            options.attribute_position = position
                .parse::<AttributePosition>()
                .map_err(|e| format!("Invalid attribute_position: {e}"))?;
        }

        if let Some(expand) = self.expand {
            options.expand =
                expand.parse::<Expand>().map_err(|e| format!("Invalid expand: {e}"))?;
        }

        if let Some(position) = self.experimental_operator_position {
            options.experimental_operator_position = position
                .parse::<OperatorPosition>()
                .map_err(|e| format!("Invalid experimental_operator_position: {e}"))?;
        }

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
            "indentStyle": "tab",
            "indentWidth": 4,
            "lineWidth": 100,
            "quoteStyle": "single",
            "semicolons": "as-needed",
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
}
