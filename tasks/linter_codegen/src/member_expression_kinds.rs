use syn::{Expr, Pat, Stmt};

use crate::{node_type_set::NodeTypeSet, utils::find_impl_function};

/// Fetches the current list of variants that can be returned by `AstKind::as_member_expression_kind()`.
/// We read the source file to avoid hardcoding the list here and ensure this will stay updated.
pub fn get_member_expression_kinds() -> Option<NodeTypeSet> {
    // Read crates/oxc_ast/src/ast_kind_impl.rs and extract all variants in `as_member_expression_kind` function
    let ast_kind_impl_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .parent()?
        .join("crates")
        .join("oxc_ast")
        .join("src")
        .join("ast_kind_impl.rs");
    let content = std::fs::read_to_string(ast_kind_impl_path).ok()?;
    let syntax = syn::parse_file(&content).ok()?;
    let mut node_type_set = NodeTypeSet::new();
    for item in syntax.items {
        if let syn::Item::Impl(impl_block) = item
            && let syn::Type::Path(type_path) = impl_block.self_ty.as_ref()
            && type_path.path.segments.last()?.ident == "AstKind"
        {
            let impl_fn = find_impl_function(&impl_block, "as_member_expression_kind")
                .expect("as_member_expression_kind function not found");

            // Look for `match self { ... }` inside the function body
            if impl_fn.block.stmts.len() != 1 {
                return None;
            }
            let stmt = &impl_fn.block.stmts[0];
            if let Stmt::Expr(Expr::Match(match_expr), _) = stmt {
                for arm in &match_expr.arms {
                    if let Pat::TupleStruct(ts) = &arm.pat
                        && let Some(variant) = self_astkind_variant_from_path(&ts.path)
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
    None
}

fn self_astkind_variant_from_path(path: &syn::Path) -> Option<String> {
    // Expect `Self::Variant`
    if path.segments.len() != 2 {
        return None;
    }
    if path.segments[0].ident != "Self" {
        return None;
    }
    Some(path.segments[1].ident.to_string())
}
