//! Generator for `AstBuilder`.

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    output::{output_path, Output},
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, VariantDef},
    utils::is_reserved_name,
    Codegen, Generator,
};

use super::define_generator;

/// Types to omit builder method for.
const BLACK_LIST: [&str; 1] = ["Span"];

/// Semantic ID types.
/// We generate builder methods both with and without these fields for types which include any of them.
const SEMANTIC_ID_TYPES: [&str; 3] = ["ScopeId", "SymbolId", "ReferenceId"];

/// Generator for `AstBuilder`.
pub struct AstBuilderGenerator;

define_generator!(AstBuilderGenerator);

impl Generator for AstBuilderGenerator {
    /// Generate `AstBuilder`.
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let fns = schema
            .types
            .iter()
            .filter(|&type_def| {
                let is_visited = match type_def {
                    TypeDef::Struct(struct_def) => struct_def.visit.is_visited,
                    TypeDef::Enum(enum_def) => enum_def.visit.is_visited,
                    _ => false,
                };
                let is_blacklisted = BLACK_LIST.contains(&type_def.name());
                is_visited && !is_blacklisted
            })
            .map(|type_def| generate_builder_methods(type_def, schema))
            .collect::<TokenStream>();

        let output = quote! {
            //! AST node factories

            //!@@line_break
            #![allow(
                clippy::default_trait_access,
                clippy::too_many_arguments,
                clippy::fn_params_excessive_bools,
            )]

            ///@@line_break
            use std::cell::Cell;

            ///@@line_break
            use oxc_allocator::{Allocator, Box, IntoIn, Vec};
            use oxc_syntax::{scope::ScopeId, symbol::SymbolId, reference::ReferenceId};

            ///@@line_break
            use crate::ast::*;

            ///@@line_break
            /// AST builder for creating AST nodes
            #[derive(Clone, Copy)]
            pub struct AstBuilder<'a> {
                /// The memory allocator used to allocate AST nodes in the arena.
                pub allocator: &'a Allocator,
            }

            ///@@line_break
            impl<'a> AstBuilder<'a> {
                #fns
            }
        };

        Output::Rust { path: output_path(crate::AST_CRATE, "ast_builder.rs"), tokens: output }
    }
}

/// Param for a builder function.
///
/// Contains reference to the struct field, and various other bits of data derived from it.
#[expect(clippy::struct_field_names)]
struct Param<'d> {
    /// Struct field which this param is for
    field: &'d FieldDef,
    /// Struct field name identifier
    ident: TokenStream,
    /// Function parameter e.g. `span: Span`
    fn_param: TokenStream,
    /// `true` if is a default param (semantic ID)
    is_default: bool,
    /// `true` if this param has a generic param e.g. `type_annotation: T1` (`T1` is generic)
    has_generic_param: bool,
}

/// Generate builder methods for a type.
fn generate_builder_methods(type_def: &TypeDef, schema: &Schema) -> TokenStream {
    match type_def {
        TypeDef::Struct(struct_def) => generate_builder_methods_for_struct(struct_def, schema),
        TypeDef::Enum(enum_def) => generate_builder_methods_for_enum(enum_def, schema),
        _ => unreachable!(),
    }
}

/// Generate builder methods for a struct.
///
/// Generates two builder methods:
/// 1. To build an owned type e.g. `boolean_literal`.
/// 2. To build a boxed type e.g. `alloc_boolean_literal`.
fn generate_builder_methods_for_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    let (mut params, generic_params, where_clause, has_default_fields) =
        get_struct_params(struct_def, schema);
    let (fn_params, fields) = get_struct_fn_params_and_fields(&params, true);

    let (fn_name_postfix, doc_postfix) = if has_default_fields {
        let default_params = params.iter().filter(|param| param.is_default);
        let fn_name_postfix = format!(
            "_with_{}",
            default_params.clone().map(|param| param.field.name()).join("_and_")
        );
        let doc_postfix = format!(
            " with `{}`",
            default_params
                .map(|param| { param.field.type_def(schema).innermost_type(schema).name() })
                .join("` and `")
        );
        (fn_name_postfix, doc_postfix)
    } else {
        (String::new(), String::new())
    };

    // Generate builder functions including all fields (inc default fields)
    let output = generate_builder_methods_for_struct_impl(
        struct_def,
        &params,
        &fn_params,
        &fields,
        &generic_params,
        &where_clause,
        &fn_name_postfix,
        &doc_postfix,
        schema,
    );

    if !has_default_fields {
        return output;
    }

    // Generate builder functions excluding default fields
    let (fn_params, fields) = get_struct_fn_params_and_fields(&params, false);
    params.retain(|param| !param.is_default);
    let mut output2 = generate_builder_methods_for_struct_impl(
        struct_def,
        &params,
        &fn_params,
        &fields,
        &generic_params,
        &where_clause,
        "",
        "",
        schema,
    );

    output2.extend(output);

    output2
}

