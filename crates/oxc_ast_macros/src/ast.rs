use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Attribute, Fields, Ident, Item, ItemEnum};

pub fn ast(input: &Item) -> TokenStream {
    let (head, tail) = match input {
        Item::Enum(enum_) => (enum_repr(enum_), assert_generated_derives(&enum_.attrs)),
        Item::Struct(struct_) => (quote!(#[repr(C)]), assert_generated_derives(&struct_.attrs)),
        _ => unreachable!(),
    };

    quote! {
        #[derive(::oxc_ast_macros::Ast)]
        #head
        #input
        #tail
    }
}

/// If `enum_` has any non-unit variant, returns `#[repr(C, u8)]`, otherwise returns `#[repr(u8)]`.
fn enum_repr(enum_: &ItemEnum) -> TokenStream {
    if enum_.variants.iter().any(|var| !matches!(var.fields, Fields::Unit)) {
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
fn assert_generated_derives(attrs: &[Attribute]) -> TokenStream {
    // NOTE: At this level we don't care if a trait is derived multiple times, It is the
    // responsibility of the `ast_tools` to raise errors for those.
    let assertion = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("generate_derive"))
        .flat_map(parse_attr)
        .map(|derive| {
            let (abs_derive, generics) = abs_trait(&derive);
            quote! {{
                // NOTE: these are wrapped in a scope to avoid the need for unique identifiers.
                trait AssertionTrait: #abs_derive #generics {}
                impl<T: #derive #generics> AssertionTrait for T {}
            }}
        });
    quote!(const _: () = { #(#assertion)* };)
}

#[inline]
fn parse_attr(attr: &Attribute) -> impl Iterator<Item = Ident> {
    attr.parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)
        .expect("`#[generate_derive]` only accepts traits as single segment paths. Found an invalid argument.")
        .into_iter()
}

// TODO: benchmark this to see if a lazy static cell containing `HashMap` would perform better.
#[inline]
fn abs_trait(
    ident: &Ident,
) -> (/* absolute type path */ TokenStream, /* possible generics */ TokenStream) {
    if ident == "CloneIn" {
        (quote!(::oxc_allocator::CloneIn), quote!(<'static>))
    } else if ident == "GetSpan" {
        (quote!(::oxc_span::GetSpan), TokenStream::default())
    } else if ident == "GetSpanMut" {
        (quote!(::oxc_span::GetSpanMut), TokenStream::default())
    } else if ident == "ContentEq" {
        (quote!(::oxc_span::cmp::ContentEq), TokenStream::default())
    } else if ident == "ContentHash" {
        (quote!(::oxc_span::hash::ContentHash), TokenStream::default())
    } else if ident == "ESTree" {
        (quote!(::oxc_estree::ESTree), TokenStream::default())
    } else {
        invalid_derive(ident)
    }
}

#[cold]
fn invalid_derive(ident: &Ident) -> ! {
    panic!(
        "Invalid derive trait(generate_derive): {ident}.\n\
        Help: If you are trying to implement a new `generate_derive` trait, \
        make sure to add it to the list in `abs_trait` function."
    )
}
