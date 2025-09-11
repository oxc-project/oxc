use std::path::Path;

use convert_case::{Case, Casing};

/// Represents a lint rule entry in the `declare_all_lint_rules!` macro.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct RuleEntry<'e> {
    /// The module name of the rule's plugin, like `eslint` in `eslint::no_debugger::NoDebugger`.
    pub plugin_module_name: &'e str,
    /// The rule's module name, like `no_debugger` in `eslint::no_debugger:NoDebugger`.
    pub rule_module_name: &'e str,
}

impl RuleEntry<'_> {
    /// Get the rule's struct name, like `NoDebugger` in `eslint::no_debugger::NoDebugger`.
    pub fn rule_struct_name(&self) -> String {
        self.rule_module_name.to_case(Case::Pascal)
    }
}

/// Parses `crates/oxc_linter/src/rules.rs` to extract all lint rule declarations into a list
/// of `RuleEntry`.
pub fn get_all_rules(contents: &str) -> std::io::Result<Vec<RuleEntry<'_>>> {
    let start_marker = "oxc_macros::declare_all_lint_rules!";
    let start = contents.find(start_marker).ok_or_else(|| {
        std::io::Error::other("could not find declare_all_lint_rules macro invocation")
    })?;

    let body = &contents[start..];

    // Collect (module path, struct name) pairs. Do NOT deduplicate by struct name because
    // different plugins may have rules with the same struct name.
    let mut rule_entries = Vec::new();
    for line in body.lines().skip(1) {
        let line = line.trim();
        if line.contains('}') {
            break;
        }
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if !line.ends_with(',') {
            continue;
        }
        let path = &line[..line.len() - 1];
        let parts = path.split("::").collect::<Vec<_>>();
        if parts.len() != 2 {
            continue;
        }
        let Some(plugin_module_name) = parts.first() else { continue };
        let Some(rule_module_name) = parts.get(1) else { continue };
        rule_entries.push(RuleEntry { plugin_module_name, rule_module_name });
    }
    // Sort deterministically
    rule_entries.sort_unstable();

    Ok(rule_entries)
}

/// Given a rule entry, attempt to find its corresponding source file path
pub fn find_rule_source_file(root: &Path, rule: &RuleEntry) -> Option<std::path::PathBuf> {
    // A rule path corresponds to:
    // 1) `crates/oxc_linter/src/rules/<plugin>/<rule>.rs`
    // 2) `crates/oxc_linter/src/rules/<plugin>/<rule>/mod.rs`
    let rules_path = root.join("crates/oxc_linter/src/rules").join(rule.plugin_module_name);

    let direct_path = rules_path.join(format!("{}.rs", rule.rule_module_name));
    if direct_path.exists() {
        return Some(direct_path);
    }

    let mod_path = rules_path.join(rule.rule_module_name).join("mod.rs");
    if mod_path.exists() {
        return Some(mod_path);
    }

    None
}
