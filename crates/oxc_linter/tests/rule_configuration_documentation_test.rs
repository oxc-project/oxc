//! Test to ensure rules with configuration options have proper documentation
//!
//! This test verifies that all linter rules with configuration options
//! have a "Configuration" section in their generated documentation.
//! This helps ensure that users can understand how to configure rules properly.

#![cfg(feature = "ruledocs")]

use cow_utils::CowUtils;
use rustc_hash::FxHashSet;

// Recursively scan rule files
fn scan_rules_dir(
    dir: &std::path::Path,
    base_dir: &std::path::Path,
    results: &mut Vec<String>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            scan_rules_dir(&path, base_dir, results)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = std::fs::read_to_string(&path)?;

            let has_declare_lint = content.contains("declare_oxc_lint!(");
            let has_from_config = content.contains("fn from_configuration(");
            // Look for "config =" as a parameter in declare_oxc_lint macro
            // Use regex to be more precise
            let has_config_param = content.contains("declare_oxc_lint!")
                && content.split("declare_oxc_lint!").any(|section| {
                    // Check if this section (after declare_oxc_lint!) contains "config ="
                    // before the closing of the macro (the semicolon after the paren)
                    if let Some(macro_end) = section.find(");") {
                        let macro_content = &section[..macro_end];
                        macro_content.contains("config =")
                    } else {
                        false
                    }
                });

            if has_declare_lint && has_from_config && !has_config_param {
                let rel_path = path.strip_prefix(base_dir).unwrap();
                results.push(rel_path.to_string_lossy().to_string());
            }
        }
    }
    Ok(())
}

