use syn::{Expr, Pat, Stmt};

use crate::{node_type_set::NodeTypeSet, utils::astkind_variant_from_path};

/// Fetches the current list of variants that are handled by `run_on_regex_node`.
/// We read the source file to avoid hardcoding the list here and ensure this will stay updated.
pub fn get_regex_node_kinds() -> Option<NodeTypeSet> {
    // Read crates/oxc_linter/src/utils/regex.rs and extract all variants in `run_on_regex_node` function
    let regex_utils_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .parent()?
        .join("crates")
        .join("oxc_linter")
        .join("src")
        .join("utils")
        .join("regex.rs");
    let content = std::fs::read_to_string(regex_utils_path).ok()?;
    let syntax = syn::parse_file(&content).ok()?;
    let mut node_type_set = NodeTypeSet::new();
    for item in syntax.items {
        if let syn::Item::Fn(func) = item
            && func.sig.ident == "run_on_regex_node"
        {
            // Look for `match node.kind() { ... }` inside the function body
            for stmt in &func.block.stmts {
                if let Stmt::Expr(Expr::Match(match_expr), _) = stmt {
                    for arm in &match_expr.arms {
                        if let Pat::TupleStruct(ts) = &arm.pat
                            && let Some(variant) = astkind_variant_from_path(&ts.path)
                        {
                            node_type_set.insert(variant);
                        }
                    }
                    if !node_type_set.is_empty() {
                        return Some(node_type_set);
                    }
                }
            }
        }
    }
    None
}
