use proc_macro::TokenStream;
use quote::quote;

pub fn impl_typename_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let name_without_vertex = name.to_string().replace("Vertex", "");
    let name_without_vertex_with_ast = format!("{name_without_vertex}AST");
    // println!("name={name}, name_with_ast={name_with_ast}");
    let gen = quote! {
    impl<'a> Typename for #name<'a> {
        fn typename(&self) -> &'static str {
            if self.ast_node.is_some() {
                #name_without_vertex_with_ast
            } else {
                #name_without_vertex
            }
        }
    }
    };
    gen.into()
}
