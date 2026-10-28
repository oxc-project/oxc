use std::path::Path;

use convert_case::{Case, Casing};

/// Represents a lint rule entry in the `declare_all_lint_rules!` macro.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct RuleEntry<'e> {
    /// Crate owning an extracted rule, or `None` for rules still in `oxc_linter`.
    pub external_crate: Option<&'e str>,
    /// The module name of the rule's plugin, like `eslint` in `eslint::no_debugger::NoDebugger`.
    pub plugin_module_name: &'e str,
    /// The rule's module name, like `no_debugger` in `eslint::no_debugger:NoDebugger`.
    pub rule_module_name: &'e str,
    /// Whether the rule overrides `Rule::from_configuration`.
    pub has_custom_from_configuration: bool,
    /// Whether the rule overrides `Rule::to_configuration`.
    pub has_custom_to_configuration: bool,
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

        let Some(plugin) = current_plugin.filter(|plugin| *plugin != "shared") else {
            continue;
        };
        let declaration = line.strip_suffix(';').and_then(|line| {
            if let Some(rule) = line.strip_prefix("pub mod ") {
                Some((None, rule))
            } else {
                let (owner, rule) = line.strip_prefix("pub use ")?.split_once("::")?;
                owner.starts_with("oxc_linter_rules_").then_some((Some(owner), rule))
            }
        });
        if let Some((external_crate, rule_name)) = declaration {
            rule_entries.push(RuleEntry {
                external_crate,
                plugin_module_name: plugin,
                rule_module_name: rule_name,
                // Preserve the existing generated behavior unless codegen confirms that the rule
                // uses the default implementation.
                has_custom_from_configuration: true,
                has_custom_to_configuration: true,
            });
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
    let path = oxc_tasks_common::linter::lint_rule_path(
        root,
        rule.plugin_module_name,
        rule.rule_module_name,
    );
    path.is_file().then_some(path)
}

#[cfg(test)]
mod tests {
    use super::get_all_rules;

    #[test]
    fn preserves_registry_order_across_crates() {
        let rules = get_all_rules(
            "pub(crate) mod eslint {\n\
                 pub mod no_debugger;\n\
             }\n\
             pub(crate) mod react_perf {\n\
                 pub use oxc_linter_rules_react_perf::jsx_no_jsx_as_prop;\n\
             }\n\
             pub(crate) mod shared {\n\
                 pub mod jest_vitest;\n\
             }",
        );
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].rule_module_name, "no_debugger");
        assert_eq!(rules[0].external_crate, None);
        assert_eq!(rules[1].plugin_module_name, "react_perf");
        assert_eq!(rules[1].rule_module_name, "jsx_no_jsx_as_prop");
        assert_eq!(rules[1].external_crate, Some("oxc_linter_rules_react_perf"));
    }
}
