use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_quote;

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

    // TODO: benchmark this to see if a lazy static cell containing `HashMap` would perform better.
    #[inline]
    fn abs_trait(
        ident: &syn::Ident,
    ) -> (/* absolute type path */ TokenStream2, /* possible generics */ TokenStream2) {
        #[cold]
        fn invalid_derive(ident: &syn::Ident) -> ! {
            panic!(
                "Invalid derive trait(generate_derive): {ident}.\n\
                    Help: If you are trying to implement a new `generate_derive` trait, \
                    Make sure to add it to the list below."
            )
        }

        if ident == "CloneIn" {
            (quote!(::oxc_allocator::CloneIn), quote!(<'static>))
        } else if ident == "GetSpan" {
            (quote!(::oxc_span::GetSpan), TokenStream2::default())
        } else if ident == "GetSpanMut" {
            (quote!(::oxc_span::GetSpanMut), TokenStream2::default())
        } else if ident == "ContentEq" {
            (quote!(::oxc_span::cmp::ContentEq), TokenStream2::default())
        } else if ident == "ContentHash" {
            (quote!(::oxc_span::hash::ContentHash), TokenStream2::default())
        } else {
            invalid_derive(ident)
        }
    }

    // NOTE: At this level we don't care if a trait is derived multiple times, It is the
    // responsibility of the `ast_tools` to raise errors for those.
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
/// First, it is a marker for our `ast_tools` to detect AST types.
/// Secondly, it generates the following code:
///
/// * Prepend `#[repr(C)]` to structs
/// * Prepend `#[repr(C, u8)]` to fieldful enums e.g. `enum E { X: u32, Y: u8 }`
/// * Prepend `#[repr(u8)]` to unit (fieldless) enums e.g. `enum E { X, Y, Z, }`
/// * Prepend `#[derive(oxc_ast_macros::Ast)]` to all structs and enums
/// * Add assertions that traits used in `#[generate_derive(...)]` are in scope.
///
/// It also allows the usage of these helper attributes via deriving a "no-op" derive macro.
///
/// # Generator Attributes:
///
/// ## `#[scope(...)]`:
///
/// This attribute can be used in 2 places:
/// ### On `struct`/`enum` items:
/// When this attribute comes before an AST type definition it accepts 3 optional arguments.
/// 1. `flags(expr)`: It accepts an expression that would evaluate to `ScopeFlags`. It is used to annotate scope flags of the AST type.
/// 2. `if(expr)`: It accepts an expression that would evaluate to `bool` used for conditional scope creation.
/// 3. `strict_if(expr)`: It accepts an expression that would evaluate to `bool`, If this value is `true` the created scope would be `strict`.
///
/// NOTE: All these expressions can use `self` to access the current node they are getting executed on via an immutable reference.
///
/// ### On `struct` fields:
/// At this position this attribute can only have one shape: `#[scope(enter_before)]`.
/// It marks where `Visit::enter_scope` events should be fired for this AST type.
///
/// ## `#[visit(...)]`:
///
/// This attribute can only occur on `struct` fields, Or `enum` attributes.
/// It accepts 4 optional arguments.
///     1. `as(ident)`: It accepts an identifier, our generators would treat the type of this field/variant as if they were called as the given identifier.
///     2. `args(arg = expr)`: It accepts an argument name and an expression. Currently it only
///        accepts one argument.
///        a. `args(flags = expr)`: `expr` is an expression that would evaluate to `ScopeFlags`, This argument can only be used at places where the AST type is `Function`.
///     3. `enter_before`: It marks where this AST type should fire `Visit::enter_node` events.
///     4. `ignore`: It would ignore this field/variant in visits.
///
/// ## `#[span]`:
///
/// This attribute can be used to hint to the `ast_tools` which field should be used to obtain the span of this AST type.
///
/// ## `#[generate_derive(...)]`
///
/// This attribute has the same spirit as the `#[derive(...)]` macro, It is used to derive traits for the types.
/// However, Instead of expanding the derive at compile-time, We do this process on PR submits via `ast_tools` code generation.
/// These derived implementations would be output in the `crates/oxc_ast/src/generated` directory.
///
/// # Derive Helper Attributes:
///
/// These are helper attributes that are only meaningful when their respective trait is derived via `generate_derive`.
///
/// ## `#[clone_in(default)]`
///
/// This attribute is only used by `CloneIn` derive.
/// `struct` fields marked with this attribute at cloning will use the `Default::default()` value instead of `CloneIn::clone_in` to initialize.
///
/// # Mocked attributes:
///
/// These are just here to remove the need for boilerplate `#[cfg_attr(...)]`. If their actual trait is derived they would consume these, Otherwise, Our mock attributes will prevent compile errors.
///
/// 1. `serde`
/// 2. `tsify`
#[proc_macro_attribute]
#[allow(clippy::missing_panics_doc)]
pub fn ast(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::Item);

    let (head, tail) = match &mut input {
        syn::Item::Enum(enum_) => (enum_repr(enum_), assert_generated_derives(&enum_.attrs)),
        syn::Item::Struct(struct_) => {
            // HACK: temperorary measure to speed up the initial implementation of `node_id`.
            {
                let args = TokenStream2::from(args);
                if args.into_iter().next().is_some_and(
                    |tk| matches!(tk, proc_macro2::TokenTree::Ident(id) if id == "visit" ),
                ) {
                    if let syn::Fields::Named(fields) = &mut struct_.fields {
                        fields
                            .named
                            .insert(0, parse_quote!(pub node_id: ::oxc_syntax::node::NodeId));
                    }
                }
            }
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
/// The only purpose is to allow the occurrence of helper attributes used with the `tasks/ast_tools`.
///
/// Read [`macro@ast`] for further details.
#[proc_macro_derive(Ast, attributes(scope, visit, span, generate_derive, clone_in, serde, tsify))]
pub fn ast_derive(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
