use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, FnArg, Ident, ImplItemFn, PatType};

use crate::{
    schema::{REnum, RStruct, RType},
    CodegenCtx, Generator, GeneratorOutput, TypeRef,
};

pub struct AstBuilderGenerator;

impl Generator for AstBuilderGenerator {
    fn name(&self) -> &'static str {
        "AstBuilderGenerator"
    }

    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput {
        let fns: Vec<ImplItemFn> = ctx.ty_table.iter().filter_map(generate_builder_fn).collect();

        GeneratorOutput::One(quote! {
            impl<'a> AstBuilder<'a> {
                #(#fns)*
            }
        })
    }
}

fn fn_ident(ident: &Ident) -> Ident {
    let fn_name = ident.to_string().to_case(Case::Snake);
    let fn_name = if RUST_KEYWORDS.contains(&fn_name.as_str()) {
        let mut fn_name = fn_name;
        fn_name.push('_');
        fn_name
    } else {
        fn_name
    };

    format_ident!("{fn_name}")
}

fn generate_builder_fn(ty: &TypeRef) -> Option<ImplItemFn> {
    match &*ty.borrow() {
        RType::Enum(it) => None,
        // RType::Enum(it) => Some(generate_enum_builder_fn(it)),
        RType::Struct(it) => Some(generate_struct_builder_fn(it)),
        _ => None,
    }
}

fn generate_enum_builder_fn(ty: &REnum) -> ImplItemFn {
    let fn_name = fn_ident(ty.ident());

    parse_quote! {
        pub fn #fn_name (self, )
    }
}

const RUST_KEYWORDS: [&str; 1] = ["super"];

fn generate_struct_builder_fn(ty: &RStruct) -> ImplItemFn {
    let ident = ty.ident();
    let fn_name = fn_ident(ty.ident());

    let ident_types = ty
        .item
        .fields
        .iter()
        .map(|f| (f.ident.as_ref().expect("expected named ident!"), &f.ty))
        .collect_vec();

    let params: Vec<PatType> =
        ident_types.iter().map(|(ident, typ)| parse_quote!(#ident: #typ)).collect_vec();
    let fields = ident_types.into_iter().map(|(ident, _)| ident).collect_vec();

    parse_quote! {
        fn #fn_name(self, #(#params),*) -> #ident {
            #ident { #(#fields),* }
        }
    }
}
