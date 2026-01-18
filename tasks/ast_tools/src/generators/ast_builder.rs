//! Generator for `AstBuilder`.

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    AST_CRATE_PATH, Codegen, Generator, Result,
    output::{Output, output_path},
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, TypeId, VariantDef},
    utils::{create_safe_ident, is_reserved_name},
};

use super::{AttrLocation, AttrPart, AttrPositions, attr_positions, define_generator};

/// Generator for `AstBuilder`.
pub struct AstBuilderGenerator;

define_generator!(AstBuilderGenerator);

impl Generator for AstBuilderGenerator {
    /// Register that accept `#[builder]` attr on structs, enums, or struct fields.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("builder", attr_positions!(Struct | Enum | StructField))]
    }

    /// Parse `#[builder(default)]` on struct, enum, or struct field,
    /// and `#[builder(skip)]` on struct or enum.
    fn parse_attr(&self, _attr_name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        // No need to check attr name is `builder`, because that's the only attribute that
        // this generator handles.
        match part {
            AttrPart::Tag("default") => match location {
                AttrLocation::Struct(struct_def) => struct_def.builder.is_default = true,
                AttrLocation::Enum(enum_def) => enum_def.builder.is_default = true,
                AttrLocation::StructField(struct_def, field_index) => {
                    struct_def.fields[field_index].builder.is_default = true;
                }
                _ => return Err(()),
            },
            AttrPart::Tag("skip") => match location {
                AttrLocation::Struct(struct_def) => struct_def.builder.skip = true,
                AttrLocation::Enum(enum_def) => enum_def.builder.skip = true,
                _ => return Err(()),
            },
            _ => return Err(()),
        }

        Ok(())
    }

    /// Generate `AstBuilder`.
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let comment_node_id_type_id = schema.type_names["CommentNodeId"];

        let fns = schema
            .types
            .iter()
            .filter(|&type_def| match type_def {
                TypeDef::Struct(struct_def) => {
                    !struct_def.builder.skip && struct_def.visit.has_visitor()
                }
                TypeDef::Enum(enum_def) => !enum_def.builder.skip && enum_def.visit.has_visitor(),
                _ => false,
            })
            .map(|type_def| generate_builder_methods(type_def, comment_node_id_type_id, schema))
            .collect::<TokenStream>();

        let output = quote! {
            //! AST node factories

            //!@@line_break
            #![allow(unused_imports)]
            #![expect(
                clippy::default_trait_access,
                clippy::unused_self,
            )]

            ///@@line_break
            use std::cell::Cell;

            ///@@line_break
            use oxc_allocator::{Allocator, Box, IntoIn, Vec};
            use oxc_syntax::{
                comment_node::CommentNodeId,
                node::NodeId,
                scope::ScopeId,
                symbol::SymbolId,
                reference::ReferenceId
            };

            ///@@line_break
            use crate::{AstBuilder, ast::*};

            ///@@line_break
            impl<'a> AstBuilder<'a> {
                #fns
            }
        };

        Output::Rust { path: output_path(AST_CRATE_PATH, "ast_builder.rs"), tokens: output }
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
    /// `true` if is `CommentNodeId` field
    is_comment_node_id: bool,
    /// `true` if is `node_id` field
    is_node_id: bool,
    /// * `None` if param is not generic.
    /// * `Some(GenericType::Into)` if is generic and uses `Into`
    ///   e.g. `name: A where A: Into<Atom<'a>>`.
    /// * `Some(GenericType::IntoIn)` if is generic and uses `IntoIn`
    ///   e.g. `type_annotation: T1 where T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>`.
    generic_type: Option<GenericType>,
}

/// Type of generic param.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GenericType {
    Into,
    IntoIn,
}

/// Generate builder methods for a type.
fn generate_builder_methods(
    type_def: &TypeDef,
    comment_node_id_type_id: TypeId,
    schema: &Schema,
) -> TokenStream {
    match type_def {
        TypeDef::Struct(struct_def) => {
            generate_builder_methods_for_struct(struct_def, comment_node_id_type_id, schema)
        }
        TypeDef::Enum(enum_def) => {
            generate_builder_methods_for_enum(enum_def, comment_node_id_type_id, schema)
        }
        _ => unreachable!(),
    }
}

