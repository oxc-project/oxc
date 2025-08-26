#![allow(clippy::print_stdout)]
use std::fmt::Write as _;
use std::io::Write as _;
use std::process::{Command, Stdio};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

use convert_case::{Case, Casing};
use oxc_ast::AstType;
use oxc_semantic::AstTypesBitset;
use syn::{Expr, ExprIf, ExprMatch, File, Pat, Path as SynPath, Stmt}; // keep syn in scope for parse_file used elsewhere

fn main() -> io::Result<()> {
    generate_rule_runner_impls()
}

/// # Errors
/// Returns `io::Error` if file operations fail.
pub fn generate_rule_runner_impls() -> io::Result<()> {
    let root = project_root::get_project_root()
        .map_err(|e| std::io::Error::other(format!("could not find project root: {e}")))?;
    let rules_rs = root.join("crates/oxc_linter/src/rules.rs");
    let contents = fs::read_to_string(&rules_rs)?;

    let start_marker = "oxc_macros::declare_all_lint_rules!";
    let start = contents.find(start_marker).ok_or_else(|| {
        std::io::Error::other("could not find declare_all_lint_rules macro invocation")
    })?;
    let brace_start = contents[start..]
        .find('{')
        .map(|i| start + i + 1)
        .ok_or_else(|| std::io::Error::other("could not find opening brace of macro body"))?;

    let mut depth = 1i32;
    let mut idx = brace_start;
    let bytes = contents.as_bytes();
    while idx < bytes.len() && depth > 0 {
        match bytes[idx] as char {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        idx += 1;
    }
    let body_end = idx - 1;
    let body = &contents[brace_start..body_end];

    // Collect (module path, struct name) pairs. Do NOT deduplicate by struct name because
    // different plugins may have rules with the same struct name.
    let mut rule_entries: Vec<(String, String)> = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if !line.ends_with(',') {
            continue;
        }
        let path = &line[..line.len() - 1];
        if path.contains('(') {
            continue;
        } // skip entries with config like foo(bar)
        if let Some(last) = path.split("::").last() {
            if last.is_empty() {
                continue;
            }
            let struct_name = last.to_case(Case::Pascal);
            rule_entries.push((path.to_string(), struct_name));
        }
    }
    // Sort deterministically by full path
    rule_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut out = String::new();
    out.push_str("// Auto-generated code, DO NOT EDIT DIRECTLY!\n");
    out.push_str("// To regenerate: `cargo run -p oxc_linter_codegen`\n\n");
    out.push_str("#![allow(clippy::needless_pass_by_value)]\n\n");
    out.push_str("use oxc_semantic::AstTypesBitset;\n");
    out.push_str("use crate::rule::RuleRunner;\n\n");
    let mut needs_ast_type_import = false;
    let mut impls_buf = String::new();

    // Build a set of valid AstType variant names to validate any textual fallbacks.
    let _allowed_asttype_variants = get_asttype_variant_names().unwrap_or_default();

    for (path, struct_name) in &rule_entries {
        let full_type_path = format!("crate::rules::{path}::{struct_name}");
        // Try to open the rule source file and use syn to detect node types
        let mut detected_types: BTreeSet<String> = BTreeSet::new();
        if let Some(src_path) = find_rule_source_file(&root, path)
            && let Ok(src_contents) = fs::read_to_string(&src_path)
            && let Ok(file) = syn::parse_file(&src_contents)
        {
            if let Some(bitset) = detect_top_level_node_types(&file) {
                for ty in &bitset {
                    detected_types.insert(format!("{ty:?}"));
                }
            } else {
                // No reliable detection; default to ANY (no textual fallback here)
            }
        }

        let has_detected = !detected_types.is_empty();
        let (node_types_init, any_node_type) = if has_detected {
            // Map variant name to AstType constant path (AstType::Variant)
            let type_idents: Vec<String> =
                detected_types.into_iter().map(|v| format!("AstType::{v}")).collect();
            (format!("AstTypesBitset::from_types(&[{}])", type_idents.join(", ")), false)
        } else {
            ("AstTypesBitset::new()".to_string(), true)
        };

        if has_detected {
            needs_ast_type_import = true;
        }
        write!(
            impls_buf,
            "impl RuleRunner for {full_type_path} {{\n    const NODE_TYPES: &AstTypesBitset = &{node_types_init};\n    const ANY_NODE_TYPE: bool = {any_node_type};\n\n    #[inline] fn types_info(&self) -> (&'static AstTypesBitset, bool) {{ (Self::NODE_TYPES, Self::ANY_NODE_TYPE) }}\n}}\n\n"
        ).unwrap();
    }

    if needs_ast_type_import {
        out.push_str("use oxc_ast::AstType;\n\n");
    }
    out.push_str(&impls_buf);

    let target_path = root.join("crates/oxc_linter/src/rule_runner_impls.rs");
    let formatted_out = rust_fmt(&out);
    fs::write(&target_path, formatted_out)?;
    println!("Generated {} impls into {}", rule_entries.len(), target_path.display());
    Ok(())
}

