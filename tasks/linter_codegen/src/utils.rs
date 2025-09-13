use syn::{Expr, File};

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

pub fn find_rule_impl_block<'a>(
    file: &'a File,
    rule_struct_name: &str,
) -> Option<&'a syn::ItemImpl> {
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
