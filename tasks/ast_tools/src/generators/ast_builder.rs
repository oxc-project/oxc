//! Generator for AST builder methods defined directly on AST types.
//!
//! A node is built with `BindingRestElement::new(span, argument, builder)`, where `builder` is
//! anything which implements `GetAstBuilder` (e.g. an `AstBuilder`, or a parser/traversal context).

use std::iter;

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    AST_CRATE_PATH, Codegen, Generator, Result,
    output::{Output, output_path},
    schema::{
        Def, EnumDef, FieldDef, Schema, StructDef, StructOrEnum, TypeDef, TypeId, VariantDef,
    },
    utils::{article_for, create_safe_ident, is_reserved_name},
};

use super::{AttrLocation, AttrPart, AttrPositions, attr_positions, define_generator};

/// Generator for builder methods defined directly on AST types.
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

    /// Generate builder methods on AST types, plus a mapping from old `AstBuilder` method names to
    /// the equivalent new builder methods.
    fn generate_many(&self, schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        let node_id_cell_type_id =
            schema.type_by_name("NodeId").as_struct().unwrap().containers.cell_id.unwrap();

        // Mapping from old `AstBuilder` method name (e.g. `null_literal`) to the equivalent method
        // on the AST type (e.g. `NullLiteral::new`). Used to drive migration to the new builder.
        let mut method_map = vec![];

        let impls = schema
            .structs_and_enums()
            .filter(|&type_def| match type_def {
                StructOrEnum::Struct(struct_def) => {
                    !struct_def.builder.skip && struct_def.visit.has_visitor()
                }
                StructOrEnum::Enum(enum_def) => {
                    !enum_def.builder.skip && enum_def.visit.has_visitor()
                }
            })
            .map(|type_def| {
                generate_builder_impl(type_def, node_id_cell_type_id, &mut method_map, schema)
            })
            .collect::<TokenStream>();

        let output = quote! {
            //! AST node builder methods.

            //!@@line_break
            #![expect(clippy::default_trait_access)]

            ///@@line_break
            use std::cell::Cell;

            ///@@line_break
            use oxc_allocator::{ArenaBox, ArenaVec, GetAllocator, IntoIn};
            use oxc_str::{Ident, Str};
            use oxc_syntax::{scope::ScopeId, symbol::SymbolId, reference::ReferenceId};

            ///@@line_break
            use crate::{ast::*, builder::{AstBuild, GetAstBuilder}};

            #impls
        };

        vec![
            Output::Rust { path: output_path(AST_CRATE_PATH, "ast_builder.rs"), tokens: output },
            generate_method_map_output(&method_map),
            generate_shorten_mappings_output(schema, node_id_cell_type_id),
        ]
    }
}

/// Generate a JSON file mapping old `AstBuilder` method names to the equivalent new builder methods
/// defined on AST types.
///
/// e.g. `"null_literal": "NullLiteral::new"`, `"alloc_null_literal": "NullLiteral::boxed"`,
/// `"statement_expression": "Statement::new_expression_statement"`.
///
/// Consumed by the `tasks/ast_builder_migration` codemod, which migrates Oxc crates and downstream
/// consumers from the old `AstBuilder` to the new builder methods.
fn generate_method_map_output(method_map: &[(String, String)]) -> Output {
    let mut code = String::from("{\n");
    for (index, (old_name, new_path)) in method_map.iter().enumerate() {
        let comma = if index + 1 < method_map.len() { "," } else { "" };
        #[expect(clippy::format_push_string)]
        code.push_str(&format!("  \"{old_name}\": \"{new_path}\"{comma}\n"));
    }
    code.push_str("}\n");

    Output::Raw { path: "tasks/ast_builder_migration/generated/mappings.json".to_string(), code }
}

