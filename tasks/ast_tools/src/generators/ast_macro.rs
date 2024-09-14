//! Generator to generate the implementation of `#[ast]` macro.

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::define_generator;
use crate::{
    codegen::{generated_header, LateCtx},
    output,
    schema::TypeDef,
    to_code::{code, to_code},
    Generator, GeneratorOutput, AST_MACROS_CRATE,
};

define_generator! {
    pub struct AstMacroGenerator;
}

impl Generator for AstMacroGenerator {
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let (trait_names_and_assert_fn_names, assert_fns) = get_assert_fns();

        let (gen_fns, match_arms): (Vec<_>, Vec<_>) = ctx
            .schema()
            .into_iter()
            .map(|def| generate_type_fn(def, &trait_names_and_assert_fn_names))
            .unzip();

        let header = generated_header!();

        let derive_ast_fn = get_derive_ast_fn();
        let repr_fns = get_repr_fns();

        GeneratorOutput(
            output(AST_MACROS_CRATE, "ast.rs"),
            quote! {
                #header

                #![allow(clippy::useless_conversion)]

                ///@@line_break
                #[allow(unused_imports)]
                use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

                ///@@line_break
                pub fn gen(name: &str, input: TokenStream) -> TokenStream {
                    match name {
                        #(#match_arms,)*
                        _ => unreachable!(),
                    }
                }

                ///@@line_break
                #(#gen_fns)*

                ///@@line_break
                #derive_ast_fn

                ///@@line_break
                #repr_fns

                ///@@line_break
                #assert_fns
            },
        )
    }
}

/// Generate function to generate macro expansion of `#[ast]` for a type
fn generate_type_fn(
    def: &TypeDef,
    trait_names_and_assert_fn_names: &[(&str, Ident)],
) -> (TokenStream, TokenStream) {
    let (name, repr, derives) = match def {
        TypeDef::Struct(struct_) => {
            let name = struct_.name.as_str();
            let repr = quote!(repr_c());
            let derives = &struct_.generated_derives;
            (name, repr, derives)
        }
        TypeDef::Enum(enum_) => {
            let name = enum_.name.as_str();
            let repr = if enum_.variants.iter().any(|variant| !variant.fields.is_empty()) {
                quote!(repr_c_u8())
            } else {
                quote!(repr_u8())
            };
            let derives = &enum_.generated_derives;
            (name, repr, derives)
        }
    };

    let add_generated_derive_assertions = {
        let mut derives = derives.clone();
        derives.sort_unstable();

        let assertions = derives.iter().map(|trait_name| {
            let (_, assert_fn_name) = trait_names_and_assert_fn_names
                .iter()
                .find(|(name, ..)| name == trait_name)
                .unwrap_or_else(|| {
                    panic!(
                        "Invalid derive trait(generate_derive): {trait_name}.\n\
                        Help: If you are trying to implement a new `generate_derive` trait, \
                        make sure to add it to the list in `get_assert_fns` function."
                    );
                });
            quote! { stream.extend(#assert_fn_name()); }
        });
        quote! { #(#assertions)* }
    };

    let gen_fn_name = Ident::new(&format!("gen_{}", name.to_case(Case::Snake)), Span::call_site());
    let gen_fn = quote! {
        ///@@line_break
        fn #gen_fn_name(input: TokenStream) -> TokenStream {
            let mut stream = derive_ast();
            stream.extend(#repr);
            stream.extend(input);
            #add_generated_derive_assertions
            stream
        }
    };

    let match_arm = quote! { #name => #gen_fn_name(input) };

    (gen_fn, match_arm)
}

/// Generate `derive_ast` function which constructs token stream for `#[derive(::oxc_ast_macros::Ast)]`
fn get_derive_ast_fn() -> TokenStream {
    let derive_ast = code!( #[derive(::oxc_ast_macros::Ast)] );
    quote! {
        fn derive_ast() -> TokenStream {
            #derive_ast
        }
    }
}

/// Generate `repr` functions which constructs token streams for `#[repr(C)]`, `#[repr(u8)]`, `#[repr(C, u8)]`
fn get_repr_fns() -> TokenStream {
    let c_ident = code!(C);
    let u8_ident = code!(u8);
    let c_u8_seq = code!(C, u8);
    let repr = code!(#[repr(@{rep})]);
    quote! {
        ///@@line_break
        fn repr_c() -> TokenStream {
            repr(#c_ident)
        }

        ///@@line_break
        fn repr_u8() -> TokenStream {
            repr(#u8_ident)
        }

        ///@@line_break
        fn repr_c_u8() -> TokenStream {
            repr(#c_u8_seq)
        }

        ///@@line_break
        fn repr(rep: TokenStream) -> TokenStream {
            #repr
        }
    }
}

/// Generate derive assertion functions
fn get_assert_fns() -> (Vec<(&'static str, Ident)>, TokenStream) {
    let (trait_names_and_assert_fn_names, assert_fns): (Vec<_>, Vec<_>) = [
        ("CloneIn", quote!(::oxc_allocator), true),
        ("GetSpan", quote!(::oxc_span), false),
        ("GetSpanMut", quote!(::oxc_span), false),
        ("ContentEq", quote!(::oxc_span::cmp), false),
        ("ContentHash", quote!(::oxc_span::hash), false),
    ]
    .into_iter()
    .map(|(trait_name, trait_path, has_lifetime)| {
        let trait_ident = Ident::new(trait_name, Span::call_site());
        let lifetime = if has_lifetime { quote!(<'static>) } else { TokenStream::new() };
        let trait_ident = quote! { #trait_ident #lifetime };

        let fn_name = format!("assert_{}", trait_name.to_case(Case::Snake));
        let fn_name = Ident::new(&fn_name, Span::call_site());

        let trait_name_code = to_code(trait_ident.clone());
        let trait_path_code = code!(#trait_path :: #trait_ident);
        let fn_def = quote! {
            ///@@line_break
            fn #fn_name() -> TokenStream {
                assert(#trait_name_code, #trait_path_code)
            }
        };

        ((trait_name, fn_name), fn_def)
    })
    .unzip();

    let assertion = code! {
        const _: () = {
            // These are wrapped in a scope to avoid the need for unique identifiers
            trait AssertionTrait: @{path} {}
            impl<T: @{name}> AssertionTrait for T {}
        };
    };
    let assert_fns = quote! {
        #(#assert_fns)*

        ///@@line_break
        fn assert(name: TokenStream, path: TokenStream) -> TokenStream {
            #assertion
        }
    };

    (trait_names_and_assert_fn_names, assert_fns)
}
