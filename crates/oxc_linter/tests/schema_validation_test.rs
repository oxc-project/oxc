//! Test to ensure JSON Schema validation is working correctly
//!
//! This test validates that the generated JSON schema for oxlintrc configuration
//! files correctly validates both valid and invalid configurations. This helps
//! prevent regressions in the schema generation process.

use std::fs;
use std::path::PathBuf;

use jsonschema::Validator;
use project_root::get_project_root;

/// Get the path to the test fixtures directory
fn get_fixtures_path() -> PathBuf {
    get_project_root().unwrap().join("crates/oxc_linter/tests/fixtures/schema_validation")
}

fn schema_content() -> String {
    // Load the generated schema from the expected location
    let schema_json = get_project_root().unwrap().join("npm/oxlint/configuration_schema.json");

    fs::read_to_string(&schema_json).expect("Failed to read generated schema")
}

/// Load the JSON schema from 'npm/oxlint/configuration_schema.json'
fn load_schema() -> Validator {
    let schema: serde_json::Value =
        serde_json::from_str(&schema_content()).expect("Failed to parse generated schema as JSON");

    Validator::new(&schema).expect("Failed to compile JSON schema")
}

/// Test that valid configuration files pass schema validation
#[test]
fn test_valid_configs_pass_validation() {
    let schema = load_schema();
    let valid_dir = get_fixtures_path().join("valid");

    let test_files = [
        "basic_plugins.json",
        "valid_categories.json",
        "full_config.json",
        "globals_config.json",
        "env_config.json",
        "rules_config.json",
        "ignore_patterns.json",
        "empty_config.json",
    ];

    for file_name in &test_files {
        let file_path = valid_dir.join(file_name);
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("Failed to read {file_name}: {e}"));

        let instance: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {file_name} as JSON: {e}"));

        // Use iter_errors to get all validation errors
        let errors: Vec<_> = schema.iter_errors(&instance).collect();

        if !errors.is_empty() {
            let error_messages: Vec<String> = errors.iter().map(|e| format!("  - {e}")).collect();
            panic!(
                "Valid config '{file_name}' failed schema validation:\n{}",
                error_messages.join("\n")
            );
        }
    }
}

/// Test that invalid configuration files fail schema validation
#[test]
fn test_invalid_configs_fail_validation() {
    let schema = load_schema();
    let invalid_dir = get_fixtures_path().join("invalid");

    // TODO: Add another invalid test case to ensure that unknown fields are caught.
    // `additionalProperties` needs to be set to false for this to work and we need
    // to explicitly allow the "$schema" field.
    let test_files = [
        "invalid_plugin.json",
        "invalid_category.json",
        "invalid_severity.json",
        "plugins_wrong_type.json",
        "globals_wrong_value.json",
        "globals_writeable_not_allowed.json",
        "env_wrong_type.json",
    ];

    for file_name in &test_files {
        let file_path = invalid_dir.join(file_name);
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("Failed to read {file_name}: {e}"));

        let instance: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {file_name} as JSON: {e}"));

        // Use iter_errors to check for validation errors
        let errors: Vec<_> = schema.iter_errors(&instance).collect();

        assert!(
            !errors.is_empty(),
            "Invalid config '{file_name}' unexpectedly passed schema validation"
        );
    }
}

/// Test that the schema itself is valid JSON Schema Draft 7
#[test]
fn test_schema_is_valid() {
    let schema: serde_json::Value =
        serde_json::from_str(&schema_content()).expect("Failed to parse generated schema as JSON");

    // Check that the schema has the expected $schema field
    assert_eq!(
        schema.get("$schema").and_then(|v| v.as_str()),
        Some("http://json-schema.org/draft-07/schema#"),
        "Schema should declare JSON Schema Draft 7"
    );

    // Check that we can compile it (this validates the schema structure)
    Validator::new(&schema).expect("Generated schema should be valid JSON Schema");
}