/// Generate a JSON file of *structural* mappings that drive the codemod's "shortening" pass, which
/// collapses verbose new-builder call shapes into their shorthand equivalents:
///
/// * **box-collapse**: `ArenaBox::new_in(T::new(args), x)` -> `T::boxed(args)`
/// * **variant-wrap**: `E::Variant(T::boxed(args))` -> `E::new_variant(args)`
/// * **inherited-from**: `Outer::from(Inner::new_x(args))` -> `Outer::new_x(args)`
/// * **vec-arg**: `T::new(.., ArenaVec::new_in(x), ..)` -> `T::new(.., [], ..)`
///
/// Emitting these from the same code that generates the builder methods means the exact method
/// names (incl. `_with_*` default-field forms) are captured correctly.
///
/// Consumed by `generate_rules.mts`, alongside `mappings.json`.
fn generate_shorten_mappings_output(schema: &Schema, node_id_cell_type_id: TypeId) -> Output {
    // `[T::new path, T::boxed path]` for each boxable struct (+ `_with_*` forms).
    let mut boxed: Vec<(String, String)> = vec![];
    // `[enum, variant ident, inner builder path, ctor path]` for each struct-payload variant
    // (own + inherited) of each enum (+ `_with_*` forms).
    let mut variant_wrap: Vec<(String, String, String, String)> = vec![];
    // `[outer enum, inherited enum, ctor method name]` for each constructor reachable via a
    // directly-inherited enum (+ `_with_*` forms).
    let mut inherited_from: Vec<(String, String, String)> = vec![];
    // Path of each builder method which has an `ArenaVec` param (+ `_with_*` forms).
    let mut vec_arg: Vec<String> = vec![];

    for type_def in schema.structs_and_enums() {
        match type_def {
            StructOrEnum::Struct(struct_def) => {
                // Skip types with no builder.
                if struct_def.builder.skip || !struct_def.visit.has_visitor() {
                    continue;
                }
                let name = struct_def.name();
                let postfix = default_field_postfix(struct_def, node_id_cell_type_id, schema);
                // Non-boxable structs have no `T::boxed`.
                let boxed_method = struct_def.containers.box_id.is_some().then_some("boxed");

                if boxed_method.is_some() {
                    boxed.push((format!("{name}::new"), format!("{name}::boxed")));
                    if let Some(postfix) = &postfix {
                        boxed.push((
                            format!("{name}::new{postfix}"),
                            format!("{name}::boxed{postfix}"),
                        ));
                    }
                }

                // `vec-arg`: every builder method of a struct which has an `ArenaVec` field.
                if has_vec_param(struct_def, schema) {
                    for method in iter::once("new").chain(boxed_method) {
                        vec_arg.push(format!("{name}::{method}"));
                        if let Some(postfix) = &postfix {
                            vec_arg.push(format!("{name}::{method}{postfix}"));
                        }
                    }
                }
            }
            StructOrEnum::Enum(enum_def) => {
                if enum_def.builder.skip || !enum_def.visit.has_visitor() {
                    continue;
                }
                let enum_name = enum_def.name();

                // `variant-wrap`: every variant (own + inherited) whose payload is a struct.
                for variant in enum_def.all_variants(schema) {
                    let Some((inner, is_boxed)) = struct_variant_payload(variant, schema) else {
                        continue;
                    };
                    let inner_name = inner.name();
                    let inner_method = if is_boxed { "boxed" } else { "new" };
                    let variant_ident = variant.ident().to_string();
                    let ctor = format!("new_{}", variant.snake_name());
                    let postfix = default_field_postfix(inner, node_id_cell_type_id, schema);
                    variant_wrap.push((
                        enum_name.to_string(),
                        variant_ident.clone(),
                        format!("{inner_name}::{inner_method}"),
                        format!("{enum_name}::{ctor}"),
                    ));
                    if let Some(postfix) = &postfix {
                        variant_wrap.push((
                            enum_name.to_string(),
                            variant_ident,
                            format!("{inner_name}::{inner_method}{postfix}"),
                            format!("{enum_name}::{ctor}{postfix}"),
                        ));
                    }

                    // `vec-arg`: the variant's ctor forwards to the inner struct's builder,
                    // so it has the same `ArenaVec` params.
                    if has_vec_param(inner, schema) {
                        vec_arg.push(format!("{enum_name}::{ctor}"));
                        if let Some(postfix) = &postfix {
                            vec_arg.push(format!("{enum_name}::{ctor}{postfix}"));
                        }
                    }
                }

                // `inherited-from`: every constructor reachable through an inherited enum. Walk the
                // *transitive* closure - inheritance chains (e.g. `AssignmentTarget` ->
                // `SimpleAssignmentTarget` -> `MemberExpression`) generate `From` impls and `new_*`
                // ctors for the whole chain, so `Outer::from(Grandparent::new_x(..))` is valid too.
                for inner_enum in enum_def.all_inherits(schema) {
                    let inner_enum_name = inner_enum.name();
                    for variant in inner_enum.all_variants(schema) {
                        let Some((inner, _)) = struct_variant_payload(variant, schema) else {
                            continue;
                        };
                        let ctor = format!("new_{}", variant.snake_name());
                        inherited_from.push((
                            enum_name.to_string(),
                            inner_enum_name.to_string(),
                            ctor.clone(),
                        ));
                        if let Some(postfix) =
                            default_field_postfix(inner, node_id_cell_type_id, schema)
                        {
                            inherited_from.push((
                                enum_name.to_string(),
                                inner_enum_name.to_string(),
                                format!("{ctor}{postfix}"),
                            ));
                        }
                    }
                }
            }
        }
    }

    // Emit one entry per line (compact arrays) for a readable, reviewable diff.
    let mut code = String::from("{\n");
    push_json_section(&mut code, "boxed", &boxed, false);
    push_json_section(&mut code, "variantWrap", &variant_wrap, false);
    push_json_section(&mut code, "inheritedFrom", &inherited_from, false);
    push_json_section(&mut code, "vecArg", &vec_arg, true);
    code.push_str("}\n");

    Output::Raw {
        path: "tasks/ast_builder_migration/generated/shorten_mappings.json".to_string(),
        code,
    }
}

