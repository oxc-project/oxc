//! Test to ensure every rule declares version metadata.
//!
//! This test verifies that all linter rules expose `version = ...` metadata
//! through `declare_oxc_lint!`.

use oxc_linter::rules::RULES;
use rustc_hash::FxHashSet;

const EXCEPTIONS: &[&str] = &[];

fn is_valid_rule_version(version: &str) -> bool {
    version == "next" || is_stable_rule_version(version)
}

fn is_stable_rule_version(version: &str) -> bool {
    let mut parts = version.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(major), Some(minor), Some(patch), None)
            if [major, minor, patch]
                .into_iter()
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    )
}

fn collect_unknown_exception_failures<'a>(
    exceptions: impl IntoIterator<Item = &'a str>,
    all_rules: &FxHashSet<String>,
) -> Vec<String> {
    let mut failures = Vec::new();

    for exception_rule in exceptions {
        if !all_rules.contains(exception_rule) {
            failures.push(format!(
                "Exception rule '{exception_rule}' is in the exceptions list but does not exist in the linter.\n\
                 This rule may have been removed or renamed. Please remove it from the exceptions list."
            ));
        }
    }

    failures
}

#[test]
fn test_all_rules_have_version_metadata() {
    let mut failures = Vec::new();

    // NOTE: This should NOT ever have any values. All rules should declare
    // version metadata so the docs and release workflow can rely on it.
    let exception_set: FxHashSet<&str> = EXCEPTIONS.iter().copied().collect();
    let all_rules: FxHashSet<String> =
        RULES.iter().map(|rule| format!("{}/{}", rule.plugin_name(), rule.name())).collect();

    for rule in RULES.iter() {
        let full_rule_name = format!("{}/{}", rule.plugin_name(), rule.name());

        if exception_set.contains(full_rule_name.as_str()) {
            if rule.version().is_some() {
                failures.push(format!(
                    "Rule '{full_rule_name}' is in the exceptions list but already declares version metadata.\n\
                     This rule has been fixed! Please remove it from the exceptions list."
                ));
            }
            continue;
        }

        match rule.version() {
            None => failures.push(format!(
                "Rule '{full_rule_name}' is missing version metadata.\n\
                 Please add `version = \"x.y.z\"` for stable rules or `version = \"next\"` for unreleased rules."
            )),
            Some(version) if !is_valid_rule_version(version) => failures.push(format!(
                "Rule '{full_rule_name}' has invalid version metadata: `{version}`.\n\
                 Please use `version = \"x.y.z\"` for stable rules or `version = \"next\"` for unreleased rules."
            )),
            Some(_) => {}
        }
    }

    failures.extend(collect_unknown_exception_failures(EXCEPTIONS.iter().copied(), &all_rules));

    assert!(
        failures.is_empty(),
        "Found {} rule version metadata issues:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

#[test]
fn test_valid_rule_version_formats() {
    assert!(is_valid_rule_version("1.60.0"));
    assert!(is_valid_rule_version("next"));

    assert!(!is_valid_rule_version(""));
    assert!(!is_valid_rule_version("1"));
    assert!(!is_valid_rule_version("1.60"));
    assert!(!is_valid_rule_version("1.60.0.1"));
    assert!(!is_valid_rule_version("v1.60.0"));
    assert!(!is_valid_rule_version("1.60.x"));
}

#[test]
fn test_unknown_exception_rules_are_reported() {
    let all_rules = FxHashSet::from_iter(["eslint/no-debugger".to_string()]);
    let failures = collect_unknown_exception_failures(
        ["eslint/no-debugger", "eslint/missing-rule"],
        &all_rules,
    );

    assert_eq!(failures.len(), 1);
    assert!(failures[0].contains("eslint/missing-rule"));
}
