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
/// of `RuleEntry` by scanning the module declarations.
pub fn get_all_rules(contents: &str) -> Vec<RuleEntry<'_>> {
    let mut rule_entries = Vec::new();
    let mut current_plugin: Option<&str> = None;

    for line in contents.lines() {
        let line = line.trim();

        // Detect plugin module start: `pub(crate) mod eslint {` or `pub(crate) mod typescript {`
        if line.starts_with("pub(crate) mod ") && line.ends_with(" {") {
            let module_name =
                line.strip_prefix("pub(crate) mod ").and_then(|s| s.strip_suffix(" {"));
            current_plugin = module_name;
            continue;
        }

        // Detect end of plugin module
        if line == "}" {
            current_plugin = None;
            continue;
        }

        // Inside a plugin module, detect rule module: `pub mod no_debugger;`
        if let Some(plugin) = current_plugin
            && line.starts_with("pub mod ")
            && line.ends_with(';')
            && let Some(rule_name) = line.strip_prefix("pub mod ").and_then(|s| s.strip_suffix(';'))
        {
            rule_entries
                .push(RuleEntry { plugin_module_name: plugin, rule_module_name: rule_name });
        }
    }

    // Preserve declaration order - do not sort
    rule_entries
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