/// Append a JSON array section (`"key": [ ... ]`), one entry per line. `last` omits the trailing
/// comma after the closing `]`.
fn push_json_section<T: serde::Serialize>(code: &mut String, key: &str, entries: &[T], last: bool) {
    #[expect(clippy::format_push_string)]
    code.push_str(&format!("  \"{key}\": [\n"));
    for (index, entry) in entries.iter().enumerate() {
        let comma = if index + 1 < entries.len() { "," } else { "" };
        let line = serde_json::to_string(entry).unwrap();
        #[expect(clippy::format_push_string)]
        code.push_str(&format!("    {line}{comma}\n"));
    }
    code.push_str(if last { "  ]\n" } else { "  ],\n" });
}

/// Get the `_with_<default fields>` postfix for a struct's builder method, if it has default fields.
///
/// Mirrors the postfix logic in `generate_builder_methods_for_struct` /
/// `generate_builder_method_for_enum_variant`.
fn default_field_postfix(
    struct_def: &StructDef,
    node_id_cell_type_id: TypeId,
    schema: &Schema,
) -> Option<String> {
    // `has_lifetime` only affects the generic params, which are not used here.
    let has_lifetime = struct_def.has_lifetime(schema);
    let (params, _, _, has_default_fields) =
        get_struct_params(struct_def, node_id_cell_type_id, has_lifetime, schema);
    if !has_default_fields {
        return None;
    }
    // Exclude `node_id` (always set by the builder) - matches the generator.
    let default_params = params.iter().filter(|param| param.is_default && !param.is_node_id);
    Some(format!("_with_{}", default_params.map(|param| param.field.name()).join("_and_")))
}

/// Get whether any of a struct's builder method params is an `ArenaVec`.
///
/// Such params are generic over `IntoIn<'a, ArenaVec<'a, T>>`, so they accept an array literal in
/// place of an `ArenaVec`. This drives the codemod's `vec-arg` shortening rules.
///
/// `Option<Vec<_>>` fields do *not* qualify. They get no `IntoIn` generic (see `get_struct_params`),
/// so still require a real `ArenaVec`.
fn has_vec_param(struct_def: &StructDef, schema: &Schema) -> bool {
    struct_def.fields.iter().any(|field| matches!(field.type_def(schema), TypeDef::Vec(_)))
}

