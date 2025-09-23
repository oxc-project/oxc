#![allow(clippy::print_stdout)]

use crate::{
    if_else_detector::IfElseKindDetector,
    let_else_detector::LetElseDetector,
    match_detector::MatchDetector,
    node_type_set::NodeTypeSet,
    rules::{RuleEntry, find_rule_source_file, get_all_rules},
    utils::{find_impl_function, find_rule_impl_block},
};
use std::{
    fmt::Write as _,
    fs,
    io::{self, Write as _},
    process::{Command, Stdio},
};
use syn::File;

mod if_else_detector;
mod let_else_detector;
mod match_detector;
mod node_type_set;
mod rules;
mod utils;

fn main() -> io::Result<()> {
    generate_rule_runner_impls()
}

/// # Errors
/// Returns `io::Error` if file operations fail.
pub fn generate_rule_runner_impls() -> io::Result<()> {
    let root = project_root::get_project_root()
        .map_err(|e| std::io::Error::other(format!("could not find project root: {e}")))?;

    let rules_file_contents = fs::read_to_string(root.join("crates/oxc_linter/src/rules.rs"))?;
    let rule_entries = get_all_rules(&rules_file_contents)?;

    let mut out = String::new();
    out.push_str("// Auto-generated code, DO NOT EDIT DIRECTLY!\n");
    out.push_str("// To regenerate: `cargo run -p oxc_linter_codegen`\n\n");
    out.push_str("#![allow(clippy::needless_pass_by_value)]\n\n");
    out.push_str("use oxc_ast::AstType;\n");
    out.push_str("use oxc_semantic::AstTypesBitset;\n\n");
    out.push_str("use crate::rule::RuleRunner;\n\n");

    for rule in &rule_entries {
        // Try to open the rule source file and use syn to detect node types
        let mut detected_types: NodeTypeSet = NodeTypeSet::new();
        if let Some(src_path) = find_rule_source_file(&root, rule)
            && let Ok(src_contents) = fs::read_to_string(&src_path)
            && let Ok(file) = syn::parse_file(&src_contents)
            && let Some(node_types) = detect_top_level_node_types(&file, rule)
        {
            detected_types.extend(node_types);
        }

        let node_types_init = if detected_types.is_empty() {
            "None".to_string()
        } else {
            format!("Some(&{})", detected_types.to_ast_type_bitset_string())
        };

        write!(
            out,
            r"
impl RuleRunner for crate::rules::{plugin_module}::{rule_module}::{rule_struct} {{
    const NODE_TYPES: Option<&AstTypesBitset> = {node_types_init};
}}
            ",
            plugin_module = rule.plugin_module_name,
            rule_module = rule.rule_module_name,
            rule_struct = rule.rule_struct_name(),
        )
        .unwrap();
    }

    let formatted_out = rust_fmt(&out);

    let target_path = root.join("crates/oxc_linter/src/generated/rule_runner_impls.rs");
    fs::write(&target_path, formatted_out)?;
    println!("Generated {} impls into {}", rule_entries.len(), target_path.display());

    Ok(())
}

/// Detect the top-level node types used in a lint rule file by analyzing the Rust AST with `syn`.
/// Returns `Some(bitset)` if at least one node type can be determined, otherwise `None`.
fn detect_top_level_node_types(file: &File, rule: &RuleEntry) -> Option<NodeTypeSet> {
    let rule_impl = find_rule_impl_block(file, &rule.rule_struct_name())?;
    let run_func = find_impl_function(rule_impl, "run")?;

    let node_types = LetElseDetector::from_run_func(run_func);
    if let Some(node_types) = node_types
        && !node_types.is_empty()
    {
        return Some(node_types);
    }

    let node_types = MatchDetector::from_run_func(run_func);
    if let Some(node_types) = node_types
        && !node_types.is_empty()
    {
        return Some(node_types);
    }

    let node_types = IfElseKindDetector::from_run_func(run_func);
    if let Some(node_types) = node_types
        && !node_types.is_empty()
    {
        return Some(node_types);
    }

    None
}

/// Result of attempting to collect node type variants.
#[derive(Debug, PartialEq, Eq)]
enum CollectionResult {
    /// All syntax recognized as supported, variants collected should be complete.
    Complete,
    /// Some syntax not recognized as supported, variants collected may be incomplete
    /// and should not be treated as valid. We should default to running on any node type.
    Incomplete,
}

/// Format Rust code with `rustfmt`.
///
/// Does not format on disk - interfaces with `rustfmt` via stdin/stdout.
///
/// # Panics
/// Panics if `rustfmt` is not installed or fails to run. Panics if any I/O operation fails.
pub fn rust_fmt(source_text: &str) -> String {
    let mut rustfmt = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to run rustfmt (is it installed?)");

    let stdin = rustfmt.stdin.as_mut().unwrap();
    stdin.write_all(source_text.as_bytes()).unwrap();
    stdin.flush().unwrap();

    let output = rustfmt.wait_with_output().unwrap();
    if output.status.success() {
        String::from_utf8(output.stdout).unwrap()
    } else {
        // Formatting failed. Return unformatted code, to aid debugging.
        source_text.to_string()
    }
}
