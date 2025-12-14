//! Test to ensure JSON Schema validation is working correctly
//!
//! This test validates that the generated JSON schema for oxlintrc configuration
//! files correctly validates both valid and invalid configurations. This helps
//! prevent regressions in the schema generation process.

use std::fs;
use std::path::PathBuf;

use insta::assert_snapshot;
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

/// Return a sorted list of JSON files in `dir`
fn list_json_files(dir: &PathBuf) -> Vec<std::path::PathBuf> {
    let mut files: Vec<std::path::PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("Failed to read dir {}: {e}", dir.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();

    files.sort_by_key(|p| p.file_name().map(|s| s.to_string_lossy().to_string()));
    files
}

/// Test that valid configuration files pass schema validation
#[test]
fn test_valid_configs_pass_validation() {
    let schema = load_schema();
    let valid_dir = get_fixtures_path().join("valid");

    // Get all json files in the valid fixture directory.
    let test_files = list_json_files(&valid_dir);

    for file_path in &test_files {
        let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap();
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
    // Get all json files in the invalid fixture directory.
    let test_files = list_json_files(&invalid_dir);

    for file_path in &test_files {
        let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap();
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("Failed to read {file_name}: {e}"));

        let instance: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {file_name} as JSON: {e}"));

        // Collect validation errors
        let errors: Vec<_> = schema.iter_errors(&instance).collect();

        // Ensure there are errors
        assert!(
            !errors.is_empty(),
            "Invalid config '{file_name}' unexpectedly passed schema validation"
        );

        // Snapshot the invalid JSON content and human-readable error messages together
        let error_messages: String =
            errors.iter().map(|e| format!("- {e}")).collect::<Vec<_>>().join("\n");

        let snapshot_body =
            format!("File: {file_name}\n\n.oxlintrc.json:\n{content}\nErrors:\n{error_messages}\n");

        // Name snapshots by file to keep them stable and readable
        let snap_name = format!("invalid_{file_name}_errors");
        insta::with_settings!({ prepend_module_to_snapshot => false }, {
            assert_snapshot!(snap_name, snapshot_body);
        });
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