/// Resolve an enum variant's payload to its inner struct, if it is one (unwrapping `Box`).
///
/// Returns `(struct, is_boxed)`, or `None` for fieldless or non-struct variants.
fn struct_variant_payload<'s>(
    variant: &VariantDef,
    schema: &'s Schema,
) -> Option<(&'s StructDef, bool)> {
    let mut variant_type = variant.field_type(schema)?;
    let mut is_boxed = false;
    if let TypeDef::Box(box_def) = variant_type {
        variant_type = box_def.inner_type(schema);
        is_boxed = true;
    }
    match variant_type {
        TypeDef::Struct(struct_def) => Some((struct_def, is_boxed)),
        _ => None,
    }
}

/// Param for a builder method.
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
    /// `true` if is `NodeId` field
    is_node_id: bool,
    /// * `None` if param is not generic.
    /// * `Some(GenericType::Into)` if is generic and uses `Into`
    ///   e.g. `name: S1 where S1: Into<Str<'a>>`.
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

/// Generate `impl` block containing builder methods for a type.
fn generate_builder_impl(
    type_def: StructOrEnum<'_>,
    node_id_cell_type_id: TypeId,
    method_map: &mut Vec<(String, String)>,
    schema: &Schema,
) -> TokenStream {
    let (methods, ty, lifetime) = match type_def {
        StructOrEnum::Struct(struct_def) => (
            generate_builder_methods_for_struct(
                struct_def,
                node_id_cell_type_id,
                method_map,
                schema,
            ),
            struct_def.ty(schema),
            struct_def.lifetime(schema),
        ),
        StructOrEnum::Enum(enum_def) => (
            generate_builder_methods_for_enum(enum_def, node_id_cell_type_id, method_map, schema),
            enum_def.ty(schema),
            enum_def.lifetime(schema),
        ),
    };

    quote! {
        ///@@line_break
        impl #lifetime #ty {
            #methods
        }
    }
}

/// Generate builder methods for a struct.
///
/// Generates two builder methods:
/// 1. To build an owned type, named `new`.
/// 2. To build a boxed type, named `boxed`.
fn generate_builder_methods_for_struct(
    struct_def: &StructDef,
    node_id_cell_type_id: TypeId,
    method_map: &mut Vec<(String, String)>,
    schema: &Schema,
) -> TokenStream {
    let has_lifetime = struct_def.has_lifetime(schema);
    let (mut params, generic_params, where_clause, has_default_fields) =
        get_struct_params(struct_def, node_id_cell_type_id, has_lifetime, schema);
    let (fn_params, fields) = get_struct_fn_params_and_fields(&params, true, schema);

    let (fn_name_postfix, doc_postfix) = if has_default_fields {
        // Exclude `node_id` from the list of default params (it's always assigned by the builder)
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

    // Record mappings from old method names to new methods (e.g. `null_literal` -> `NullLiteral::new`).
    // The boxed (`alloc_*`) method only exists when `Box<T>` appears in the AST.
    let struct_name = struct_def.name();
    let snake_name = struct_def.snake_name();
    let has_box = struct_def.containers.box_id.is_some();
    push_struct_method_map(method_map, struct_name, &snake_name, "", has_box);
    if has_default_fields {
        push_struct_method_map(method_map, struct_name, &snake_name, &fn_name_postfix, has_box);
    }

    // Generate builder methods including all fields (inc default fields)
    let output = generate_builder_methods_for_struct_impl(
        struct_def,
        &params,
        &fn_params,
        &fields,
        &generic_params,
        &where_clause,
        &fn_name_postfix,
        &doc_postfix,
    );

    if !has_default_fields {
        return output;
    }

    // Generate builder methods excluding default fields
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
    );

    output2.extend(output);

    output2
}

