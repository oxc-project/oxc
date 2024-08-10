use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// returns `#[repr(C, u8)]` if `enum_` has any non-unit variant,
/// Otherwise it would return `#[repr(u8)]`.
fn enum_repr(enum_: &syn::ItemEnum) -> TokenStream2 {
    if enum_.variants.iter().any(|var| !matches!(var.fields, syn::Fields::Unit)) {
        quote!(#[repr(C, u8)])
    } else {
        quote!(#[repr(u8)])
    }
}

/// Generate assertions that traits used in `#[generate_derive]` are in scope.
///
/// e.g. for `#[generate_derive(GetSpan)]`, it generates:
///
/// ```rs
/// const _: () = {
///     {
///         trait AssertionTrait: ::oxc_span::GetSpan {}
///         impl<T: GetSpan> AssertionTrait for T {}
///     }
/// };
/// ```
///
/// If `GetSpan` is not in scope, or it is not the correct `oxc_span::GetSpan`,
/// this will raise a compilation error.
fn assert_generated_derives(attrs: &[syn::Attribute]) -> TokenStream2 {
    #[inline]
    fn parse(attr: &syn::Attribute) -> impl Iterator<Item = syn::Ident> {
        attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Ident, syn::token::Comma>::parse_terminated,
        )
        .expect("`generate_derive` only accepts traits as single segment paths, Found an invalid argument")
        .into_iter()
    }

    // TODO: benchmark this to see if a lazy static cell would perform better.
    #[inline]
    fn abs_trait(
        ident: &syn::Ident,
    ) -> (/* absolute type path */ TokenStream2, /* possible generics */ TokenStream2) {
        if ident == "CloneIn" {
            (quote!(::oxc_allocator::CloneIn), quote!(<'static>))
        } else if ident == "GetSpan" {
            (quote!(::oxc_span::GetSpan), TokenStream2::default())
        } else if ident == "GetSpanMut" {
            (quote!(::oxc_span::GetSpanMut), TokenStream2::default())
        } else {
            panic!("Invalid derive trait(generate_derive): {ident}");
        }
    }

    // NOTE: At this level we don't care if a trait is derived multiple times, It is the
    // responsibility of the codegen to raise errors for those.
    let assertion =
        attrs.iter().filter(|attr| attr.path().is_ident("generate_derive")).flat_map(parse).map(
            |derive| {
                let (abs_derive, generics) = abs_trait(&derive);
                quote! {{
                    // NOTE: these are wrapped in a scope to avoid the need for unique identifiers.
                    trait AssertionTrait: #abs_derive #generics {}
                    impl<T: #derive #generics> AssertionTrait for T {}
                }}
            },
        );
    quote!(const _: () = { #(#assertion)* };)
}

/// This attribute serves two purposes.
/// First, it is a marker for our codegen to detect AST types.
/// Secondly, it generates the following code:
///
/// * Prepend `#[repr(C)]` to structs
/// * Prepend `#[repr(C, u8)]` to fieldful enums e.g. `enum E { X: u32, Y: u8 }`
/// * Prepend `#[repr(u8)]` to unit (fieldless) enums e.g. `enum E { X, Y, Z, }`
/// * Prepend `#[derive(oxc_ast_macros::Ast)]` to all structs and enums
/// * Add assertions that traits used in `#[generate_derive(...)]` are in scope.
#[proc_macro_attribute]
#[allow(clippy::missing_panics_doc)]
pub fn ast(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::Item);

    let (head, tail) = match &input {
        syn::Item::Enum(enum_) => (enum_repr(enum_), assert_generated_derives(&enum_.attrs)),
        syn::Item::Struct(struct_) => {
            (quote!(#[repr(C)]), assert_generated_derives(&struct_.attrs))
        }

        _ => unreachable!(),
    };

    let expanded = quote! {
        #[derive(::oxc_ast_macros::Ast)]
        #head
        #input
        #tail
    };
    TokenStream::from(expanded)
}

/// Dummy derive macro for a non-existent trait `Ast`.
///
/// Does not generate any code.
/// Only purpose is to allow using `#[scope]`, `#[visit]`, and other attrs in the AST node type defs.
/// These "marker" attributes are used in codegen.
#[proc_macro_derive(Ast, attributes(scope, visit, span, serde, tsify, generate_derive))]
pub fn ast_derive(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

/// Derive macro generating an impl of the trait `CloneIn`.
///
/// NOTE: This is an internal macro!
/// # Panics
///
#[proc_macro_derive(CloneIn)]
pub fn derive_clone_in(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::Item);
    match &item {
        syn::Item::Struct(syn::ItemStruct { ident, generics, .. })
        | syn::Item::Enum(syn::ItemEnum { ident, generics, .. })
            if generics.params.is_empty() =>
        {
            quote! {
                #[automatically_derived]
                impl<'alloc> ::oxc_allocator::CloneIn<'alloc> for #ident {
                    type Cloned = #ident;

                    fn clone_in(&self, _: &'alloc ::oxc_allocator::Allocator) -> Self::Cloned {
                        std::clone::Clone::clone(self)
                    }
                }
            }
            .into()
        }
        _ => panic!("At the moment `CloneIn` derive macro only works for types without lifetimes and/or generic params"),
    }
}
