mod ast;
mod ast_kind;
mod impl_get_span;

/// Inserts a newline in the `TokenStream`.
#[allow(unused)]
macro_rules! endl {
    () => {
        /* only works in the context of `quote` macro family! */
    };
}

/// Similar to how `insert` macro works in the context of `quote` macro family, But this one can be
/// used outside and accepts expressions.
/// Wraps the result of the given expression in `insert!({value here});` and outputs it as `TokenStream`.
macro_rules! insert {
    ($txt:expr) => {{
        let txt: &str = &*$txt;
        format!(r#"insert!("{}");"#, txt).parse::<proc_macro2::TokenStream>().unwrap()
    }};
}

/// Creates a generated file warning + required information for a generated file.
macro_rules! generated_header {
    () => {{
        let file = file!().replace("\\", "/");
        let edit_comment = $crate::generators::insert!(format!(
            "// To edit this generated file you have to edit `{file}`"
        ));
        // TODO add generation date, AST source hash, etc here.
        quote::quote! {
            insert!("// Auto-generated code, DO NOT EDIT DIRECTLY!");
            #edit_comment
            endl!();
        }
    }};
}

pub(crate) use generated_header;
pub(crate) use insert;

pub use ast::AstGenerator;
pub use ast_kind::AstKindGenerator;
pub use impl_get_span::ImplGetSpanGenerator;
