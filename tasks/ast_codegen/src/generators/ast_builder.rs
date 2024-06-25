use std::collections::HashMap;
use std::stringify;

use convert_case::{Case, Casing};
use itertools::Itertools;
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, FnArg, Ident, ImplItemFn, PatType, Type};

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
            use oxc_allocator::{Allocator, Box, Vec};
            use oxc_span::{Atom, SourceType, Span};
            use oxc_syntax::{
                number::{BigintBase, NumberBase},
                operator::{
                    AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
                },
            };

            endl!();

            #[allow(clippy::wildcard_imports)]
            use crate::ast::*;

            endl!();

            /// AST builder for creating AST nodes
            #[derive(Clone, Copy)]
            pub struct AstBuilder<'a> {
                pub allocator: &'a Allocator,
            }

            endl!();

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

fn default_init_field((ident, typ): &(&Ident, &Type)) -> bool {
    macro_rules! field {
        ($ident:ident: $ty:ty) => {
            (stringify!($ident), stringify!($ty))
        };
    }
    lazy_static! {
        static ref DEFAULT_FIELDS: HashMap<&'static str, &'static str> = HashMap::from([
            field!(scope_id: Cell<Option<ScopeId>>),
            field!(symbol_id: Cell<Option<SymbolId>>),
            field!(reference_id: Cell<Option<ReferenceId>>),
            field!(reference_flag: ReferenceFlag),
        ]);
    }
    if let Some(default_type) = DEFAULT_FIELDS.get(ident.to_string().as_str()) {
        *default_type == typ.to_token_stream().to_string().replace(' ', "")
    } else {
        false
    }
}

fn generate_struct_builder_fn(ty: &RStruct) -> ImplItemFn {
    let ident = ty.ident();
    let as_type = ty.as_type();
    let fn_name = fn_ident(ty.ident());

    let ident_types =
        ty.item.fields.iter().map(|f| (f.ident.as_ref().expect("expected named ident!"), &f.ty));
    // .collect_vec();
    let (actual_fields, default_fields) =
        ident_types.fold((Vec::new(), Vec::new()), |mut acc, it| {
            if default_init_field(&it) {
                acc.1.push(it);
            } else {
                acc.0.push(it);
            }
            acc
        });

    let params: Vec<PatType> =
        actual_fields.iter().map(|(ident, typ)| parse_quote!(#ident: #typ)).collect_vec();
    let default_fields =
        default_fields.into_iter().map(|(ident, _)| quote!(#ident: Default::default()));
    let fields = actual_fields
        .into_iter()
        .map(|(ident, _)| ident.to_token_stream())
        .chain(default_fields)
        .collect_vec();

    parse_quote! {
        fn #fn_name(self, #(#params),*) -> #as_type {
            #ident { #(#fields),* }
        }
    }
}
