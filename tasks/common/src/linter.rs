use std::path::{Path, PathBuf};

/// Source directory for a built-in lint plugin, whether extracted or still in the linter.
pub fn lint_plugin_path(root: &Path, plugin: &str) -> PathBuf {
    let external = root.join(format!("crates/oxc_linter_rules_{plugin}/src"));
    if external.is_dir() { external } else { root.join("crates/oxc_linter/src/rules").join(plugin) }
}

/// Source file for a lint rule. Rules can be single files or directories with `mod.rs`.
pub fn lint_rule_path(root: &Path, plugin: &str, rule: &str) -> PathBuf {
    let path = lint_plugin_path(root, plugin).join(rule);
    if path.is_dir() { path.join("mod.rs") } else { path.with_extension("rs") }
}