/// Generate builder methods for a struct.
///
/// Generates two builder methods:
/// 1. To build an owned type e.g. `boolean_literal`.
/// 2. To build a boxed type e.g. `alloc_boolean_literal`.
fn generate_builder_methods_for_struct(
    struct_def: &StructDef,
    comment_node_id_type_id: TypeId,
    schema: &Schema,
) -> TokenStream {
    let (mut params, generic_params, where_clause, has_default_fields) =
        get_struct_params(struct_def, comment_node_id_type_id, schema);
    let (fn_params, fields) = get_struct_fn_params_and_fields(&params, true, schema);

    let (fn_name_postfix, doc_postfix) = if has_default_fields {
        // Exclude node_id from the list of default params (it's always set to NodeId::DUMMY)
        let default_params = params.iter().filter(|param| param.is_default && !param.is_node_id);
        let fn_name_postfix = format!(
            "_with_{}",
            default_params.clone().map(|param| param.field.name()).join("_and_")
        );
        let doc_postfix =
            format!(" with `{}`", default_params.map(|param| param.field.name()).join("` and `"));
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
    let (fn_params, fields) = get_struct_fn_params_and_fields(&params, false, schema);
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

    let args = params
        .iter()
        .filter(|param| !param.is_comment_node_id && !param.is_node_id)
        .map(|param| &param.ident);

    let mut fn_name_base = struct_def.snake_name();
    if !fn_name_postfix.is_empty() {
        fn_name_base.push_str(fn_name_postfix);
    }
    let fn_name = struct_builder_name(&fn_name_base, false);

    // Only generate an `alloc_*` method if `Box<T>` exists in AST
    let alloc_fn_name = if struct_def.containers.box_id.is_some() {
        Some(struct_builder_name(&fn_name_base, true))
    } else {
        None
    };

    // Generate main builder method
    let struct_name = struct_def.name();
    let article = article_for(struct_name);
    let fn_doc1 = format!(" Build {article} [`{struct_name}`]{doc_postfix}.");
    let mut fn_docs = quote!( #[doc = #fn_doc1] );
    if let Some(alloc_fn_name) = &alloc_fn_name {
        let fn_doc2 = format!(" use [`AstBuilder::{alloc_fn_name}`] instead.");
        fn_docs.extend(quote! {
            #[doc = ""]
            #[doc = " If you want the built node to be allocated in the memory arena,"]
            #[doc = #fn_doc2]
        });
    }

    let params_docs = generate_doc_comment_for_params(params);

    let method = quote! {
        ///@@line_break
        #fn_docs
        #params_docs
        #[inline]
        pub fn #fn_name #generic_params (self, #fn_params) -> #struct_ty #where_clause {
            #struct_ident { #fields }
        }
    };

    let Some(alloc_fn_name) = alloc_fn_name else { return method };

    // Generate `alloc_*` builder method, if required
    let alloc_doc1 = format!(
        " Build {article} [`{struct_name}`]{doc_postfix}, and store it in the memory arena."
    );
    let alloc_doc2 =
        format!(" If you want a stack-allocated node, use [`AstBuilder::{fn_name}`] instead.");

    quote! {
        #method

        ///@@line_break
        #[doc = #alloc_doc1]
        #[doc = ""]
        #[doc = " Returns a [`Box`] containing the newly-allocated node."]
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
    comment_node_id_type_id: TypeId,
    schema: &'s Schema,
) -> (
    Vec<Param<'s>>, // Params
    TokenStream,    // Generic params
    TokenStream,    // `where` clause
    bool,           // Has default fields
) {
    let mut generic_count = 0u32;
    let mut atom_generic_count = 0u32;
    let mut has_default_fields = false;

    let mut generics = vec![];

    let params = struct_def
        .fields
        .iter()
        .map(|field| {
            let type_def = field.type_def(schema);
            let ty = type_def.ty(schema);

            // A field is default if the field is marked `#[builder(default)]`,
            // or its innermost type is marked `#[builder(default)]`
            let is_default = field.builder.is_default || {
                let innermost_type = type_def.innermost_type(schema);
                match innermost_type {
                    TypeDef::Struct(inner_struct) => inner_struct.builder.is_default,
                    TypeDef::Enum(inner_enum) => inner_enum.builder.is_default,
                    _ => false,
                }
            };
            if is_default {
                has_default_fields = true;
            }

            let generic_details = match type_def {
                TypeDef::Primitive(primitive_def) if primitive_def.name() == "Atom" => {
                    atom_generic_count += 1;
                    Some((format_ident!("A{atom_generic_count}"), GenericType::Into))
                }
                TypeDef::Box(_) => {
                    generic_count += 1;
                    Some((format_ident!("T{generic_count}"), GenericType::IntoIn))
                }
                TypeDef::Option(option_def) if option_def.inner_type(schema).is_box() => {
                    generic_count += 1;
                    Some((format_ident!("T{generic_count}"), GenericType::IntoIn))
                }
                _ => None,
            };

            let (fn_param_ty, generic_type) = if is_default {
                assert!(generic_details.is_none());
                let ty = type_def.innermost_type(schema).ty(schema);
                (ty, None)
            } else if let Some((generic_ident, generic_type)) = generic_details {
                let where_clause_part = match generic_type {
                    GenericType::Into => quote!( #generic_ident: Into<#ty> ),
                    GenericType::IntoIn => quote!( #generic_ident: IntoIn<'a, #ty> ),
                };
                let generic_ty = quote!( #generic_ident );
                generics.push((generic_ident, where_clause_part));
                (generic_ty, Some(generic_type))
            } else {
                (ty, None)
            };

            let field_ident = field.ident();
            let fn_param = quote!( #field_ident: #fn_param_ty );

            let is_comment_node_id = field.type_id == comment_node_id_type_id;
            let is_node_id = field.name() == "node_id";
            Param {
                field,
                ident: field_ident,
                fn_param,
                is_default,
                is_comment_node_id,
                is_node_id,
                generic_type,
            }
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
    schema: &Schema,
) -> (/* function params */ TokenStream, /* fields */ TokenStream) {
    let mut fields = vec![];
    let fn_params = params.iter().filter_map(|param| {
        let param_ident = &param.ident;

        // Special case: node_id always uses NodeId::DUMMY and is never a parameter
        // Must check before is_default to handle cases where node_id might be marked as default
        if param.is_node_id {
            fields.push(quote!( #param_ident: NodeId::DUMMY ));
            return None;
        } else if param.is_default {
            if include_default_fields {
                // Builder functions which take default fields receive the innermost type as param.
                // So wrap the param's value in `Cell::new(...)`, or `Some(...)` if necessary.
                let field_type = param.field.type_def(schema);
                let value = wrap_default_field_value(quote!( #param_ident ), field_type, schema);

                fields.push(quote!( #param_ident: #value ));
                return Some(&param.fn_param);
            }

            fields.push(quote!( #param_ident: Default::default() ));
            return None;
        } else if param.is_comment_node_id {
            fields.push(quote!( #param_ident: self.get_comment_node_id() ));
            return None;
        }

        let field = match param.generic_type {
            Some(GenericType::Into) => quote!( #param_ident: #param_ident.into() ),
            Some(GenericType::IntoIn) => {
                quote!( #param_ident: #param_ident.into_in(self.allocator) )
            }
            None => quote!( #param_ident ),
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
fn generate_builder_methods_for_enum(
    enum_def: &EnumDef,
    comment_node_id_type_id: TypeId,
    schema: &Schema,
) -> TokenStream {
    enum_def
        .variants
        .iter()
        .map(|variant| {
            generate_builder_method_for_enum_variant(
                enum_def,
                variant,
                comment_node_id_type_id,
                schema,
            )
        })
        .collect()
}

/// Generate builder method for an enum variant.
fn generate_builder_method_for_enum_variant(
    enum_def: &EnumDef,
    variant: &VariantDef,
    comment_node_id_type_id: TypeId,
    schema: &Schema,
) -> TokenStream {
    let mut variant_type = variant.field_type(schema).unwrap();
    let mut is_boxed = false;
    if let TypeDef::Box(box_def) = variant_type {
        variant_type = box_def.inner_type(schema);
        is_boxed = true;
    }
    let TypeDef::Struct(struct_def) = variant_type else { panic!("Unsupported!") };

    let (mut params, generic_params, where_clause, has_default_fields) =
        get_struct_params(struct_def, comment_node_id_type_id, schema);

    let fn_name = enum_variant_builder_name(enum_def, variant);
    let variant_ident = variant.ident();

    let output = has_default_fields.then(|| {
        // Exclude node_id from the list of default params (it's always set to NodeId::DUMMY)
        let default_params = params.iter().filter(|param| param.is_default && !param.is_node_id);
        let fn_name_postfix = format!(
            "_with_{}",
            default_params.clone().map(|param| param.field.name()).join("_and_")
        );
        let doc_postfix =
            format!(" with `{}`", default_params.map(|param| param.field.name()).join("` and `"));
        generate_builder_method_for_enum_variant_impl(
            enum_def,
            struct_def,
            &variant_ident,
            &params,
            &fn_name,
            &generic_params,
            &where_clause,
            &fn_name_postfix,
            &doc_postfix,
            schema,
            is_boxed,
        )
    });

    params.retain(|param| !param.is_default);
    let mut output2 = generate_builder_method_for_enum_variant_impl(
        enum_def,
        struct_def,
        &variant_ident,
        &params,
        &fn_name,
        &generic_params,
        &where_clause,
        "",
        "",
        schema,
        is_boxed,
    );

    if let Some(output) = output {
        output2.extend(output);
    }

    output2
}

fn generate_builder_method_for_enum_variant_impl(
    enum_def: &EnumDef,
    struct_def: &StructDef,
    variant_ident: &Ident,
    params: &[Param],
    fn_name: &str,
    generic_params: &TokenStream,
    where_clause: &TokenStream,
    fn_name_postfix: &str,
    doc_postfix: &str,
    schema: &Schema,
    is_boxed: bool,
) -> TokenStream {
    let fn_name = format_ident!("{}{}", fn_name, fn_name_postfix);
    let fn_params = params
        .iter()
        .filter(|param| !param.is_comment_node_id && !param.is_node_id)
        .map(|param| &param.fn_param);
    let args = params
        .iter()
        .filter(|param| !param.is_comment_node_id && !param.is_node_id)
        .map(|param| &param.ident);

    let enum_ident = enum_def.ident();
    let enum_ty = enum_def.ty(schema);
    let inner_builder_name = format!("{}{fn_name_postfix}", struct_def.snake_name());
    let inner_builder_name = struct_builder_name(&inner_builder_name, is_boxed);

    // Generate doc comments
    let enum_name = enum_def.name();
    let article_enum = article_for(enum_name);
    let fn_doc1 = format!(" Build {article_enum} [`{enum_name}::{variant_ident}`]{doc_postfix}.");
    let mut fn_docs = quote!( #[doc = #fn_doc1] );
    if is_boxed {
        let variant_type_name = struct_def.name();
        let article_variant = article_for(variant_type_name);
        let fn_doc2 = format!(
            " This node contains {article_variant} [`{variant_type_name}`] that will be stored in the memory arena."
        );
        fn_docs.extend(quote!( #[doc = ""] #[doc = #fn_doc2] ));
    }
    let params_docs = generate_doc_comment_for_params(params);

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
        // We just checked name is not a reserved word
        create_safe_ident(snake_name)
    }
}

/// Get name of enum variant builder method.
fn enum_variant_builder_name(enum_def: &EnumDef, variant: &VariantDef) -> String {
    let enum_name = enum_def.snake_name();

    let variant_name = variant.snake_name();
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

    format!("{enum_name}_{variant_name}")
}

/// Wrap the value of a default field in `Cell::new(...)` or `Some(...)` if necessary.
///
/// Wrap recursively, moving inwards towards the innermost type.
fn wrap_default_field_value(
    value: TokenStream,
    type_def: &TypeDef,
    schema: &Schema,
) -> TokenStream {
    match type_def {
        TypeDef::Cell(cell_def) => {
            let inner_value = wrap_default_field_value(value, cell_def.inner_type(schema), schema);
            quote!( Cell::new(#inner_value) )
        }
        TypeDef::Option(option_def) => {
            let inner_value =
                wrap_default_field_value(value, option_def.inner_type(schema), schema);
            quote!( Some(#inner_value) )
        }
        _ => value,
    }
}

/// Generate doc comment for function params.
fn generate_doc_comment_for_params(params: &[Param]) -> TokenStream {
    if params.is_empty() {
        return quote!();
    }

    let lines =
        params.iter().filter(|param| !param.is_comment_node_id && !param.is_node_id).map(|param| {
            let field = param.field;
            let field_name = field.name();
            let field_comment = if let Some(field_comment) = field.doc_comment.as_deref() {
                format!(" * `{field_name}`: {field_comment}")
            } else if field.name() == "span" {
                " * `span`: The [`Span`] covering this node".to_string()
            } else {
                format!(" * `{field_name}`")
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