/// Given a module path like `eslint::no_debugger`, attempt to find the corresponding source file path
fn find_rule_source_file(root: &Path, module_path: &str) -> Option<std::path::PathBuf> {
    // A rule path corresponds to `crates/oxc_linter/src/rules/<module_path>.rs`
    // or `.../<module_path>/mod.rs`. Module path segments separated by `::`.
    let mut p = root.join("crates/oxc_linter/src/rules");
    for (i, seg) in module_path.split("::").enumerate() {
        let is_last = i == module_path.split("::").count() - 1;
        if is_last {
            let file_rs = p.join(format!("{seg}.rs"));
            if file_rs.exists() {
                return Some(file_rs);
            }
            let mod_rs = p.join(seg).join("mod.rs");
            if mod_rs.exists() {
                return Some(mod_rs);
            }
            return None;
        }
        p = p.join(seg);
    }
    None
}

/// Detect the top-level node types used in a lint rule file by analyzing the Rust AST with `syn`.
/// Returns `Some(bitset)` if at least one node type can be determined, otherwise `None`.
pub fn detect_top_level_node_types(file: &File) -> Option<AstTypesBitset> {
    // Strategy: prefer a single top-level `if let AstKind::... = node.kind()` chain.
    // If not present, look for a top-level `let AstKind::... = node.kind() else { ... };` in run.
    // If still not present, try a single top-level `match node.kind()`.
    let variants: BTreeSet<String> = if let Some(det) = IfElseKindDetector::from_file(file) {
        det.variants
    } else if let Some(det) = LetElseDetector::from_file(file) {
        det.variants
    } else if let Some(det) = MatchKindDetector::from_file(file) {
        det.variants
    } else {
        return None;
    };
    if variants.is_empty() {
        return None;
    }

    // Resolve variant names to AstType values by reading the generated AstType enum to get discriminants.
    let Ok(root) = project_root::get_project_root() else { return None };
    let ast_kind_path = root.join("crates/oxc_ast/src/generated/ast_kind.rs");
    let Ok(ast_kind_contents) = fs::read_to_string(&ast_kind_path) else { return None };
    let Ok(ast_file) = syn::parse_file(&ast_kind_contents) else { return None };
    let mapping = build_asttype_name_to_index_map(&ast_file);
    if mapping.is_empty() {
        return None;
    }

    let mut types: Vec<AstType> = Vec::new();
    for name in variants {
        if let Some(&idx) = mapping.get(name.as_str()) {
            // SAFETY: `AstType` is repr(u8) and indices in mapping are parsed from the enum.
            let ty: AstType = unsafe { std::mem::transmute::<u8, AstType>(idx) };
            types.push(ty);
        }
    }
    if types.is_empty() { None } else { Some(AstTypesBitset::from_types(&types)) }
}

/// Detects top-level `if let AstKind::... = node.kind()` pattern(s) in the `run` method.
struct IfElseKindDetector {
    variants: BTreeSet<String>,
}

