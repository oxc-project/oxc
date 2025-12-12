//! Test to ensure rules with configuration options have proper documentation
//!
//! This test verifies that all linter rules with configuration options
//! have a schema value set in the declare_oxc_lint! macro.
//!
//! This helps ensure that users can understand how to configure rules properly.

// NOTE: You will need to run the tests with `--features ruledocs` or
// `--all-features` for this test file to run.
#![cfg(feature = "ruledocs")]

use lazy_regex::Regex;
use oxc_linter::{rules::RULES, table::RuleTable};
use rustc_hash::FxHashSet;
use schemars::r#gen;

/// Test to ensure that all rules with configuration options have proper documentation.
///
/// This test gets the full rule list programmatically, identifies rules with
/// configuration schemas, and verifies that they are configured to generate
/// configuration documentation.
#[test]
fn test_rules_with_custom_configuration_have_schema() {
    let mut failures = Vec::new();

    // Rules that have from_configuration, but no proper schema documentation yet.
    // These rules are temporarily allowed to not have schema docs.
    //
    // TODO: Remove rules from this list as they get fixed. Do NOT add new rules to this
    // list - newly-created rules should always be documented before being merged!
    let exceptions: &[&str] = &[
        // eslint
        "eslint/arrow-body-style",
        "eslint/func-names",
        "eslint/no-empty-function",
        "eslint/no-restricted-imports",
        "eslint/no-warning-comments",
        "eslint/yoda",
        // jest
        "jest/valid-title",
        // react
        "react/forbid-dom-props",
        "react/forbid-elements",
    ];

    let exception_set: FxHashSet<&str> = exceptions.iter().copied().collect();

    // Get the full rule list programmatically
    let mut generator = r#gen::SchemaGenerator::new(r#gen::SchemaSettings::default());
    let table = RuleTable::new(Some(&mut generator));

    // Build a map from rule name to RuleTableRow for easy lookup, filters
    // out rules that have no schema.
    // This is used to check which rules have schemas defined.
    let rules_with_schemas: FxHashSet<String> = table
        .sections
        .iter()
        .flat_map(|section| &section.rows)
        .filter(|row| row.schema.is_some())
        .map(|row| format!("{}/{}", row.plugin, row.name))
        .collect();

    // Regex to detect if a rule has configuration options in its debug output.
    //
    // Matches:
    // - Any usage of curly braces `{}` (struct configs)
    // - Any usage of square brackets `[]` (array configs)
    // - Nested content in parentheses like `PluginRuleName(RuleName(Foo))` (enum configs)
    //
    // It will NOT match simple wrapper patterns like `Foo(Bar)`, with no nested content.
    //
    // Examples of rules with config options:
    // - `UnicornPreferAt(PreferAt(PreferAtConfig { check_all_index_access: false, get_last_element_functions: [] }))`
    // - `PromiseNoReturnWrap(NoReturnWrap { allow_reject: false })`
    // - `VueDefineEmitsDeclaration(DefineEmitsDeclaration(TypeBased))`
    //
    // A rule with no configuration options would look like this, with no nesting past the first level:
    // - `UnicornPreferTopLevelAwait(PreferTopLevelAwait)`
    let config_regex = Regex::new(r"[{}\[\]]|\(\w+\([^)]+\)\)").unwrap();

    // Check each rule to see if it has configuration options but no schema
    for rule in RULES.iter() {
        let full_rule_name = format!("{}/{}", rule.plugin_name(), rule.name());

        // Skip if in exceptions list
        if exception_set.contains(full_rule_name.as_str()) {
            // Error if it is listed as an exception but has a schema defined
            if rules_with_schemas.contains(&full_rule_name) {
                failures.push(format!(
                    "Rule '{full_rule_name}' is in the exceptions list but has a schema defined.\n\
                     This rule has been fixed! Please remove it from the exceptions list."
                ));
            }
            continue;
        }

        // Check if this rule has configuration options by looking at the debug
        // output of its default values.
        //
        // This should work in all normal cases, but there may be a better option if we
        // can check which rules have `from_configuration` defined explicitly in their
        // source.
        let default_rule = rule.clone();
        let rule_debug = format!("{default_rule:?}");
        let rule_has_config_options = config_regex.is_match(&rule_debug);

        // Skip rules with config options that already have schemas.
        if rules_with_schemas.contains(&full_rule_name) && rule_has_config_options {
            continue;
        }

        // If a rule has a schema but no config options, fail. Something has likely broken about the way
        // we're checking for this.
        if rules_with_schemas.contains(&full_rule_name) && !rule_has_config_options {
            failures.push(format!(
                "Rule '{full_rule_name}' has a schema defined but no configuration options.\n\
                 Please add configuration options to this rule.",
            ));
        }

        // Fail here if the rule has config options but no schema.
        if rule_has_config_options {
            failures.push(format!(
                "Rule '{full_rule_name}' accepts configuration options but has no schema defined.\n\
                 Please see the oxc website for info on adding config option schemas and docs to this rule.\n\
                 https://oxc.rs/docs/contribute/linter/adding-rules.html",
            ));
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