/// Build a pair of builder methods (`new` and `boxed`) for a struct.
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
) -> TokenStream {
    let struct_ident = struct_def.ident();

    let args = params.iter().filter(|param| !param.is_node_id).map(|param| &param.ident);

    let new_fn_name = format_ident!("new{fn_name_postfix}");

    // Only generate a `boxed` method if `Box<T>` exists in AST
    let boxed_fn_name =
        struct_def.containers.box_id.is_some().then(|| format_ident!("boxed{fn_name_postfix}"));

    // Generate main builder method
    let struct_name = struct_def.name();
    let article = article_for(struct_name);
    let fn_doc1 = format!(" Build {article} [`{struct_name}`]{doc_postfix}.");
    let mut fn_docs = quote!( #[doc = #fn_doc1] );
    if let Some(boxed_fn_name) = &boxed_fn_name {
        let fn_doc2 = format!(" use [`{struct_name}::{boxed_fn_name}`] instead.");
        fn_docs.extend(quote! {
            #[doc = ""]
            #[doc = " If you want the built node to be allocated in the memory arena,"]
            #[doc = #fn_doc2]
        });
    }

    let params_docs = generate_doc_comment_for_params(params);

    let new_method = quote! {
        ///@@line_break
        #fn_docs
        #params_docs
        #[inline]
        pub fn #new_fn_name #generic_params (#fn_params, builder: &B) -> Self #where_clause {
            let builder = builder.builder();
            #struct_ident { #fields }
        }
    };

    let Some(boxed_fn_name) = boxed_fn_name else { return new_method };

    // Generate `boxed` builder method
    let boxed_doc1 = format!(
        " Build {article} [`{struct_name}`]{doc_postfix}, and store it in the memory arena."
    );
    let boxed_doc2 = format!(
        " If you want a stack-allocated node, use [`{struct_name}::{new_fn_name}`] instead."
    );

    quote! {
        #new_method

        ///@@line_break
        #[doc = #boxed_doc1]
        #[doc = ""]
        #[doc = " Returns a [`Box`](ArenaBox) containing the newly-allocated node."]
        #[doc = #boxed_doc2]
        #params_docs
        #[inline]
        pub fn #boxed_fn_name #generic_params (#fn_params, builder: &B) -> ArenaBox<'a, Self> #where_clause {
            ArenaBox::new_in(Self::#new_fn_name(#(#args),*, builder), builder.builder())
        }
    }
}

