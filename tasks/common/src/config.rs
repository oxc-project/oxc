//! Configuration parsing utilities
//!
//! This module provides utilities for parsing various configuration formats
//! commonly used in task binaries.

use std::collections::HashMap;

use serde_json::Value;

/// Utilities for parsing JavaScript/JSON configuration
pub struct ConfigParser;

impl ConfigParser {
    /// Convert JavaScript-like config syntax to JSON
    /// This handles common patterns like unquoted keys, single quotes, etc.
    pub fn js_to_json(js_config: &str) -> Result<String, String> {
        // This is a simplified version - for more complex cases,
        // you might want to use a proper JS parser
        let mut json_config = js_config.to_string();

        // Handle common transformations
        json_config = Self::handle_unquoted_keys(&json_config);
        json_config = Self::handle_single_quotes(&json_config);
        json_config = Self::handle_trailing_commas(&json_config);
        json_config = Self::handle_undefined(&json_config);

        // Validate that it's valid JSON
        serde_json::from_str::<Value>(&json_config)
            .map_err(|e| format!("Invalid JSON after transformation: {e}"))?;

        Ok(json_config)
    }

    /// Handle unquoted object keys
    fn handle_unquoted_keys(config: &str) -> String {
        // This is a simplified implementation
        // In a real scenario, you'd want more robust parsing
        config.to_string()
    }

    /// Convert single quotes to double quotes
    fn handle_single_quotes(config: &str) -> String {
        // Simple replacement - in practice you'd need to handle escaped quotes
        config.replace('\'', "\"")
    }

    /// Remove trailing commas
    fn handle_trailing_commas(config: &str) -> String {
        // Remove trailing commas before closing braces/brackets
        config.replace(",}", "}").replace(",]", "]")
    }

    /// Handle undefined values
    fn handle_undefined(config: &str) -> String {
        config.replace("undefined", "null")
    }

    /// Parse a JSON configuration string
    pub fn parse_json(json_str: &str) -> Result<Value, String> {
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse JSON: {e}"))
    }

    /// Extract specific configuration values
    pub fn extract_config_value<'a>(config: &'a Value, key: &str) -> Option<&'a Value> {
        config.get(key)
    }

    /// Convert configuration to a flat HashMap for easier access
    pub fn flatten_config(config: &Value) -> HashMap<String, String> {
        let mut result = HashMap::new();
        Self::flatten_recursive(config, String::new(), &mut result);
        result
    }

    fn flatten_recursive(value: &Value, prefix: String, result: &mut HashMap<String, String>) {
        match value {
            Value::Object(obj) => {
                for (key, val) in obj {
                    let new_prefix =
                        if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };
                    Self::flatten_recursive(val, new_prefix, result);
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let new_prefix = format!("{prefix}[{i}]");
                    Self::flatten_recursive(val, new_prefix, result);
                }
            }
            _ => {
                result.insert(prefix, value.to_string());
            }
        }
    }
}

/// ESLint-style configuration parsing
pub struct ESLintConfigParser;

impl ESLintConfigParser {
    /// Parse ESLint rule configuration
    pub fn parse_rule_config(config_str: &str) -> Result<Value, String> {
        // Handle common ESLint config patterns
        let normalized = Self::normalize_eslint_config(config_str);
        ConfigParser::parse_json(&normalized)
    }

    /// Normalize ESLint configuration syntax
    fn normalize_eslint_config(config: &str) -> String {
        // Handle ESLint-specific patterns like rule severity levels
        config.replace("\"error\"", "2").replace("\"warn\"", "1").replace("\"off\"", "0")
    }
}

/// Test configuration utilities
pub struct TestConfigParser;

impl TestConfigParser {
    /// Parse test case options from various formats
    pub fn parse_test_options(input: &str) -> HashMap<String, Value> {
        let mut options = HashMap::new();

        // Try to parse as JSON first
        if let Ok(value) = serde_json::from_str::<Value>(input) {
            if let Value::Object(obj) = value {
                for (key, val) in obj {
                    options.insert(key, val);
                }
            }
        }

        options
    }

    /// Extract filename from test configuration
    pub fn extract_filename(config: &HashMap<String, Value>) -> Option<String> {
        config.get("filename").and_then(|v| v.as_str()).map(|s| s.trim_matches('"').to_string())
    }

    /// Extract language options from test configuration
    pub fn extract_language_options(config: &HashMap<String, Value>) -> Option<String> {
        config.get("languageOptions").map(|v| v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_quote_conversion() {
        let input = r#"{'key': 'value'}"#;
        let expected = r#"{"key": "value"}"#;
        assert_eq!(ConfigParser::handle_single_quotes(input), expected);
    }

    #[test]
    fn test_trailing_comma_removal() {
        let input = r#"{"key": "value",}"#;
        let expected = r#"{"key": "value"}"#;
        assert_eq!(ConfigParser::handle_trailing_commas(input), expected);
    }

    #[test]
    fn test_undefined_handling() {
        let input = r#"{"key": undefined}"#;
        let expected = r#"{"key": null}"#;
        assert_eq!(ConfigParser::handle_undefined(input), expected);
    }
}
