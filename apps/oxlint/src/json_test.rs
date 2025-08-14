use crate::tsgolint::{TsGoLintInput, TsGoLintInputFile, TsGoLintRules};
use serde_json::json;

#[test]
fn test_json_serialization_formats() {
    // Test array format
    let array_input = TsGoLintInput {
        files: vec![TsGoLintInputFile {
            file_path: "/path/to/file.ts".to_string(),
            rules: TsGoLintRules::Array(vec!["no-floating-promises".to_string()]),
        }],
    };

    let array_json = serde_json::to_string_pretty(&array_input).unwrap();
    println!("Array format:\n{}", array_json);

    // Test object format
    let mut rules_object = serde_json::Map::new();
    rules_object.insert("no-floating-promises".to_string(), json!({}));

    let object_input = TsGoLintInput {
        files: vec![TsGoLintInputFile {
            file_path: "/path/to/file.ts".to_string(),
            rules: TsGoLintRules::Object(rules_object),
        }],
    };

    let object_json = serde_json::to_string_pretty(&object_input).unwrap();
    println!("Object format:\n{}", object_json);

    // Verify they can be parsed back
    let _: TsGoLintInput = serde_json::from_str(&array_json).unwrap();
    let _: TsGoLintInput = serde_json::from_str(&object_json).unwrap();
}

#[test]
fn test_object_format_with_configuration() {
    // Test object format with actual configuration
    let mut rules_object = serde_json::Map::new();
    rules_object.insert(
        "no-floating-promises".to_string(),
        json!({
            "ignoreVoid": true,
            "ignoreIIFE": true
        }),
    );
    rules_object.insert(
        "no-misused-promises".to_string(),
        json!({
            "checksVoidReturn": false,
            "checksConditionals": true
        }),
    );

    let input = TsGoLintInput {
        files: vec![TsGoLintInputFile {
            file_path: "/path/to/file.ts".to_string(),
            rules: TsGoLintRules::Object(rules_object),
        }],
    };

    let json_output = serde_json::to_string_pretty(&input).unwrap();
    println!("Object format with configuration:\n{}", json_output);

    // Verify it can be parsed back
    let parsed: TsGoLintInput = serde_json::from_str(&json_output).unwrap();
    assert_eq!(parsed.files.len(), 1);

    // Verify the rules are object format
    match &parsed.files[0].rules {
        TsGoLintRules::Object(rules) => {
            assert!(rules.contains_key("no-floating-promises"));
            assert!(rules.contains_key("no-misused-promises"));
        }
        TsGoLintRules::Array(_) => panic!("Expected object format"),
    }
}

#[test]
fn test_backward_compatibility_parsing() {
    // Test that we can parse the old array format
    let array_json = r#"{
        "files": [
            {
                "file_path": "/path/to/file.ts",
                "rules": ["no-floating-promises", "no-unsafe-call"]
            }
        ]
    }"#;

    let parsed: TsGoLintInput = serde_json::from_str(array_json).unwrap();
    match &parsed.files[0].rules {
        TsGoLintRules::Array(rules) => {
            assert_eq!(rules.len(), 2);
            assert!(rules.contains(&"no-floating-promises".to_string()));
            assert!(rules.contains(&"no-unsafe-call".to_string()));
        }
        TsGoLintRules::Object(_) => panic!("Expected array format"),
    }

    // Test that we can parse the new object format
    let object_json = r#"{
        "files": [
            {
                "file_path": "/path/to/file.ts",
                "rules": {
                    "no-floating-promises": {"ignoreVoid": true},
                    "no-unsafe-call": {}
                }
            }
        ]
    }"#;

    let parsed: TsGoLintInput = serde_json::from_str(object_json).unwrap();
    match &parsed.files[0].rules {
        TsGoLintRules::Object(rules) => {
            assert_eq!(rules.len(), 2);
            assert!(rules.contains_key("no-floating-promises"));
            assert!(rules.contains_key("no-unsafe-call"));
        }
        TsGoLintRules::Array(_) => panic!("Expected object format"),
    }
}