/// Build a pair of builder methods for a struct.
///
/// This is a separate function as may need to be called twice, with and without semantic ID fields.
#[expect(clippy::too_many_arguments)]
fn generate_builder_methods_for_struct_impl(
    struct_def: &StructDef,
    params: &[Param],
    fn_params: &TokenStream,
    fields: &TokenStream,
    generic_params: &TokenStream,
    where_clause: &TokenStream,
    fn_name_postfix: &str,
    doc_postfix: &str,
    schema: &Schema,
) -> TokenStream {
    let struct_ident = struct_def.ident();
    let struct_ty = struct_def.ty(schema);

    let args = params.iter().map(|param| &param.ident);

    let mut fn_name_base = struct_def.snake_name();
    if !fn_name_postfix.is_empty() {
        fn_name_base.push_str(fn_name_postfix);
    }
    let fn_name = struct_builder_name(&fn_name_base, false);
    let alloc_fn_name = struct_builder_name(&fn_name_base, true);

    // Generate doc comments
    let struct_name = struct_def.name();
    let article = article_for(struct_name);
    let fn_doc1 = format!(" Build {article} [`{struct_name}`]{doc_postfix}.");
    let fn_doc2 = format!(" If you want the built node to be allocated in the memory arena, use [`AstBuilder::{alloc_fn_name}`] instead.");
    let alloc_doc1 = format!(
        " Build {article} [`{struct_name}`]{doc_postfix}, and store it in the memory arena."
    );
    let alloc_doc2 = format!(" Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::{fn_name}`] instead.");
    let params_docs = generate_doc_comment_for_params(params);

    quote! {
        ///@@line_break
        #[doc = #fn_doc1]
        #[doc = ""]
        #[doc = #fn_doc2]
        #params_docs
        #[inline]
        pub fn #fn_name #generic_params (self, #fn_params) -> #struct_ty #where_clause {
            #struct_ident { #fields }
        }

        ///@@line_break
        #[doc = #alloc_doc1]
        #[doc = ""]
        #[doc = #alloc_doc2]
        #params_docs
        #[inline]
        pub fn #alloc_fn_name #generic_params (self, #fn_params) -> Box<'a, #struct_ty> #where_clause {
            Box::new_in(self.#fn_name(#(#args),*), self.allocator)
        }
    }
}