impl IfElseKindDetector {
    fn from_file(file: &File) -> Option<Self> {
        // Find `impl <Trait>? for <Type>` blocks and a method named `run`.
        for item in &file.items {
            let syn::Item::Impl(imp) = item else { continue };
            for impl_item in &imp.items {
                let syn::ImplItem::Fn(func) = impl_item else { continue };
                if func.sig.ident != "run" {
                    continue;
                }
                // Only consider when the body has exactly one top-level statement and it's an `if`.
                let block = &func.block;
                if block.stmts.len() != 1 {
                    return None;
                }
                let stmt = &block.stmts[0];
                let Stmt::Expr(Expr::If(ifexpr), _) = stmt else { return None };
                let mut variants = BTreeSet::new();
                if !collect_if_chain_variants(ifexpr, &mut variants) {
                    return None;
                }
                return Some(Self { variants });
            }
        }
        None
    }
}

fn collect_if_chain_variants(ifexpr: &ExprIf, out: &mut BTreeSet<String>) -> bool {
    // Expect condition to be `if let AstKind::Xxx(..) = node.kind()`.
    if !extract_variants_from_if_condition(&ifexpr.cond, out) {
        return false;
    }
    // Walk else-if chain.
    if let Some((_, else_branch)) = &ifexpr.else_branch {
        match &**else_branch {
            Expr::If(nested) => collect_if_chain_variants(nested, out),
            _ => true, // plain else { .. } is fine
        }
    } else {
        true
    }
}

/// Detects top-level `let AstKind::... = node.kind() else { ... };` statements in `run`.
struct LetElseDetector {
    variants: BTreeSet<String>,
}

impl LetElseDetector {
    fn from_file(file: &File) -> Option<Self> {
        for item in &file.items {
            let syn::Item::Impl(imp) = item else { continue };
            for impl_item in &imp.items {
                let syn::ImplItem::Fn(func) = impl_item else { continue };
                if func.sig.ident != "run" {
                    continue;
                }
                let mut variants = BTreeSet::new();
                for (idx, stmt) in func.block.stmts.iter().enumerate() {
                    if let Stmt::Local(local) = stmt {
                        // Must have initializer and an else branch (let-else)
                        if let Some(init) = &local.init {
                            // Check RHS is `node.kind()`
                            if !is_node_kind_call(&init.expr) {
                                continue;
                            }
                            // Ensure there is an else branch and it contains only a single `return` statement
                            let has_else_return = init
                                .diverge
                                .as_ref()
                                .is_some_and(|d| else_expr_is_only_return(&d.1));
                            if !has_else_return {
                                continue;
                            }
                            // Ensure there are no top-level `if` statements before this let-else
                            if func
                                .block
                                .stmts
                                .iter()
                                .take(idx)
                                .any(|s| matches!(s, Stmt::Expr(Expr::If(_), _)))
                            {
                                return None;
                            }
                            // Ensure that else
                            let _ = extract_variants_from_pat(&local.pat, &mut variants);
                        }
                    }
                }
                if !variants.is_empty() {
                    return Some(Self { variants });
                }
            }
        }
        None
    }
}

/// Detects top-level `match node.kind()` expressions in the `run` method.
struct MatchKindDetector {
    variants: BTreeSet<String>,
}

impl MatchKindDetector {
    fn from_file(file: &File) -> Option<Self> {
        for item in &file.items {
            let syn::Item::Impl(imp) = item else { continue };
            for impl_item in &imp.items {
                let syn::ImplItem::Fn(func) = impl_item else { continue };
                if func.sig.ident != "run" {
                    continue;
                }
                let block = &func.block;
                if block.stmts.len() != 1 {
                    return None;
                }
                let stmt = &block.stmts[0];
                let matchexpr: &ExprMatch = match stmt {
                    Stmt::Expr(Expr::Match(m), _) => m,
                    _ => return None,
                };
                // Ensure the scrutinee is node.kind()
                if !is_node_kind_call(&matchexpr.expr) {
                    return None;
                }
                // If any arm has a guard `if ...`, or the wildcard arm contains code, bail to ANY
                for arm in &matchexpr.arms {
                    if arm.guard.is_some() {
                        return None;
                    }
                    if arm_pat_has_wildcard(&arm.pat) && arm_body_has_code(&arm.body) {
                        return None;
                    }
                }
                let mut variants = BTreeSet::new();
                for arm in &matchexpr.arms {
                    if arm_pat_has_wildcard(&arm.pat) {
                        continue;
                    }
                    let _ = extract_variants_from_pat(&arm.pat, &mut variants);
                }
                return Some(Self { variants });
            }
        }
        None
    }
}

