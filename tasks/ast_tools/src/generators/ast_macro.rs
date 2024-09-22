//! Generator to generate the implementation of `#[ast]` macro.

use std::borrow::Cow;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use rustc_hash::{FxHashMap, FxHashSet};

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

enum Repr {
    C,
    U8,
    CU8,
}

struct TypeInfo<'t> {
    def: &'t TypeDef,
    repr: Repr,
    first_in_module: Option<&'t str>,
}

impl Generator for AstMacroGenerator {
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        // Get info about types, and record what derive assertions are required for each module.
        // Instead of outputting same derive assertions for lots of types, output them only once in
        // each module, in the expansion of `#[ast]` macro for 1st type in that module.
        let mut modules = FxHashMap::default();
        let type_infos = ctx
            .schema()
            .into_iter()
            .map(|def| get_type_info(def, &mut modules))
            .collect::<Vec<_>>();

        // Generate derive assertion functions
        let (trait_names_and_assert_fn_names, assert_fns) = get_assert_fns();

        // Generate generator functions, and record all the types which use each generator function
        let mut gen_fns = vec![];
        let mut fn_types: FxHashMap<Cow<'static, str>, Vec<&str>> = FxHashMap::default();
        for info in type_infos {
            let gen_fn_name =
                generate_type_fn(&info, &trait_names_and_assert_fn_names, &modules, &mut gen_fns);
            let type_names = fn_types.entry(gen_fn_name).or_default();
            type_names.push(info.def.name());
        }

        // Generate match arms for `gen` function
        let mut fn_types = fn_types.into_iter().collect::<Vec<_>>();
        fn_types.sort_unstable_by(|(gen_fn_name1, _), (gen_fn_name2, _)| {
            gen_fn_name1.cmp(gen_fn_name2)
        });
        let match_arms = fn_types.into_iter().map(|(gen_fn_name, mut type_names)| {
            let gen_fn_name = Ident::new(&gen_fn_name, Span::call_site());
            type_names.sort_unstable();
            quote! { #(#type_names)|* => #gen_fn_name(input) }
        });

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

/// Get info about type.
///
/// * Determine what `#[repr]` attribute it needs.
/// * Determine if is first type in its module
///   (in which case all derive assertions for the module will be made part of it's `#[ast] expansion`).
/// * Record what derive assertions it needs in `modules`.
fn get_type_info<'t>(
    def: &'t TypeDef,
    modules: &mut FxHashMap<String, FxHashSet<String>>,
) -> TypeInfo<'t> {
    let (repr, derives, module_path) = match def {
        TypeDef::Struct(struct_) => {
            let repr = Repr::C;
            let derives = &*struct_.generated_derives;
            let module_path = &*struct_.module_path;
            (repr, derives, module_path)
        }
        TypeDef::Enum(enum_) => {
            let repr = if enum_.variants.iter().any(|variant| !variant.fields.is_empty()) {
                Repr::CU8
            } else {
                Repr::U8
            };
            let derives = &*enum_.generated_derives;
            let module_path = &*enum_.module_path;
            (repr, derives, module_path)
        }
    };

    let (first_in_module, recorded_derives) = if let Some(derives) = modules.get_mut(module_path) {
        (None, derives)
    } else {
        modules.insert(module_path.to_string(), FxHashSet::default());
        let recorded_derives = modules.get_mut(module_path).unwrap();
        (Some(module_path), recorded_derives)
    };

    for derive in derives {
        recorded_derives.insert(derive.clone());
    }

    TypeInfo { def, repr, first_in_module }
}

/// Generate function to generate macro expansion of `#[ast]` for a type.
///
/// Store `TokenStream` for that function in `gen_fns`.
/// Or if `#[ast]` expansion for this type does not include derive assertions,
/// use one of the `repr_*` functions.
fn generate_type_fn(
    info: &TypeInfo,
    trait_names_and_assert_fn_names: &[(&str, Ident)],
    modules: &FxHashMap<String, FxHashSet<String>>,
    gen_fns: &mut Vec<TokenStream>,
) -> Cow<'static, str> {
    let repr = match info.repr {
        Repr::C => "repr_c",
        Repr::U8 => "repr_u8",
        Repr::CU8 => "repr_c_u8",
    };

    let derives = match info.first_in_module {
        Some(module_path) => {
            let derives = modules.get(module_path).unwrap();
            if derives.is_empty() {
                None
            } else {
                Some(derives)
            }
        }
        None => None,
    };

    let Some(derives) = derives else { return Cow::Borrowed(repr) };

    let mut derives = derives.iter().collect::<Vec<_>>();
    derives.sort_unstable();

    let name = info.def.name().as_str();
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

    let gen_fn_name = format!("gen_{}", name.to_case(Case::Snake));
    let gen_fn_ident = Ident::new(&gen_fn_name, Span::call_site());
    let repr = Ident::new(repr, Span::call_site());
    let gen_fn = quote! {
        ///@@line_break
        fn #gen_fn_ident(input: TokenStream) -> TokenStream {
            let mut stream = #repr(input);
            #(#assertions)*
            stream
        }
    };
    gen_fns.push(gen_fn);

    Cow::Owned(gen_fn_name)
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

/// Generate `repr` functions which construct token streams for `#[repr(C)]`, `#[repr(u8)]`, `#[repr(C, u8)]`
fn get_repr_fns() -> TokenStream {
    let c_ident = code!(C);
    let u8_ident = code!(u8);
    let c_u8_seq = code!(C, u8);
    let repr = code!(#[repr(@{rep})]);
    quote! {
        ///@@line_break
        fn repr_c(input: TokenStream) -> TokenStream {
            repr(#c_ident, input)
        }

        ///@@line_break
        fn repr_u8(input: TokenStream) -> TokenStream {
            repr(#u8_ident, input)
        }

        ///@@line_break
        fn repr_c_u8(input: TokenStream) -> TokenStream {
            repr(#c_u8_seq, input)
        }

        ///@@line_break
        fn repr(rep: TokenStream, input: TokenStream) -> TokenStream {
            let mut stream = derive_ast();
            stream.extend(repr_raw(rep));
            stream.extend(input);
            stream
        }

        ///@@line_break
        fn repr_raw(rep: TokenStream) -> TokenStream {
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
