#![allow(clippy::print_stdout)]

use std::{
    fmt::Write as _,
    fs,
    io::{self, Write as _},
    path::Path,
    process::{Command, Stdio},
};

use convert_case::{Case, Casing};
use rustc_hash::FxHashSet;
use syn::{Expr, ExprIf, File, Pat, Path as SynPath, Stmt}; // keep syn in scope for parse_file used elsewhere

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

        let has_detected = !detected_types.is_empty();
        let (node_types_init, any_node_type) = if has_detected {
            (detected_types.to_ast_type_bitset_string(), false)
        } else {
            ("AstTypesBitset::new()".to_string(), true)
        };

        write!(
            out,
            r"
impl RuleRunner for crate::rules::{plugin_module}::{rule_module}::{rule_struct} {{
    const NODE_TYPES: &AstTypesBitset = &{node_types_init};
    const ANY_NODE_TYPE: bool = {any_node_type};
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

/// Given a rule entry, attempt to find its corresponding source file path
fn find_rule_source_file(root: &Path, rule: &RuleEntry) -> Option<std::path::PathBuf> {
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

/// Represents a lint rule entry in the `declare_all_lint_rules!` macro.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct RuleEntry<'e> {
    /// The module name of the rule's plugin, like `eslint` in `eslint::no_debugger::NoDebugger`.
    plugin_module_name: &'e str,
    /// The rule's module name, like `no_debugger` in `eslint::no_debugger:NoDebugger`.
    rule_module_name: &'e str,
}

impl RuleEntry<'_> {
    /// Get the rule's struct name, like `NoDebugger` in `eslint::no_debugger::NoDebugger`.
    fn rule_struct_name(&self) -> String {
        self.rule_module_name.to_case(Case::Pascal)
    }
}

/// Parses `crates/oxc_linter/src/rules.rs` to extract all lint rule declarations into a list
/// of `RuleEntry`.
fn get_all_rules(contents: &str) -> io::Result<Vec<RuleEntry<'_>>> {
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

/// A set of AstKind variants, used for storing the unique node types detected in a rule,
/// or a portion of the rule file.
struct NodeTypeSet {
    node_types: FxHashSet<String>,
}

impl NodeTypeSet {
    /// Create a new set of node variants
    fn new() -> Self {
        Self { node_types: FxHashSet::default() }
    }

    /// Insert a variant into the set
    fn insert(&mut self, node_type_variant: String) {
        self.node_types.insert(node_type_variant);
    }

    /// Returns `true` if there are no node types in the set.
    fn is_empty(&self) -> bool {
        self.node_types.is_empty()
    }

    /// Extend the set with another set of node types.
    fn extend(&mut self, other: NodeTypeSet) {
        self.node_types.extend(other.node_types);
    }

    /// Returns the generated code string to initialize an `AstTypesBitset` with the variants
    /// in this set.
    fn to_ast_type_bitset_string(&self) -> String {
        let mut variants: Vec<&str> =
            self.node_types.iter().map(std::string::String::as_str).collect();
        variants.sort_unstable();
        let type_idents: Vec<String> =
            variants.into_iter().map(|v| format!("AstType::{v}")).collect();
        format!("AstTypesBitset::from_types(&[{}])", type_idents.join(", "))
    }
}

/// Detect the top-level node types used in a lint rule file by analyzing the Rust AST with `syn`.
/// Returns `Some(bitset)` if at least one node type can be determined, otherwise `None`.
fn detect_top_level_node_types(file: &File, rule: &RuleEntry) -> Option<NodeTypeSet> {
    let rule_impl = find_rule_impl_block(file, &rule.rule_struct_name())?;
    let run_func = find_impl_function(rule_impl, "run")?;

    let node_types = IfElseKindDetector::from_run_func(run_func)?;
    if node_types.is_empty() {
        return None;
    }

    Some(node_types)
}

fn find_rule_impl_block<'a>(file: &'a File, rule_struct_name: &str) -> Option<&'a syn::ItemImpl> {
    for item in &file.items {
        let syn::Item::Impl(imp) = item else { continue };
        let ident = match imp.self_ty.as_ref() {
            syn::Type::Path(p) => p.path.get_ident(),
            _ => None,
        };
        if ident.is_some_and(|id| id == rule_struct_name)
            && imp.trait_.as_ref().is_some_and(|(_, path, _)| path.is_ident("Rule"))
        {
            return Some(imp);
        }
    }
    None
}