fn arm_pat_has_wildcard(pat: &Pat) -> bool {
    match pat {
        Pat::Wild(_) => true,
        Pat::Or(orpat) => orpat.cases.iter().any(arm_pat_has_wildcard),
        _ => false,
    }
}

fn arm_body_has_code(expr: &Expr) -> bool {
    match expr {
        Expr::Block(b) => !b.block.stmts.is_empty(),
        _ => true,
    }
}

fn else_expr_is_only_return(expr: &Expr) -> bool {
    if let Expr::Block(b) = expr {
        let stmts = &b.block.stmts;
        if stmts.len() != 1 {
            return false;
        }
        matches!(stmts[0], Stmt::Expr(Expr::Return(_), _))
    } else {
        false
    }
}

fn extract_variants_from_if_condition(cond: &Expr, out: &mut BTreeSet<String>) -> bool {
    let Expr::Let(let_expr) = cond else { return false };
    // RHS must be `node.kind()`
    if !is_node_kind_call(&let_expr.expr) {
        return false;
    }
    // LHS pattern must be `AstKind::Variant(..)` possibly with `|` patterns.
    extract_variants_from_pat(&let_expr.pat, out)
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

fn extract_variants_from_pat(pat: &Pat, out: &mut BTreeSet<String>) -> bool {
    match pat {
        Pat::Or(orpat) => {
            let mut ok = false;
            for p in &orpat.cases {
                ok |= extract_variants_from_pat(p, out);
            }
            ok
        }
        Pat::TupleStruct(ts) => {
            if let Some(variant) = astkind_variant_from_path(&ts.path) {
                out.insert(variant);
                true
            } else {
                false
            }
        }
        Pat::Path(ppath) => {
            if let Some(variant) = astkind_variant_from_path(&ppath.path) {
                out.insert(variant);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn astkind_variant_from_path(path: &SynPath) -> Option<String> {
    // Expect `AstKind::Variant`
    let mut segments = path.segments.iter();
    let first = segments.next()?;
    if first.ident != "AstKind" {
        return None;
    }
    let second = segments.next()?;
    // Ensure no further segments like `AstKind::Variant::Something`
    if segments.next().is_some() {
        return None;
    }
    Some(second.ident.to_string())
}

// Load the set of valid AstType variant names from the generated enum source.
fn get_asttype_variant_names() -> Option<BTreeSet<String>> {
    let root = project_root::get_project_root().ok()?;
    let ast_kind_path = root.join("crates/oxc_ast/src/generated/ast_kind.rs");
    let contents = fs::read_to_string(&ast_kind_path).ok()?;
    let file = syn::parse_file(&contents).ok()?;
    let mapping = build_asttype_name_to_index_map(&file);
    if mapping.is_empty() {
        return None;
    }
    let mut set = BTreeSet::new();
    for k in mapping.keys() {
        set.insert(k.clone());
    }
    Some(set)
}

fn build_asttype_name_to_index_map(file: &File) -> BTreeMap<String, u8> {
    let mut map = BTreeMap::new();
    for item in &file.items {
        if let syn::Item::Enum(en) = item
            && en.ident == "AstType"
        {
            for variant in &en.variants {
                let name = variant.ident.to_string();
                if let Some((_, expr)) = &variant.discriminant
                    && let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit_int), .. }) = expr
                    && let Ok(value) = lit_int.base10_parse::<u8>()
                {
                    map.insert(name, value);
                }
            }
        }
    }
    map
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