/// Get params for builder method for struct.
///
/// Also generate generic params and where clause for the method.
///
/// ```
/// //        ↓↓↓↓ generic params
/// pub fn foo<T1>(self, span: Span, type_parameters: T1) -> Foo<'a>
///     where T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>> {}
/// //  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ where clause
/// ```
fn get_struct_params<'s>(
    struct_def: &'s StructDef,
    schema: &'s Schema,
) -> (
    Vec<Param<'s>>, // Params
    TokenStream,    // Generic params
    TokenStream,    // `where` clause
    bool,           // Has default fields
) {
    // Only a single `Atom` or `&str` generic supported at present
    let mut has_atom_generic = false;
    let mut has_str_generic = false;
    let mut generic_count = 0u32;
    let mut has_default_fields = false;

    let mut generics = vec![];

    let params = struct_def
        .fields
        .iter()
        .map(|field| {
            let type_def = field.type_def(schema);
            let ty = type_def.ty(schema);

            let is_default = SEMANTIC_ID_TYPES.contains(&type_def.innermost_type(schema).name());
            if is_default {
                has_default_fields = true;
            };

            let generic_ident = match type_def {
                TypeDef::Primitive(primitive_def) => match primitive_def.name() {
                    "Atom" if !has_atom_generic => {
                        has_atom_generic = true;
                        Some(format_ident!("A"))
                    }
                    "&str" if !has_str_generic => {
                        has_str_generic = true;
                        Some(format_ident!("S"))
                    }
                    _ => None,
                },
                TypeDef::Box(_) => {
                    generic_count += 1;
                    Some(format_ident!("T{generic_count}"))
                }
                TypeDef::Option(option_def) if option_def.inner_type(schema).is_box() => {
                    generic_count += 1;
                    Some(format_ident!("T{generic_count}"))
                }
                _ => None,
            };
            let has_generic_param = generic_ident.is_some();

            let fn_param_ty = if is_default {
                assert!(!has_generic_param);
                type_def.innermost_type(schema).ty(schema)
            } else if let Some(generic_ident) = generic_ident {
                let where_clause_part = quote!( #generic_ident: IntoIn<'a, #ty> );
                let generic_ty = quote!( #generic_ident );
                generics.push((generic_ident, where_clause_part));
                generic_ty
            } else {
                ty
            };

            let field_ident = field.ident();
            let fn_param = quote!( #field_ident: #fn_param_ty );

            Param { field, ident: field_ident, fn_param, is_default, has_generic_param }
        })
        .collect();

    let (generic_params, where_clause) = if generics.is_empty() {
        (quote!(), quote!())
    } else {
        let generic_params = generics.iter().map(|(generic_ident, _)| generic_ident);
        let generic_params = quote!( <#(#generic_params),*> );
        let where_clause = generics.iter().map(|(_, where_clause_part)| where_clause_part);
        let where_clause = quote!( where #(#where_clause),* );
        (generic_params, where_clause)
    };

    (params, generic_params, where_clause, has_default_fields)
}

/// Get function params and fields for a struct builder method.
///
/// Omit default fields from function params if `include_default_fields == true`.
///
/// ```
/// //         ↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓ function params
/// pub fn foo(span: Span, bar: Bar<'a>) -> Foo<'a> {
///     Bar { span, bar }
/// //        ^^^^^^^^^ fields
/// }
/// ```
fn get_struct_fn_params_and_fields(
    params: &[Param],
    include_default_fields: bool,
) -> (/* function params */ TokenStream, /* fields */ TokenStream) {
    let mut fields = vec![];
    let fn_params = params.iter().filter_map(|param| {
        let param_ident = &param.ident;

        if param.is_default {
            if include_default_fields {
                fields.push(quote!( #param_ident: Cell::new(Some(#param_ident)) ));
                return Some(&param.fn_param);
            }

            fields.push(quote!( #param_ident: Default::default() ));
            return None;
        }

        let field = if param.has_generic_param {
            quote!( #param_ident: #param_ident.into_in(self.allocator) )
        } else {
            quote!( #param_ident )
        };

        fields.push(field);

        Some(&param.fn_param)
    });

    let fn_params = quote!( #(#fn_params),* );
    let fields = quote!( #(#fields),* );
    (fn_params, fields)
}

/// Generate builder methods for an enum.
///
/// Generates a builder method for every variant of the enum (not including inherited variants).
fn generate_builder_methods_for_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    enum_def
        .variants
        .iter()
        .map(|variant| generate_builder_method_for_enum_variant(enum_def, variant, schema))
        .collect()
}

/// Generate builder method for an enum variant.
#[expect(clippy::similar_names)]
fn generate_builder_method_for_enum_variant(
    enum_def: &EnumDef,
    variant: &VariantDef,
    schema: &Schema,
) -> TokenStream {
    let mut variant_type = variant.field_type(schema).unwrap();
    let mut is_boxed = false;
    if let TypeDef::Box(box_def) = variant_type {
        variant_type = box_def.inner_type(schema);
        is_boxed = true;
    }
    let TypeDef::Struct(variant_type) = variant_type else { panic!("Unsupported!") };

    let (mut params, generic_params, where_clause, has_default_fields) =
        get_struct_params(variant_type, schema);
    if has_default_fields {
        params.retain(|param| !param.is_default);
    }

    let fn_params = params.iter().map(|param| &param.fn_param);
    let args = params.iter().map(|param| &param.ident);

    let enum_ident = enum_def.ident();
    let enum_ty = enum_def.ty(schema);
    let fn_name = enum_variant_builder_name(enum_def, variant, schema);
    let variant_ident = variant.ident();
    let inner_builder_name = struct_builder_name(&variant_type.snake_name(), is_boxed);

    // Generate doc comments
    let enum_name = enum_def.name();
    let article_enum = article_for(enum_name);
    let variant_name = variant.ident();
    let fn_doc1 = format!(" Build {article_enum} [`{enum_name}::{variant_name}`].");
    let mut fn_docs = quote!( #[doc = #fn_doc1] );
    if is_boxed {
        let variant_type_name = variant_type.name();
        let article_variant = article_for(variant_type_name);
        let fn_doc2 = format!(
            " This node contains {article_variant} [`{variant_type_name}`] that will be stored in the memory arena."
        );
        fn_docs.extend(quote!( #[doc = ""] #[doc = #fn_doc2] ));
    }
    let params_docs = generate_doc_comment_for_params(&params);

    quote! {
        ///@@line_break
        #fn_docs
        #params_docs
        #[inline]
        pub fn #fn_name #generic_params(self, #(#fn_params),*) -> #enum_ty #where_clause {
            #enum_ident::#variant_ident(self.#inner_builder_name(#(#args),*))
        }
    }
}

/// Get name of struct builder method.
///
/// If `does_alloc == true`, prepends `alloc_` to start of name.
fn struct_builder_name(snake_name: &str, does_alloc: bool) -> Ident {
    if does_alloc {
        format_ident!("alloc_{snake_name}")
    } else if is_reserved_name(snake_name) {
        format_ident!("{snake_name}_")
    } else {
        format_ident!("{snake_name}")
    }
}

/// Get name of enum variant builder method.
fn enum_variant_builder_name(enum_def: &EnumDef, variant: &VariantDef, schema: &Schema) -> Ident {
    let enum_name = enum_def.snake_name();

    // TODO: `let variant_name = variant.snake_name();` would be better
    let mut variant_type = variant.field_type(schema).unwrap();
    if let TypeDef::Box(box_def) = variant_type {
        variant_type = box_def.inner_type(schema);
    }
    let variant_name = variant_type.snake_name();

    let variant_name = if variant_name.len() > enum_name.len()
        && variant_name.ends_with(&enum_name)
        && variant_name.as_bytes()[variant_name.len() - enum_name.len() - 1] == b'_'
    {
        // Replace `xxx_yyy_xxx` with `xxx_yyy`
        &variant_name[..variant_name.len() - enum_name.len() - 1]
    } else if enum_name.starts_with("ts_") && variant_name.starts_with("ts_") {
        // Replace `ts_xxx_ts_yyy` with `ts_xxx_yyy`
        &variant_name[3..]
    } else {
        &variant_name
    };

    format_ident!("{enum_name}_{variant_name}")
}

/// Generate doc comment for function params.
fn generate_doc_comment_for_params(params: &[Param]) -> TokenStream {
    if params.is_empty() {
        return quote!();
    }

    let lines = params.iter().map(|param| {
        let field = param.field;
        // TODO: `field.name()` would be better.
        let field_ident = field.ident();
        let field_comment = if let Some(field_comment) = field.doc_comment.as_deref() {
            format!(" * `{field_ident}`: {field_comment}")
        } else if field.name() == "span" {
            " * `span`: The [`Span`] covering this node".to_string()
        } else {
            format!(" * `{field_ident}`")
        };
        quote!( #[doc = #field_comment] )
    });

    quote! {
        ///
        /// ## Parameters
        #(#lines)*
    }
}

/// Get the correct article ("a" / "an") that should precede a word in a doc comment.
fn article_for(word: &str) -> &'static str {
    match word.as_bytes().first().map(u8::to_ascii_uppercase) {
        Some(b'A' | b'E' | b'I' | b'O' | b'U') => "an",
        _ => "a",
    }
}
