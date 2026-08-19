use syn::{Expr, File, Item, Stmt};

/// Return top-level statements which can affect rule execution.
///
/// Local enum and struct declarations are type declarations, so they must not prevent the node
/// type detectors from recognizing an otherwise top-level narrowing check.
pub fn executable_stmts(block: &syn::Block) -> Vec<&Stmt> {
    block
        .stmts
        .iter()
        .filter(|stmt| !matches!(stmt, Stmt::Item(Item::Enum(_) | Item::Struct(_))))
        .collect()
}

pub fn is_node_kind_call(expr: &Expr) -> bool {
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
pub fn astkind_variant_from_path(path: &syn::Path) -> Option<String> {
    // Expect `AstKind::Variant`
    if path.segments.len() != 2 {
        return None;
    }
    if path.segments[0].ident != "AstKind" {
        return None;
    }
    Some(path.segments[1].ident.to_string())
}

fn find_impl_block_for<'a>(
    file: &'a File,
    rule_struct_name: &str,
    trait_name: &str,
) -> Option<&'a syn::ItemImpl> {
    file.items.iter().find_map(|item| {
        let syn::Item::Impl(imp) = item else { return None };
        let syn::Type::Path(self_ty) = imp.self_ty.as_ref() else { return None };
        let (trait_path, _) = imp.trait_.as_ref()?;
        (self_ty.path.is_ident(rule_struct_name) && trait_path.is_ident(trait_name)).then_some(imp)
    })
}

pub fn find_rule_impl_block<'a>(
    file: &'a File,
    rule_struct_name: &str,
) -> Option<&'a syn::ItemImpl> {
    find_impl_block_for(file, rule_struct_name, "Rule")
}

pub fn implements_project_rule(file: &File, rule_struct_name: &str) -> bool {
    find_impl_block_for(file, rule_struct_name, "ProjectRule").is_some()
}

pub fn find_impl_function<'a>(
    imp: &'a syn::ItemImpl,
    func_name: &str,
) -> Option<&'a syn::ImplItemFn> {
    for impl_item in &imp.items {
        let syn::ImplItem::Fn(func) = impl_item else { continue };
        if func.sig.ident == func_name {
            return Some(func);
        }
    }
    None
}
