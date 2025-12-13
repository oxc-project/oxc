//! Test to ensure rule configuration consistency
//!
//! This test verifies that for all linter rules, the default configuration
//! obtained via `Default::default()` matches the configuration obtained via
//! `from_configuration(null)`. This helps prevent bugs where rules behave
//! differently depending on how they are initialized.

use oxc_linter::rules::RULES;
use serde_json::Value;

/// Test to ensure consistency between `rule::default()` and `rule::from_configuration(null)`.
///
/// This test helps prevent issues where a rule's default configuration differs from
/// its configuration when initialized with a null value. Both should produce identical
/// results to maintain consistency in rule behavior.
///
/// See issue #12718 for an example of why this test is important.
#[test]
fn test_rule_default_matches_from_configuration_null() {
    let mut failures = Vec::new();

    // Rules that are known to have configuration mismatches and need to be fixed
    // TODO: These should be removed as the rules are fixed to have consistent defaults
    // When fixing a rule, ensure that either:
    // 1. The Default implementation returns the same values as from_configuration(null), or
    // 2. The from_configuration method is updated to return Default::default() when given null
    let exceptions = [];

    // Iterate through all available linter rules
    for rule in RULES.iter() {
        let plugin_name = rule.plugin_name();
        let rule_name = rule.name();

        // Skip rules that are known to have issues
        if exceptions.contains(&format!("{plugin_name}/{rule_name}").as_str()) {
            continue;
        }

        // Get the default configuration using rule::default
        let default_rule = rule.clone();

        // Get the configuration using rule::from_configuration(null)
        let null_configured_rule = rule.from_configuration(Value::Null);

        // Compare the two configurations
        // Since RuleEnum doesn't implement PartialEq for the inner rule types,
        // we need to compare them by running them and checking their behavior
        // For now, we'll use the debug representation as a proxy
        let default_debug = format!("{default_rule:?}");
        let null_debug = format!("{null_configured_rule:?}");

        if default_debug != null_debug {
            failures.push(format!(
                "Rule '{plugin_name}/{rule_name}' has different configurations between default() and from_configuration(null).\n\
                 Default: {default_debug}\n\
                 From null: {null_debug}",

            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Found {} rules with configuration mismatches:\n\n{}\n\n\
To fix these issues:\n\
1. Update the Default implementation to match from_configuration(null), or\n\
2. Update from_configuration to return Default::default() when given null\n\
3. Add the rule to the exceptions list above if the mismatch is intentional",
        failures.len(),
        failures.join("\n\n")
    );
}