fn find_impl_function<'a>(imp: &'a syn::ItemImpl, func_name: &str) -> Option<&'a syn::ImplItemFn> {
    for impl_item in &imp.items {
        let syn::ImplItem::Fn(func) = impl_item else { continue };
        if func.sig.ident == func_name {
            return Some(func);
        }
    }
    None
}

/// Detects top-level `if let AstKind::... = node.kind()` patterns in the `run` method.
struct IfElseKindDetector {
    node_types: NodeTypeSet,
}

impl IfElseKindDetector {
    fn from_run_func(run_func: &syn::ImplItemFn) -> Option<NodeTypeSet> {
        // Only consider when the body has exactly one top-level statement and it's an `if`.
        let block = &run_func.block;
        if block.stmts.len() != 1 {
            return None;
        }
        let stmt = &block.stmts[0];
        let Stmt::Expr(Expr::If(ifexpr), _) = stmt else { return None };
        let mut detector = Self { node_types: NodeTypeSet::new() };
        let result = detector.collect_if_chain_variants(ifexpr);
        if result == CollectionResult::Incomplete || detector.node_types.is_empty() {
            return None;
        }
        Some(detector.node_types)
    }

    /// Collects AstKind variants from an if-else chain of `if let AstKind::Xxx(..) = node.kind()`.
    /// Returns `true` if all syntax was recognized as supported, otherwise `false`, indicating that
    /// the variants collected may be incomplete and should not be treated as valid.
    fn collect_if_chain_variants(&mut self, ifexpr: &ExprIf) -> CollectionResult {
        // Extract variants from condition like `if let AstKind::Xxx(..) = node.kind()`.
        if self.extract_variants_from_if_let_condition(&ifexpr.cond) == CollectionResult::Incomplete
        {
            // If syntax is not recognized, return Incomplete.
            return CollectionResult::Incomplete;
        }
        // Walk else-if chain.
        if let Some((_, else_branch)) = &ifexpr.else_branch {
            match &**else_branch {
                Expr::If(nested) => self.collect_if_chain_variants(nested),
                // plain `else { ... }` should default to any node type
                _ => CollectionResult::Incomplete,
            }
        } else {
            CollectionResult::Complete
        }
    }

    /// Extracts AstKind variants from an `if let` condition like `if let AstKind::Xxx(..) = node.kind()`.
    fn extract_variants_from_if_let_condition(&mut self, cond: &Expr) -> CollectionResult {
        let Expr::Let(let_expr) = cond else { return CollectionResult::Incomplete };
        // RHS must be `node.kind()`
        if is_node_kind_call(&let_expr.expr) {
            self.extract_variants_from_pat(&let_expr.pat)
        } else {
            CollectionResult::Incomplete
        }
    }

    fn extract_variants_from_pat(&mut self, pat: &Pat) -> CollectionResult {
        match pat {
            Pat::Or(orpat) => {
                for p in &orpat.cases {
                    if self.extract_variants_from_pat(p) == CollectionResult::Incomplete {
                        return CollectionResult::Incomplete;
                    }
                }
                CollectionResult::Complete
            }
            Pat::TupleStruct(ts) => {
                if let Some(variant) = astkind_variant_from_path(&ts.path) {
                    self.node_types.insert(variant);
                    CollectionResult::Complete
                } else {
                    CollectionResult::Incomplete
                }
            }
            _ => CollectionResult::Incomplete,
        }
    }
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

fn is_node_kind_call(expr: &Expr) -> bool {
    if let Expr::MethodCall(mc) = expr
        && mc.method == "kind"
        && mc.args.is_empty()
        && let Expr::Path(p) = &*mc.receiver
    {
        return p.path.is_ident("node");
    }
    false
}

/// Extract AstKind variant from something like `AstKind::Variant`
fn astkind_variant_from_path(path: &SynPath) -> Option<String> {
    // Expect `AstKind::Variant`
    if path.segments.len() != 2 {
        return None;
    }
    if path.segments[0].ident != "AstKind" {
        return None;
    }
    Some(path.segments[1].ident.to_string())
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