/// Test to ensure that all rules with `from_configuration` implementations
/// also have a schema and proper documentation.
///
/// This test:
/// 1. Scans source code to find rules with from_configuration but no config = parameter
/// 2. Generates the actual website documentation
/// 3. Verifies that rules with from_configuration have a Configuration section in their docs
///
/// The Configuration section in generated docs is auto-created from the schema,
/// so rules with custom configuration must have a schema defined.
#[test]
fn test_rules_with_custom_configuration_have_schema() {
    let mut failures = Vec::new();

    // Rules that have from_configuration but no schema yet.
    // These were found by running the source code scanner in this test.
    // TODO: Remove rules from this list as they get fixed. Do NOT add new rules to this
    // list, newly-created rules should always be documented before being merged!
    let exceptions: &[&str] = &[
        // eslint (15)
        "eslint/arrow-body-style",
        "eslint/default-case",
        "eslint/func-names",
        "eslint/new-cap",
        "eslint/no-bitwise",
        "eslint/no-cond-assign",
        "eslint/no-console",
        "eslint/no-else-return",
        "eslint/no-empty-function",
        "eslint/no-fallthrough",
        "eslint/no-inner-declarations",
        "eslint/no-restricted-globals",
        "eslint/no-restricted-imports",
        "eslint/no-self-assign",
        "eslint/no-warning-comments",
        "eslint/yoda",
        // jest (3)
        "jest/consistent-test-it",
        "jest/prefer-lowercase-title",
        "jest/valid-title",
        // jsdoc (2)
        "jsdoc/require-param",
        "jsdoc/require-returns",
        // jsx_a11y (3)
        "jsx_a11y/label-has-associated-control",
        "jsx_a11y/media-has-caption",
        "jsx_a11y/no-noninteractive-tabindex",
        // promise (3)
        "promise/no-callback-in-promise",
        "promise/param-names",
        "promise/spec-only",
        // react (4)
        "react/forbid-dom-props",
        "react/forbid-elements",
        "react/jsx-handler-names",
        "react/prefer-es6-class",
        // typescript (3)
        "typescript/ban-ts-comment",
        "typescript/consistent-generic-constructors",
        "typescript/consistent-type-imports",
        // unicorn (3)
        "unicorn/catch-error-name",
        "unicorn/filename-case",
        "unicorn/switch-case-braces",
        // vue (2)
        "vue/define-emits-declaration",
        "vue/define-props-declaration",
    ];

    let exception_set: FxHashSet<&str> = exceptions.iter().copied().collect();

    // Step 1: Scan source code to find rules with from_configuration but no config =
    let workspace_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let rules_dir = workspace_root.join("crates/oxc_linter/src/rules");

    let mut rules_with_from_config_no_schema = Vec::new();

    scan_rules_dir(&rules_dir, &rules_dir, &mut rules_with_from_config_no_schema)
        .expect("Failed to scan rules directory");

    // Convert file paths to rule names (plugin/rule-name format)
    let mut rules_needing_schema: Vec<String> = rules_with_from_config_no_schema
        .iter()
        .filter_map(|path| {
            // Path format: "plugin/rule_file.rs" or "plugin/rule_name/mod.rs"
            // Always use '/' as separator regardless of OS
            let normalized_path = if std::path::MAIN_SEPARATOR == '/' {
                path.as_str()
            } else {
                &path.cow_replace(std::path::MAIN_SEPARATOR, "/")
            };
            let parts: Vec<&str> = normalized_path.split('/').collect();
            if parts.len() >= 2 {
                let plugin = parts[0];
                let rule_file = if parts.len() == 2 {
                    // plugin/rule_file.rs
                    parts[1].strip_suffix(".rs")?
                } else {
                    // plugin/rule_name/mod.rs
                    parts[1]
                };
                // Convert underscores to hyphens for rule name
                let rule_name = if rule_file.contains('_') {
                    rule_file.cow_replace('_', "-").into_owned()
                } else {
                    rule_file.to_string()
                };
                Some(format!("{plugin}/{rule_name}"))
            } else {
                None
            }
        })
        .collect();

    rules_needing_schema.sort();
    rules_needing_schema.dedup();

    // Step 2: Generate documentation
    let temp_dir = std::env::temp_dir().join(format!("oxc-rule-docs-test-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    let git_ref = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to get git ref")
        .stdout;
    let git_ref = String::from_utf8_lossy(&git_ref).trim().to_string();

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "website",
            "--",
            "linter-rules",
            "--rule-docs",
            temp_dir.to_str().unwrap(),
            "--git-ref",
            &git_ref,
        ])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to generate documentation");

    assert!(
        output.status.success(),
        "Failed to generate documentation:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Step 3: Check generated documentation for Configuration sections
    for rule_name in &rules_needing_schema {
        if exception_set.contains(rule_name.as_str()) {
            // Expected - this rule is a known exception
            continue;
        }

        // Read the generated markdown file
        let doc_file = temp_dir.join(format!("{rule_name}.md"));

        if !doc_file.exists() {
            failures.push(format!(
                "Rule '{rule_name}' has from_configuration in source but no documentation file was generated at {}",
                doc_file.display()
            ));
            continue;
        }

        let doc_content = std::fs::read_to_string(&doc_file)
            .unwrap_or_else(|_| panic!("Failed to read {}", doc_file.display()));

        // Check if the documentation has a Configuration section
        let has_config_section = doc_content.contains("## Configuration");

        if !has_config_section {
            failures.push(format!(
                "Rule '{rule_name}' has from_configuration in source code but no Configuration section in generated docs.\n\
                 This means the rule accepts configuration options but they are not documented.\n\
                 \n\
                 To fix:\n\
                 1. Create a config struct that derives JsonSchema (and typically Deserialize, Debug, Clone)\n\
                 2. Add `config = YourConfigStruct` to the declare_oxc_lint! macro\n\
                 3. Add documentation comments to the config struct and its fields\n\
                 \n\
                 Or if this rule should not have configuration docs, add it to the exceptions list in this test.\n\
                 \n\
                 Example:\n\
                 #[derive(Debug, Clone, JsonSchema)]\n\
                 #[serde(rename_all = \"camelCase\", default)]\n\
                 pub struct YourRuleConfig {{\n\
                     /// Description of this option\n\
                     some_option: bool,\n\
                 }}\n\
                 \n\
                 declare_oxc_lint!(\n\
                     YourRule,\n\
                     plugin,\n\
                     category,\n\
                     config = YourRuleConfig,\n\
                 );",
            ));
        }
    }

    // Verify exception list is accurate
    for &exception_rule in exceptions {
        if !rules_needing_schema.contains(&exception_rule.to_string()) {
            failures.push(format!(
                "Exception rule '{exception_rule}' is in the exceptions list but was not found by source code analysis.\n\
                 This rule may have been fixed or removed. Please remove it from the exceptions list."
            ));
        }
    }

    // Clean up temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    assert!(
        failures.is_empty(),
        "Found {} rules with configuration issues:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}
