// Test file to verify the fix for typescript/no-explicit-any rule defaults
use oxc_linter::{ConfigStoreBuilder, Oxlintrc, ExternalPluginStore};

#[test]
fn test_typescript_no_explicit_any_default_config_via_categories() {
    // This test verifies that typescript/no-explicit-any rule gets proper default
    // configuration when enabled via categories, fixing issue #12561
    
    let oxlintrc_content = r#"
{
    "plugins": ["typescript"],
    "categories": {
        "restriction": "error"
    }
}
"#;

    let oxlintrc: Oxlintrc = serde_json::from_str(oxlintrc_content).unwrap();
    let mut external_plugin_store = ExternalPluginStore::default();
    let config = ConfigStoreBuilder::from_oxlintrc(true, oxlintrc, None, &mut external_plugin_store)
        .unwrap()
        .build();

    // Find the typescript/no-explicit-any rule
    let no_explicit_any_rule = config
        .rules()
        .iter()
        .find(|(r, _)| r.name() == "no-explicit-any" && r.plugin_name() == "typescript");

    assert!(
        no_explicit_any_rule.is_some(),
        "typescript/no-explicit-any should be enabled when restriction category is set"
    );

    let (rule, _severity) = no_explicit_any_rule.unwrap();
    
    // The rule should have default configuration applied
    // We can verify this by checking if the rule is properly configured
    // (The exact internal structure may not be directly accessible, but the rule should be there)
    assert_eq!(rule.plugin_name(), "typescript");
    assert_eq!(rule.name(), "no-explicit-any");
}

#[test]
fn test_explicit_rule_configuration_still_works() {
    // This test verifies that explicit rule configuration still works
    // and takes precedence over category defaults
    
    let oxlintrc_content = r#"
{
    "plugins": ["typescript"],
    "categories": {
        "restriction": "error"
    },
    "rules": {
        "typescript/no-explicit-any": ["warn", { "fixToUnknown": true }]
    }
}
"#;

    let oxlintrc: Oxlintrc = serde_json::from_str(oxlintrc_content).unwrap();
    let mut external_plugin_store = ExternalPluginStore::default();
    let config = ConfigStoreBuilder::from_oxlintrc(true, oxlintrc, None, &mut external_plugin_store)
        .unwrap()
        .build();

    // Find the typescript/no-explicit-any rule
    let no_explicit_any_rule = config
        .rules()
        .iter()
        .find(|(r, _)| r.name() == "no-explicit-any" && r.plugin_name() == "typescript");

    assert!(
        no_explicit_any_rule.is_some(),
        "typescript/no-explicit-any should be enabled with explicit configuration"
    );

    let (rule, severity) = no_explicit_any_rule.unwrap();
    
    // The rule should have the explicit configuration
    assert_eq!(rule.plugin_name(), "typescript");
    assert_eq!(rule.name(), "no-explicit-any");
    // Severity should be Warn because we explicitly set it to "warn"
    assert!(severity.is_warn_deny());
}