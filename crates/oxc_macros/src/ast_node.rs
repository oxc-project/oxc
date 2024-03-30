use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

const AST_NODE_ID_IDENT: &str = "ast_node_id";

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    validate_derive_input(&input);

    derive_for_input(input)
}

fn validate_derive_input(input: &DeriveInput) {
    match &input.data {
        Data::Struct(data) => {
            assert!(data
                .fields
                .iter()
                .any(|field| field.ident.as_ref().is_some_and(|f| f == AST_NODE_ID_IDENT)),
                "AstNode derive macro needs the implementer structure to contain an `ast_node_id` field."
            );
        }
        Data::Enum(_) => {
            // TODO: maybe we need to check for all enum variants, They all have to also implement
            // the AstNode trait. Helps with better error messages when derive fails.
        }
        Data::Union(_) => {
            panic!("Ast derive macro doesn't support union types.");
        }
    }
}

fn derive_for_input(input: DeriveInput) -> TokenStream {
    match &input.data {
        Data::Struct(_) => derive_for_struct(input),
        Data::Enum(_) => derive_for_enum(input),
        Data::Union(_) => unreachable!("Union types aren't supported."),
    }
}

fn derive_for_struct(input: DeriveInput) -> TokenStream {
    debug_assert!(matches!(input.data, Data::Struct(_)));

    let ident = input.ident;

    let generics = input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics crate::AstNode for #ident #ty_generics #where_clause {
            fn ast_node_id(&self) -> Option<AstNodeId> {
                self.ast_node_id.get()
            }

            fn set_ast_node_id(&self, id: Option<AstNodeId>) {
                self.ast_node_id.set(id);
            }

            fn swap_ast_node_id(&self, id: Option<AstNodeId>) -> Option<AstNodeId> {
                self.ast_node_id.swap(id)
            }
        }
    }
    .into()
}

fn derive_for_enum(input: DeriveInput) -> TokenStream {
    debug_assert!(matches!(input.data, Data::Enum(_)));

    let Data::Enum(data) = input.data else {
        unreachable!("We check for it in debug builds, It shouldn't happen in production!");
    };

    let node = quote!(node);
    let id = quote!(id);

    let ident = input.ident;

    let generics = input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let variant_matcher = |operation: TokenStream2| {
        data.variants.iter().fold(TokenStream2::new(), |mut acc, var| {
            let span = var.span();
            let var_ident = var.ident.clone();

            assert!(
                var.fields.len() == 1,
                "AstNode derive macro only supports enum variants with a single field."
            );

            let fields = match var.fields {
                Fields::Unnamed(_) => quote_spanned! ( span=> (#node) ),
                Fields::Named(_) => {
                    panic!("AstNode derive macro does not support Named enum fields.")
                }
                Fields::Unit => panic!("AstNode derive macro does not support Unit enum fields."),
            };

            acc.extend(quote_spanned! {
                span=> #ident::#var_ident #fields => #operation,
            });
            acc
        })
    };

    let get_match = variant_matcher(quote! {#node.ast_node_id()});
    let set_match = variant_matcher(quote! {#node.set_ast_node_id(#id)});
    let swap_match = variant_matcher(quote! {#node.swap_ast_node_id(#id)});

    quote! {
        impl #impl_generics crate::AstNode for #ident #ty_generics #where_clause {
            fn ast_node_id(&self) -> Option<AstNodeId> {
                match self {
                    #get_match
                }
            }

            fn set_ast_node_id(&self, #id: Option<AstNodeId>) {
                match self {
                    #set_match
                };
            }

            fn swap_ast_node_id(&self, #id: Option<AstNodeId>) -> Option<AstNodeId> {
                match self {
                    #swap_match
                }
            }
        }
    }
    .into()
}