/// Get params for builder methods for a struct.
///
/// Also generates the generic params and `where` clause for the methods.
/// These always include the builder generic `B` (and `'a` when `has_lifetime` is `false`),
/// plus any `Into` / `IntoIn` generics derived from the struct's fields.
///
/// ```
/// //        ↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓ generic params
/// pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, function: T1, builder: &B) -> Self
///     where T1: IntoIn<'a, Box<'a, Function<'a>>>
/// //  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ where clause
/// ```
fn get_struct_params<'s>(
    struct_def: &'s StructDef,
    node_id_cell_type_id: TypeId,
    has_lifetime: bool,
    schema: &'s Schema,
) -> (
    Vec<Param<'s>>, // Params
    TokenStream,    // Generic params
    TokenStream,    // `where` clause
    bool,           // Has default fields
) {
    let mut generic_count = 0u32;
    let mut str_generic_count = 0u32;
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
                TypeDef::Primitive(primitive_def)
                    if matches!(primitive_def.name(), "Str" | "Ident") =>
                {
                    str_generic_count += 1;
                    Some((format_ident!("S{str_generic_count}"), GenericType::Into))
                }
                TypeDef::Box(_) | TypeDef::Vec(_) => {
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

            let is_node_id = field.type_id == node_id_cell_type_id;
            Param { field, ident: field_ident, fn_param, is_default, is_node_id, generic_type }
        })
        .collect();

    let lifetime = if has_lifetime { quote!() } else { quote!( 'a, ) };
    let generic_idents = generics.iter().map(|(generic_ident, _)| generic_ident);
    let generic_params = quote!( <#lifetime B: GetAstBuilder<'a> #(, #generic_idents)*> );

    // `Into`/`IntoIn` bounds for field generics go in a `where` clause. Most methods have no field
    // generics, so have no `where` clause.
    let where_clause = if generics.is_empty() {
        quote!()
    } else {
        let where_clause_parts = generics.iter().map(|(_, where_clause_part)| where_clause_part);
        quote!( where #(#where_clause_parts),* )
    };

    (params, generic_params, where_clause, has_default_fields)
}

/// Get function params and fields for a struct builder method.
///
/// Omit default fields from function params if `include_default_fields == false`.
///
/// The generated field values reference a local `builder` binding (`let builder = builder.builder();`)
/// for the allocator and node ID.
///
/// ```
/// //                               ↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓ function params
/// pub fn new<B: GetAstBuilder<'a>>(span: Span, bar: Bar<'a>, builder: &B) -> Self {
///     let builder = builder.builder();
///     Foo { node_id: Cell::new(builder.node_id()), span, bar }
/// //        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ fields
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

        // Special case: `NodeId` is always assigned by the builder and is never a parameter
        if param.is_node_id {
            fields.push(quote!( #param_ident: Cell::new(builder.node_id()) ));
            return None;
        }

        if param.is_default {
            if include_default_fields {
                // Builder methods which take default fields receive the innermost type as param.
                // So wrap the param's value in `Cell::new(...)`, or `Some(...)` if necessary.
                let field_type = param.field.type_def(schema);
                let value = wrap_default_field_value(quote!( #param_ident ), field_type, schema);

                fields.push(quote!( #param_ident: #value ));
                return Some(&param.fn_param);
            }

            fields.push(quote!( #param_ident: Default::default() ));
            return None;
        }

        let field = match param.generic_type {
            Some(GenericType::Into) => quote!( #param_ident: #param_ident.into() ),
            Some(GenericType::IntoIn) => {
                quote!( #param_ident: #param_ident.into_in(builder.allocator()) )
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
/// Generates a builder method for every variant of the enum, including inherited variants.
/// Each method is named after the variant (not the variant's type) with a `new_` prefix,
/// e.g. `Expression::new_identifier`, not `Expression::new_identifier_reference`.
fn generate_builder_methods_for_enum(
    enum_def: &EnumDef,
    node_id_cell_type_id: TypeId,
    method_map: &mut Vec<(String, String)>,
    schema: &Schema,
) -> TokenStream {
    // `all_variants` yields the enum's own variants first, then inherited ones. The old `AstBuilder`
    // only had methods for own variants, so only those get an entry in the migration mapping.
    let own_variant_count = enum_def.variants.len();
    enum_def
        .all_variants(schema)
        .enumerate()
        .map(|(index, variant)| {
            let method_map = (index < own_variant_count).then_some(&mut *method_map);
            generate_builder_method_for_enum_variant(
                enum_def,
                variant,
                node_id_cell_type_id,
                method_map,
                schema,
            )
        })
        .collect()
}

/// Generate builder method for an enum variant.
fn generate_builder_method_for_enum_variant(
    enum_def: &EnumDef,
    variant: &VariantDef,
    node_id_cell_type_id: TypeId,
    mut method_map: Option<&mut Vec<(String, String)>>,
    schema: &Schema,
) -> TokenStream {
    let mut variant_type = variant.field_type(schema).unwrap();
    let mut is_boxed = false;
    if let TypeDef::Box(box_def) = variant_type {
        variant_type = box_def.inner_type(schema);
        is_boxed = true;
    }
    let TypeDef::Struct(struct_def) = variant_type else { panic!("Unsupported!") };

    let has_lifetime = enum_def.has_lifetime(schema);
    let (mut params, generic_params, where_clause, has_default_fields) =
        get_struct_params(struct_def, node_id_cell_type_id, has_lifetime, schema);

    let method_name = format!("new_{}", variant.snake_name());
    let variant_ident = variant.ident();

    // Record mappings from old method names to new methods (e.g. `statement_expression` ->
    // `Statement::new_expression_statement`). The old name is de-duplicated by
    // `enum_variant_builder_name`, whereas the new name is always `new_<variant>`.
    let enum_name = enum_def.name();
    let old_fn_name = enum_variant_builder_name(enum_def, variant);
    if let Some(method_map) = method_map.as_deref_mut() {
        method_map.push((old_fn_name.clone(), format!("{enum_name}::{method_name}")));
    }

    let output = has_default_fields.then(|| {
        // Exclude `node_id` from the list of default params (it's always assigned by the builder)
        let default_params = params.iter().filter(|param| param.is_default && !param.is_node_id);
        let fn_name_postfix = format!(
            "_with_{}",
            default_params.clone().map(|param| param.field.name()).join("_and_")
        );
        let doc_postfix =
            format!(" with `{}`", default_params.map(|param| param.field.name()).join("` and `"));
        if let Some(method_map) = method_map {
            method_map.push((
                format!("{old_fn_name}{fn_name_postfix}"),
                format!("{enum_name}::{method_name}{fn_name_postfix}"),
            ));
        }
        generate_builder_method_for_enum_variant_impl(
            enum_def,
            struct_def,
            &variant_ident,
            &params,
            &method_name,
            &generic_params,
            &where_clause,
            &fn_name_postfix,
            &doc_postfix,
            is_boxed,
        )
    });

    params.retain(|param| !param.is_default);
    let mut output2 = generate_builder_method_for_enum_variant_impl(
        enum_def,
        struct_def,
        &variant_ident,
        &params,
        &method_name,
        &generic_params,
        &where_clause,
        "",
        "",
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
    method_name: &str,
    generic_params: &TokenStream,
    where_clause: &TokenStream,
    fn_name_postfix: &str,
    doc_postfix: &str,
    is_boxed: bool,
) -> TokenStream {
    let fn_name = format_ident!("{}{}", method_name, fn_name_postfix);
    let fn_params = params.iter().filter(|param| !param.is_node_id).map(|param| &param.fn_param);
    let args = params.iter().filter(|param| !param.is_node_id).map(|param| &param.ident);

    let struct_ident = struct_def.ident();
    let inner_fn_name =
        format_ident!("{}{fn_name_postfix}", if is_boxed { "boxed" } else { "new" });

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
        pub fn #fn_name #generic_params (#(#fn_params),*, builder: &B) -> Self #where_clause {
            Self::#variant_ident(#struct_ident::#inner_fn_name(#(#args),*, builder))
        }
    }
}

/// Record mappings for a struct's builder methods (with the given postfix) into `method_map`.
///
/// Maps old `AstBuilder` method names to the equivalent new methods on the AST type,
/// e.g. `null_literal` -> `NullLiteral::new`, `alloc_null_literal` -> `NullLiteral::boxed`.
/// The boxed (`alloc_*` / `::boxed`) method only exists when `has_box` is `true`.
fn push_struct_method_map(
    method_map: &mut Vec<(String, String)>,
    struct_name: &str,
    snake_name: &str,
    postfix: &str,
    has_box: bool,
) {
    let base = format!("{snake_name}{postfix}");
    method_map.push((
        struct_builder_name(&base, false).to_string(),
        format!("{struct_name}::new{postfix}"),
    ));
    if has_box {
        method_map.push((
            struct_builder_name(&base, true).to_string(),
            format!("{struct_name}::boxed{postfix}"),
        ));
    }
}

/// Get name of the *old* `AstBuilder` method for a struct.
///
/// If `does_alloc == true`, prepends `alloc_` to start of name.
///
/// The old builder is gone, but its method names are still needed to generate the migration
/// mappings consumed by `tasks/ast_builder_migration`.
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

/// Get name of the *old* `AstBuilder` method for an enum variant.
///
/// As with [`struct_builder_name`], this exists only to generate the migration mappings.
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

    let lines = params.iter().filter(|param| !param.is_node_id).map(|param| {
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
