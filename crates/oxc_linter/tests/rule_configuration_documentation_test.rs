//! Test to ensure rules with configuration options have proper documentation
//!
//! This test verifies that all linter rules with configuration options
//! have a "Configuration" section in their generated documentation.
//! This helps ensure that users can understand how to configure rules properly.

#![cfg(feature = "ruledocs")]

use oxc_linter::table::RuleTable;
use rustc_hash::FxHashSet;
use schemars::{r#gen, schema::Schema};

/// Test to ensure that all rules with configuration options have proper documentation.
///
/// This test gets the full rule list programmatically, identifies rules with
/// configuration schemas, and verifies that they are configured to generate
/// configuration documentation.
#[test]
fn test_rules_with_custom_configuration_have_schema() {
    let mut failures = Vec::new();

    // Rules that have from_configuration but no proper schema documentation yet.
    // These rules are temporarily allowed to not have schema docs.
    // TODO: Remove rules from this list as they get fixed. Do NOT add new rules to this
    // list - newly-created rules should always be documented before being merged!
    let exceptions: &[&str] = &[
        // eslint
        "eslint/arrow-body-style",
        "eslint/default-case",
        "eslint/func-names",
        "eslint/new-cap",
        "eslint/no-cond-assign",
        "eslint/no-else-return",
        "eslint/no-empty-function",
        "eslint/no-fallthrough",
        "eslint/no-restricted-globals",
        "eslint/no-restricted-imports",
        "eslint/no-warning-comments",
        "eslint/yoda",
        // jest
        "jest/consistent-test-it",
        "jest/valid-title",
        // jsdoc
        "jsdoc/require-param",
        "jsdoc/require-returns",
        // promise
        "promise/param-names",
        // react
        "react/forbid-dom-props",
        "react/forbid-elements",
        "react/jsx-handler-names",
        "react/prefer-es6-class",
        "react/state-in-constructor",
        // typescript
        "typescript/ban-ts-comment",
        "typescript/consistent-type-imports",
        // unicorn
        "unicorn/catch-error-name",
        "unicorn/filename-case",
        "unicorn/switch-case-braces",
        // vue
        "vue/define-emits-declaration",
        "vue/define-props-declaration",
    ];

    let exception_set: FxHashSet<&str> = exceptions.iter().copied().collect();

    // Get the full rule list programmatically
    let mut generator = r#gen::SchemaGenerator::new(r#gen::SchemaSettings::default());
    let table = RuleTable::new(Some(&mut generator));

    // Check each rule to see if it has a schema and whether it would generate config docs
    for rule in table.sections.iter().flat_map(|section| &section.rows) {
        let rule_name = format!("{}/{}", rule.plugin, rule.name);

        // Check if this rule has a schema
        let has_schema = rule.schema.is_some();

        if has_schema {
            // Rule has a schema - verify it will generate proper documentation
            if let Some(schema) = &rule.schema {
                let resolved = generator.dereference(schema).unwrap_or(schema);

                // Check if this would generate a configuration section
                // Following the same logic as render_rule_docs_page
                let will_generate_docs = if let Schema::Object(schema_obj) = resolved {
                    use schemars::schema::{InstanceType, SingleOrVec};

                    // Check if the schema has meaningful content that would generate docs
                    let has_properties = schema_obj.object.as_ref().is_some_and(|obj| {
                        !obj.properties.is_empty() || obj.additional_properties.is_some()
                    });

                    let has_subschemas =
                        schema_obj.subschemas.is_some() || schema_obj.enum_values.is_some();

                    let is_array = schema_obj.instance_type.as_ref().map_or(false, |ty| {
                        matches!(ty, SingleOrVec::Single(t) if **t == InstanceType::Array)
                            || matches!(ty, SingleOrVec::Vec(types) if types.contains(&InstanceType::Array))
                    });

                    has_properties || has_subschemas || is_array
                } else {
                    false
                };

                if !will_generate_docs && !exception_set.contains(rule_name.as_str()) {
                    failures.push(format!(
                        "Rule '{rule_name}' has a schema but it won't generate configuration documentation.\n\
                         The schema may be empty or improperly configured."
                    ));
                }
            }
        }
    }

    // Verify all exceptions actually exist in the rule table
    let all_rules: FxHashSet<String> = table
        .sections
        .iter()
        .flat_map(|section| &section.rows)
        .map(|rule| format!("{}/{}", rule.plugin, rule.name))
        .collect();

    for &exception_rule in exceptions {
        if !all_rules.contains(exception_rule) {
            failures.push(format!(
                "Exception rule '{exception_rule}' is in the exceptions list but does not exist in the linter.\n\
                 This rule may have been removed or renamed. Please remove it from the exceptions list."
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Found {} configuration documentation issues:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

/// Test to verify react/state-in-constructor is properly handled.
/// This rule has from_configuration but no schema documentation yet,
/// so it's in the exceptions list.
#[test]
fn test_state_in_constructor_in_exceptions() {
    let mut generator = r#gen::SchemaGenerator::new(r#gen::SchemaSettings::default());
    let table = RuleTable::new(Some(&mut generator));

    let rule = table
        .sections
        .iter()
        .flat_map(|section| &section.rows)
        .find(|rule| rule.plugin == "react" && rule.name == "state-in-constructor");

    assert!(rule.is_some(), "react/state-in-constructor should exist in the rule table");

    let rule = rule.unwrap();

    // This rule should NOT have a schema yet (that's why it's in exceptions)
    assert!(
        rule.schema.is_none(),
        "react/state-in-constructor is in the exceptions list, indicating it doesn't have a schema yet. \
         If it now has a schema, remove it from the exceptions list!"
    );
}
