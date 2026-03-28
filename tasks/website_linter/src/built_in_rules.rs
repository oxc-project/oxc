use std::collections::BTreeSet;

use rustc_hash::FxHashMap;

use oxc_linter::rules::RULES;

#[expect(clippy::print_stdout)]
pub fn print_built_in_rules_ts() {
    println!("{}", render_built_in_rules_ts());
}

fn render_built_in_rules_ts() -> String {
    let mut rules = RULES
        .iter()
        .map(|rule| {
            (
                rule.plugin_name().to_string(),
                rule.name().to_string(),
                rule.category().as_str().to_string(),
            )
        })
        .collect::<BTreeSet<_>>();

    let jest_categories = RULES
        .iter()
        .filter(|rule| rule.plugin_name() == "jest")
        .map(|rule| (rule.name(), rule.category().as_str()))
        .collect::<FxHashMap<_, _>>();

    let vitest_compatible_jest_rules: Vec<String> = serde_json::from_str(include_str!(
        "../../../crates/oxc_linter/data/vitest_compatible_jest_rules.json"
    ))
    .expect("Failed to parse vitest_compatible_jest_rules.json");

    for rule_name in vitest_compatible_jest_rules {
        if let Some(category) = jest_categories.get(rule_name.as_str()) {
            rules.insert(("vitest".to_string(), rule_name, (*category).to_string()));
        }
    }

    if let Some(category) = jest_categories.get("no-restricted-jest-methods") {
        rules.insert((
            "vitest".to_string(),
            "no-restricted-vi-methods".to_string(),
            (*category).to_string(),
        ));
    }

    let mut out = String::new();
    out.push_str("/*\n");
    out.push_str(" * This file is generated from Rust built-in rule metadata.\n");
    out.push_str(" * Run `just linter-built-in-rules-ts` to regenerate.\n");
    out.push_str(" */\n\n");

    out.push_str("export const allRules = [\n");
    for (plugin, rule, category) in rules {
        out.push_str("  { plugin: \"");
        out.push_str(&plugin);
        out.push_str("\", rule: \"");
        out.push_str(&rule);
        out.push_str("\", category: \"");
        out.push_str(&category);
        out.push_str("\" },\n");
    }
    out.push_str("] as const;\n\n");
    out.push_str("export type BuiltInRule = (typeof allRules)[number];\n");
    out.push_str("export type BuiltInRulePlugin = BuiltInRule[\"plugin\"];\n");
    out.push_str("export type BuiltInRuleCategory = BuiltInRule[\"category\"];\n");

    out
}

#[test]
fn test_built_in_rules_ts_is_up_to_date() {
    use project_root::get_project_root;
    use std::fs;

    let path =
        get_project_root().unwrap().join("apps/oxlint/src-js/package/built-in-rules.generated.ts");
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let generated = render_built_in_rules_ts();

    assert_eq!(
        existing.trim(),
        generated.trim(),
        "built-in rules metadata is outdated. Run `just linter-built-in-rules-ts`.",
    );
}
