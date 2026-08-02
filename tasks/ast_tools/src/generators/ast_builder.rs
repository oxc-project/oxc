//! Generator for AST builder methods defined directly on AST types.
//!
//! Generates 2 files:
//!
//! 1. `ast_builder.rs` - `new` and `boxed` methods, which build a node from all of its fields
//!    at once e.g. `BindingRestElement::new(span, argument, builder)`.
//! 2. `builders.rs` - builder types, which construct a node in place in the memory arena,
//!    one field at a time e.g. `BindingRestElement::build(builder).span(span).argument(argument).finish()`.
//!
//! In both, `builder` is anything which implements `GetAstBuilder`
//! (e.g. an `AstBuilder`, or a parser/traversal context).

use std::{iter, ops::Range};

use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rustc_hash::FxHashSet;
use syn::Ident;

use crate::{
    AST_CRATE_PATH, Codegen, Generator, Result,
    output::{Output, output_path},
    schema::{
        Def, EnumDef, FieldDef, Schema, StructDef, StructOrEnum, TypeDef, TypeId, VariantDef,
        extensions::layout::GetLayout,
    },
    utils::{article_for, create_ident, create_safe_ident, number_lit},
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

    /// Generate builder methods on AST types.
    fn generate_many(&self, schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        let type_defs = schema
            .structs_and_enums()
            .filter(|&type_def| has_builder(type_def))
            .collect::<Vec<_>>();

        vec![
            generate_ast_builder_output(&type_defs, schema),
            generate_builders_output(&type_defs, schema),
        ]
    }
}

/// Generate `ast_builder.rs` output, containing `new` / `boxed` methods.
fn generate_ast_builder_output(type_defs: &[StructOrEnum], schema: &Schema) -> Output {
    let node_id_cell_type_id = node_id_cell_type_id(schema);

    let impls = type_defs
        .iter()
        .map(|&type_def| generate_builder_impl(type_def, node_id_cell_type_id, schema))
        .collect::<TokenStream>();

    let output = quote! {
        //! AST node builder methods.
        //!
        //! Each method is generic over [`GetAstBuilder`], so it can be called with a builder
        //! directly, or with a type which holds one (e.g. parser or traverse context).
        //!
        //! Before forwarding the builder to another build method, methods first call `builder.builder()`
        //! to obtain the concrete [`AstBuild`]er. This way, everything below the outermost call is
        //! monomorphized over the [`AstBuild`] type (of which there are few), rather than over every
        //! [`GetAstBuilder`] type the method is called with (of which there may be many).

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

    Output::Rust { path: output_path(AST_CRATE_PATH, "ast_builder.rs"), tokens: output }
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
    ///   e.g. `name: impl Into<Str<'a>>`.
    /// * `Some(GenericType::IntoIn)` if is generic and uses `IntoIn`
    ///   e.g. `params: impl IntoIn<'a, ArenaVec<'a, TSTypeParameter<'a>>>`.
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
    schema: &Schema,
) -> TokenStream {
    let (methods, ty, lifetime) = match type_def {
        StructOrEnum::Struct(struct_def) => (
            generate_builder_methods_for_struct(struct_def, node_id_cell_type_id, schema),
            struct_def.ty(schema),
            struct_def.lifetime(schema),
        ),
        StructOrEnum::Enum(enum_def) => (
            generate_builder_methods_for_enum(enum_def, node_id_cell_type_id, schema),
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
    schema: &Schema,
) -> TokenStream {
    let (mut params, has_default_fields) =
        get_struct_params(struct_def, node_id_cell_type_id, schema);
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

    // Generate builder methods including all fields (inc default fields)
    let output = generate_builder_methods_for_struct_impl(
        struct_def,
        &params,
        &fn_params,
        &fields,
        &fn_name_postfix,
        &doc_postfix,
    );

    if !has_default_fields {
        return output;
    }

    // Generate builder methods excluding default fields
    let (fn_params, fields) = get_struct_fn_params_and_fields(&params, false, schema);
    params.retain(|param| !param.is_default);
    let mut output2 =
        generate_builder_methods_for_struct_impl(struct_def, &params, &fn_params, &fields, "", "");

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
    fn_name_postfix: &str,
    doc_postfix: &str,
) -> TokenStream {
    let struct_ident = struct_def.ident();

    let args = params.iter().filter(|param| !param.is_node_id).map(|param| &param.ident);

    let new_fn_name = format_ident!("new{fn_name_postfix}");
    // Function only needs lifetime param for `impl GetAstBuilder<'a>` if the struct doesn't have a lifetime itself
    let lifetime_param = if struct_def.has_lifetime { quote!() } else { quote!( <'a> ) };

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
            ///
            /// If you want the built node to be allocated in the memory arena,
            #[doc = #fn_doc2]
        });
    }

    let params_docs = generate_doc_comment_for_params(params);
    let unused_builder_attr =
        (!fields.to_string().contains("builder")).then(|| quote!(#[expect(unused_variables)]));

    let new_method = quote! {
        ///@@line_break
        #fn_docs
        #params_docs
        #unused_builder_attr
        #[inline]
        pub fn #new_fn_name #lifetime_param (#fn_params, builder: &impl GetAstBuilder<'a>) -> Self {
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
        ///
        /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
        #[doc = #boxed_doc2]
        #params_docs
        #[inline]
        pub fn #boxed_fn_name #lifetime_param (
            #fn_params, builder: &impl GetAstBuilder<'a>
        ) -> ArenaBox<'a, Self> {
            let builder = builder.builder();
            // Allocate via `&Allocator` (not `builder`), so `ArenaBox::new_in` shares a
            // monomorphization with every other `&Allocator`-based allocation
            ArenaBox::new_in(Self::#new_fn_name(#(#args),*, builder), &builder.allocator())
        }
    }
}

/// Get params for builder methods for a struct.
fn get_struct_params<'s>(
    struct_def: &'s StructDef,
    node_id_cell_type_id: TypeId,
    schema: &'s Schema,
) -> (
    Vec<Param<'s>>, // Params
    bool,           // Has default fields
) {
    let mut has_default_fields = false;

    let params = struct_def
        .fields
        .iter()
        .map(|field| {
            let type_def = field.type_def(schema);
            let ty = type_def.ty(schema);

            // A field is default if the field is marked `#[builder(default)]`,
            // or its innermost type is marked `#[builder(default)]`
            let is_default = is_default_field(field, schema);
            if is_default {
                has_default_fields = true;
            }

            let generic_type = match type_def {
                TypeDef::Primitive(primitive_def)
                    if matches!(primitive_def.name(), "Str" | "Ident") =>
                {
                    Some(GenericType::Into)
                }
                TypeDef::Vec(_) => Some(GenericType::IntoIn),
                _ => None,
            };

            let (fn_param_ty, generic_type) = if is_default {
                assert!(generic_type.is_none());
                let ty = type_def.innermost_type(schema).ty(schema);
                (ty, None)
            } else if let Some(generic_type) = generic_type {
                // Generics are expressed as `impl Trait` params, so they can't be turbofished,
                // but that keeps the method signatures much shorter
                let generic_ty = match generic_type {
                    GenericType::Into => quote!( impl Into<#ty> ),
                    GenericType::IntoIn => quote!( impl IntoIn<'a, #ty> ),
                };
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

    (params, has_default_fields)
}

/// Get function params and fields for a struct builder method.
///
/// Omit default fields from function params if `include_default_fields == false`.
///
/// The generated field values reference a local `builder` binding (`let builder = builder.builder();`)
/// for the allocator and node ID.
///
/// ```
/// //          ↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓ function params
/// pub fn new(span: Span, bar: Bar<'a>, builder: &impl GetAstBuilder<'a>) -> Self {
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
    schema: &Schema,
) -> TokenStream {
    enum_def
        .all_variants(schema)
        .map(|variant| {
            generate_builder_method_for_enum_variant(
                enum_def,
                variant,
                node_id_cell_type_id,
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
    schema: &Schema,
) -> TokenStream {
    let mut variant_type = variant.field_type(schema).unwrap();
    let mut is_boxed = false;
    if let TypeDef::Box(box_def) = variant_type {
        variant_type = box_def.inner_type(schema);
        is_boxed = true;
    }
    let TypeDef::Struct(struct_def) = variant_type else { panic!("Unsupported!") };

    let (mut params, has_default_fields) =
        get_struct_params(struct_def, node_id_cell_type_id, schema);

    let method_name = format!("new_{}", variant.snake_name());
    let variant_ident = variant.ident();

    let output = has_default_fields.then(|| {
        // Exclude `node_id` from the list of default params (it's always assigned by the builder)
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
            &method_name,
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
    // Function only needs lifetime param for `impl GetAstBuilder<'a>` if the enum doesn't have a lifetime itself
    let lifetime_param = if enum_def.has_lifetime { quote!() } else { quote!( <'a> ) };

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
        fn_docs.extend(quote! {
            ///
            #[doc = #fn_doc2]
        });
    }
    let params_docs = generate_doc_comment_for_params(params);

    quote! {
        ///@@line_break
        #fn_docs
        #params_docs
        #[inline]
        pub fn #fn_name #lifetime_param (
            #(#fn_params),*, builder: &impl GetAstBuilder<'a>
        ) -> Self {
            Self::#variant_ident(#struct_ident::#inner_fn_name(#(#args),*, builder.builder()))
        }
    }
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

/// Generate `builders.rs` output, containing builder types.
fn generate_builders_output(type_defs: &[StructOrEnum<'_>], schema: &Schema) -> Output {
    let node_id_cell_type_id = node_id_cell_type_id(schema);
    let span_type_id = schema.type_names["Span"];

    let mut builders = TokenStream::new();
    let mut builder_names = vec![];

    // Collected separately from the builders, and emitted after all of them,
    // so that the traits don't break up the run of builder types
    let mut enum_slot_traits = TokenStream::new();
    let mut enum_slot_trait_names = vec![];

    // Every field state used by any builder. Field names repeat across the AST, so the marker
    // trait generated for each of these names is shared between builders.
    let mut state_names = FxHashSet::default();

    // Number of field states the largest builder has. `FieldsState` is generated with that
    // many slots, and shared by every builder - the ones with fewer just leave the rest
    // permanently unset.
    let mut max_state_count = 0;

    for &type_def in type_defs {
        // Only structs get a builder type. An enum gets `Slot` conversions instead - one per
        // variant, narrowing a slot for the enum to a slot for that variant's payload.
        match type_def {
            StructOrEnum::Struct(struct_def) => {
                let (builder, builder_state_names) =
                    generate_node_builder(struct_def, node_id_cell_type_id, span_type_id, schema);
                builders.extend(builder);
                builder_names.push(builder_type_ident(struct_def.name()));
                max_state_count = max_state_count.max(builder_state_names.len());
                state_names.extend(builder_state_names);
            }
            // A fieldless enum is a `u8`, so a field holding one is written with its setter -
            // there is nothing to narrow a slot to, and so no trait
            StructOrEnum::Enum(enum_def) if !enum_def.is_fieldless() => {
                enum_slot_traits.extend(generate_enum_slot_trait(enum_def, schema));
                enum_slot_trait_names.push(enum_slot_trait_ident(enum_def.name()));
            }
            StructOrEnum::Enum(_) => {}
        }
    }

    let fields_state_trait = generate_fields_state_trait(max_state_count);
    let marker_traits = generate_marker_traits(&state_names);

    let output = quote! {
        //! Builders which construct AST nodes in place in the memory arena.
        //!
        //! One builder per AST type. See [`crate::builder::builders`], which these are re-exported from,
        //! for what they are, how to use them, and the types they are built on.

        //!@@line_break
        #![expect(
            private_bounds,
            unused_must_use,
            clippy::default_trait_access,
            clippy::inline_always,
            clippy::semicolon_if_nothing_returned,
            clippy::undocumented_unsafe_blocks
        )]

        ///@@line_break
        use std::{cell::Cell, marker::PhantomData, mem::MaybeUninit};

        ///@@line_break
        use oxc_allocator::{ArenaBox, ArenaVec, GetAllocator, OwnedSlot, Slot, SlotFilled};
        use oxc_str::{Ident, Str};
        use oxc_syntax::{scope::ScopeId, symbol::SymbolId, reference::ReferenceId};

        ///@@line_break
        use crate::{
            ast::*,
            builder::{
                AstBuild, GetAstBuilder,
                builders::{
                    BuilderTarget, EnumSlot, FieldState, No, Set, SlotBuild, init_node_id,
                },
            },
        };

        ///@@line_break
        ///@ --------------------------------------------------------------------------------
        ///@ Trait recording which fields of a builder have been set.
        ///@ --------------------------------------------------------------------------------

        #fields_state_trait

        ///@@line_break
        ///@ --------------------------------------------------------------------------------
        ///@ Marker traits that `Set` alone implements.
        ///@ One per field name, so that a missing field can name itself in the error.
        ///@ --------------------------------------------------------------------------------

        ///@@line_break
        #[doc(hidden)]
        pub mod markers {
            use super::Set;

            ///@@line_break
            /// Private module, so `IsSet` cannot be implemented outside of it,
            /// and therefore neither can the `*IsSet` traits which have it as a supertrait.
            /// Specifically, this prevents other code in this crate doing `impl AbstractIsSet for No {}`.
            mod private {
                /// Sealing trait, implemented only for [`Set`](super::Set).
                pub trait IsSet {}
                impl IsSet for super::Set {}
            }
            use private::IsSet;

            #marker_traits
        }
        use markers::*;

        ///@@line_break
        ///@ --------------------------------------------------------------------------------
        ///@ AST struct builders.
        ///@ --------------------------------------------------------------------------------

        #builders

        ///@@line_break
        /// Every builder type, and the trait which records their field states,
        /// for importing them as a group.
        #[expect(clippy::module_inception)]
        pub mod builders {
            pub use super::{#(#builder_names),*};
        }

        ///@@line_break
        ///@ --------------------------------------------------------------------------------
        ///@ Traits which narrow a `Slot` for an enum to a `Slot` for one variant's payload.
        ///@ One per enum with fieldful variants.
        ///@ --------------------------------------------------------------------------------

        #enum_slot_traits

        ///@@line_break
        /// Every trait needed to build AST nodes in [`Slot`]s, for importing as a group.
        ///
        /// [`SlotBuild`] provides `Slot::build`.
        /// Each enum's `into_<variant>` methods come from its own trait.
        /// To make all these methods available, import:
        ///
        /// ```ignore
        /// use oxc_ast::builder::builders::traits::*;
        /// ```
        pub mod traits {
            pub use crate::builder::builders::SlotBuild;
            ///@@line_break
            pub use super::{#(#enum_slot_trait_names),*};
        }
    };

    Output::Rust { path: output_path(AST_CRATE_PATH, "builders.rs"), tokens: output }
}

/// Generate the `FieldsState` trait, and the `NoFieldsSet` alias for the state `build` starts from.
///
/// A builder's `State` param is a tuple of [`Set`] / [`No`] markers, one per field. This trait is
/// what a builder reads it through: `Field<N>` for whether a field has been set, `Set<N>` for
/// the state after setting one.
///
/// One trait, with as many slots as the largest builder needs, shared by every builder. A builder
/// with fewer fields leaves the slots above its own count permanently [`No`], and never names them:
/// `finish` only bounds the ones it has. The alternative - one trait per number of fields - costs
/// the same declarations over again for each, and they are O(n^2).
///
/// `Set<N>` says only that the result is another state, not what it does to each `Field<N>`.
/// So a chain of setters resolves by normalizing through the impl below, which needs `State` to be
/// a concrete tuple - as it is at every call site, and inside any function which takes the [`Slot`]
/// for a node and calls `build` on it.
///
/// Declaring each `Set<N>`'s effect on every `Field<N>` would additionally let a function be
/// generic over the state it receives a builder in. That is deliberately not supported: such a
/// function monomorphizes per state it is called with, and those copies are not identical
/// (`defaults` folds its branches differently in each), so it would cost binary size. It also
/// costs 256 lines here, being O(n^2) in the number of fields.
///
/// [`Slot`]: oxc_allocator::Slot
fn generate_fields_state_trait(count: usize) -> TokenStream {
    let field_idents = (0..count).map(fields_state_field_ident).collect::<Vec<_>>();
    let set_idents = (0..count).map(fields_state_set_ident).collect::<Vec<_>>();
    // Type params of the impl below - one per field, in the order the tuple holds them
    let params = (0..count).map(|index| format_ident!("F{}", index + 1)).collect::<Vec<_>>();

    // This state with the fields at `set_indices` set e.g. `(Set, F2, F3)`.
    // Fields not being set keep the impl's own type param, so they pass through unchanged.
    let set_tuple = |set_indices: &[usize]| {
        let states = params.iter().enumerate().map(|(index, param)| {
            if set_indices.contains(&index) { quote!(Set) } else { quote!( #param ) }
        });
        quote!( (#(#states,)*) )
    };
    let set_tuples = (0..count).map(|index| set_tuple(&[index])).collect::<Vec<_>>();
    let set_1_and_2_tuple = set_tuple(&[0, 1]);

    let no_states = iter::repeat_n(quote!(No), count);
    let trait_doc = format!(
        " Which of a builder's fields have been set, as a tuple of {count} [`Set`] / [`No`] markers."
    );

    quote! {
        ///@@line_break
        #[doc = #trait_doc]
        ///
        /// `Field<N>` is whether field `N` has been set.
        /// `Set<N>` is this state with field `N` set.
        /// `Set1And2` is this state with the first 2 fields set (which is what `span` produces).
        ///
        /// Shared by every builder. A builder with fewer fields than this leaves the rest
        /// permanently [`No`], and never names them - `finish` only bounds the fields it has.
        ///
        /// `Set<N>` says only that the result is another state, so a chain of setters resolves
        /// by normalizing through the impl below. That needs `State` to be a concrete tuple,
        /// which it is at every call site, and in any function which takes the [`Slot`]
        /// for a node and calls `build` on it. A function generic over the state it receives
        /// a builder in is not supported - it would monomorphize per state it is called with.
        pub trait FieldsState {
            #(type #field_idents: FieldState;)*

            ///@@line_break
            #(type #set_idents: FieldsState;)*

            // The `span` setter sets both halves of the span at once, which is states 1 and 2
            // of every builder that has one
            ///@@line_break
            type Set1And2: FieldsState;
        }

        ///@@line_break
        impl<#(#params: FieldState),*> FieldsState for (#(#params,)*) {
            #(type #field_idents = #params;)*

            ///@@line_break
            #(type #set_idents = #set_tuples;)*

            ///@@line_break
            type Set1And2 = #set_1_and_2_tuple;
        }

        ///@@line_break
        /// State of a builder with none of its fields set yet, which `build` starts from.
        pub type NoFieldsSet = (#(#no_states,)*);
    }
}

/// Generate the `*IsSet` traits - one for every field state name used by any builder.
///
/// Each is implemented only for `Set`, and names the setter which has not been called,
/// so that a missing field produces ``error: `.operator()` has not been called on this builder``.
///
/// These go in the `markers` module, which is `#[doc(hidden)]`, so they don't need to be
/// hidden individually.
fn generate_marker_traits(state_names: &FxHashSet<&str>) -> TokenStream {
    let mut state_names = state_names.iter().copied().collect::<Vec<_>>();
    state_names.sort_unstable();

    state_names
        .into_iter()
        .map(|state_name| {
            let trait_ident = create_safe_ident(&marker_trait_name(state_name));
            // Name the setter as it's spelled at the call site, which for a reserved word
            // e.g. `type` means `.r#type()`
            let setter_ident = create_ident(state_name);
            let message = format!("`.{setter_ident}()` has not been called on this builder");
            quote! {
                ///@@line_break
                #[diagnostic::on_unimplemented(message = #message)]
                pub trait #trait_ident: IsSet {}
                impl #trait_ident for Set {}
            }
        })
        .collect()
}

/// Generate the builder type for a struct, the methods which start one, and its own methods.
///
/// Returns the generated code, and the names of the builder's field states,
/// which the caller needs to generate the marker traits they're bounded on.
fn generate_node_builder<'s>(
    struct_def: &'s StructDef,
    node_id_cell_type_id: TypeId,
    span_type_id: TypeId,
    schema: &Schema,
) -> (TokenStream, Vec<&'s str>) {
    let (state_names, setters) =
        get_states_and_setters(struct_def, node_id_cell_type_id, span_type_id, schema);

    let builder_ident = builder_type_ident(struct_def.name());

    let builder_struct = generate_builder_struct(struct_def, &builder_ident);
    let build_methods = generate_build_methods(struct_def, &builder_ident, schema);
    let slot_build_impl = generate_slot_build_impl(struct_def, &builder_ident, schema);
    let builder_impls = generate_builder_impls(
        struct_def,
        &builder_ident,
        &setters,
        &state_names,
        node_id_cell_type_id,
        schema,
    );

    let tokens = quote! {
        #builder_struct
        #build_methods
        #slot_build_impl
        #builder_impls
    };

    (tokens, state_names)
}

/// Generate the builder type for a struct.
///
/// `Target` is where the node is written and what `finish` returns, and it holds the exclusive
/// reference to the node, so the builder is only that reference. `State` exists solely to record
/// which fields have been set, so `PhantomData` is what holds it, along with `'a` - which `Target`
/// constrains, but does not name in the builder's own params.
///
/// `State` is a single param holding a tuple, rather than one param per field, because a setter
/// can then name its return state as a projection off it (`State::Set3`) instead of respelling
/// every other field's param. That is the difference between a signature which fits on one line
/// and one which does not, 1330 times over.
fn generate_builder_struct(struct_def: &StructDef, builder_ident: &Ident) -> TokenStream {
    let struct_name = struct_def.name();
    let article = article_for(struct_name);
    let doc1 = format!(" Builder for {article} [`{struct_name}`].");
    let doc2 = format!(" Created by [`{struct_name}::build`].");

    quote! {
        ///@@line_break
        #[doc = #doc1]
        ///
        /// Constructs the node in place in the memory arena.
        #[doc = #doc2]
        ///
        /// `State` records which fields have been set - see [`FieldsState`].
        /// [`finish`] is only callable once they are all [`Set`].
        ///
        /// [`finish`]: Self::finish
        #[must_use]
        #[repr(transparent)]
        pub struct #builder_ident<'a, Target, State> {
            target: Target,
            marker: PhantomData<(&'a (), State)>,
        }
    }
}

/// Generate the `build` and `uninit` methods on the AST type.
///
/// `build` allocates the node and returns a builder writing into it. `uninit` only reserves
/// the memory, for a caller which will fill it another way.
fn generate_build_methods(
    struct_def: &StructDef,
    builder_ident: &Ident,
    schema: &Schema,
) -> TokenStream {
    let struct_name = struct_def.name();
    let struct_ty = struct_def.ty(schema);
    let article = article_for(struct_name);

    // Structs which have no lifetime of their own take `'a` as a param of the methods instead
    let (type_lifetime, fn_lifetime) = if struct_def.has_lifetime {
        (Some(quote!( <'a> )), None)
    } else {
        (None, Some(quote!( <'a> )))
    };

    let initial_builder_ty = quote!( #builder_ident<'a, OwnedSlot<'a, Self>, NoFieldsSet> );

    let build_doc1 = format!(" Start building {article} [`{struct_name}`] in the memory arena.");
    let build_doc2 = format!(" [`finish`]: {builder_ident}::finish");
    let uninit_doc = format!(
        " Reserve memory in the arena for {article} [`{struct_name}`], without initializing it."
    );

    quote! {
        ///@@line_break
        impl #type_lifetime #struct_ty {
            #[doc = #build_doc1]
            ///
            /// Set every field on the returned builder, then call [`finish`].
            ///
            #[doc = #build_doc2]
            #[inline(always)]
            pub fn build #fn_lifetime (builder: &impl GetAstBuilder<'a>) -> #initial_builder_ty {
                let builder = builder.builder();
                // Allocate via `&Allocator` (not `builder`), so `OwnedSlot::new_in` shares
                // a monomorphization with every other `&Allocator`-based allocation
                #builder_ident::new(OwnedSlot::new_in(&builder.allocator()), builder)
            }

            ///@@line_break
            #[doc = #uninit_doc]
            ///
            /// Returns a [`Box`] of uninitialized memory.
            /// Write the node into it with [`fill`], or [`fill_with`].
            ///
            /// [`Box`]: ArenaBox
            /// [`fill`]: ArenaBox::fill
            /// [`fill_with`]: ArenaBox::fill_with
            //
            // `#[inline(always)]` because this just delegates, and the allocation it delegates
            // to is a bump-pointer increment
            #[inline(always)]
            pub fn uninit #fn_lifetime (allocator: &impl GetAllocator<'a>) -> ArenaBox<'a, MaybeUninit<Self>> {
                ArenaBox::new_uninit_in(&allocator.allocator())
            }
        }
    }
}

/// Generate the [`SlotBuild`] impl for a struct, so that a field holding the struct can be
/// built in place rather than filled with a value.
///
/// Takes a builder, like `build` on the node type does, and for the same reason - `new` writes
/// the `node_id` field, which has no setter.
///
/// A field of another shape - `Option<T>`, `Box<T>` - is narrowed to a `Slot<T>` first,
/// with the `Slot` conversion methods.
///
/// This is a trait impl rather than an inherent `Slot::build` method because `Slot` belongs to
/// `oxc_allocator`, and an inherent impl has to live in the crate which defines the type.
///
/// [`SlotBuild`]: oxc_ast::builder::builders::SlotBuild
fn generate_slot_build_impl(
    struct_def: &StructDef,
    builder_ident: &Ident,
    schema: &Schema,
) -> TokenStream {
    let struct_name = struct_def.name();
    let struct_ty = struct_def.ty(schema);

    // `'a` is always declared here, even for a struct which has no lifetime of its own and so
    // does not name it in `Self`. `SlotBuild<'a>` constrains it either way, and it is what
    // `Builder` and the `GetAstBuilder` bound are in terms of.
    //
    // The slot's own lifetime is elided - naming the target as `Self` means it is never referred to.
    let doc = format!(" Build the [`{struct_name}`] in place, in this [`Slot`].");
    quote! {
        ///@@line_break
        impl<'a> SlotBuild<'a> for Slot<'_, #struct_ty> {
            type Builder = #builder_ident<'a, Self, NoFieldsSet>;

            ///@@line_break
            #[doc = #doc]
            #[inline(always)]
            fn build(self, builder: &impl GetAstBuilder<'a>) -> Self::Builder {
                #builder_ident::new(self, builder.builder())
            }
        }
    }
}

/// Generate the `impl` blocks on a builder type, containing all of its methods.
///
/// `new` gets a block of its own, restricted to `NoFieldsSet`, so that it cannot mint a builder
/// which claims fields are set that aren't. It's the only other way `Set` could enter `State`,
/// and `finish`'s soundness rests on a setter being the only way.
///
/// The rest share a block. `finish`'s bounds are on that block's own params, rather than on a
/// separate `impl #builder_ident<'a, Set, Set, ...>` block, so that a missing field can name
/// itself in the error via the marker trait's `on_unimplemented` diagnostic.
///
/// `transition` takes the builder to one with different field states, once a setter has written
/// its field, or to a different target, for `with`. It's separate from `new` because `new` also
/// writes `node_id` - the one field which has no setter, so can only be written when the builder
/// is created. Its new state is unbounded: each caller's own return type says what that state is,
/// and pins it there.
fn generate_builder_impls(
    struct_def: &StructDef,
    builder_ident: &Ident,
    setters: &[Setter],
    state_names: &[&str],
    node_id_cell_type_id: TypeId,
    schema: &Schema,
) -> TokenStream {
    let struct_name = struct_def.name();
    let struct_ty = struct_def.ty(schema);

    let write_node_id = generate_node_id_write(struct_def, node_id_cell_type_id);

    let setter_methods = setters
        .iter()
        .map(|setter| generate_setter(setter, struct_name, builder_ident))
        .collect::<TokenStream>();
    let defaults_method = generate_defaults_setter(setters, struct_name, builder_ident);

    let finish_bounds = state_names.iter().enumerate().map(|(index, state_name)| {
        let field_ident = fields_state_field_ident(index);
        let trait_ident = create_safe_ident(&marker_trait_name(state_name));
        quote!( State::#field_ident: #trait_ident )
    });

    let finish_doc = format!(" Finish building the [`{struct_name}`].");
    let with_doc = format!(" Build the [`{struct_name}`] by handing this builder to `build`.");

    quote! {
        ///@@line_break
        impl<'a, Target: BuilderTarget<#struct_ty>> #builder_ident<'a, Target, NoFieldsSet> {
            #[inline(always)]
            fn new(mut target: Target, builder: &impl AstBuild<'a>) -> Self {
                #write_node_id
                Self { target, marker: PhantomData }
            }
        }

        ///@@line_break
        impl<'a, Target, State> #builder_ident<'a, Target, State>
        where
            Target: BuilderTarget<#struct_ty>,
            State: FieldsState,
        {
            #setter_methods
            #defaults_method

            ///@@line_break
            #[doc = #finish_doc]
            ///
            /// Returns a [`Box`](ArenaBox) containing the node, or - if the node is being built
            /// into a field of its parent - a [`SlotFilled`] token.
            ///
            /// Only callable once every field has been set.
            //
            // `#[inline(always)]` because this is a no-op at runtime
            #[inline(always)]
            pub fn finish(self) -> Target::Output where #(#finish_bounds),* {
                // SAFETY: The bounds above require every field's state to be `Set`, which only
                // that field's setter can produce, so the node is fully initialized
                unsafe { self.target.assume_filled() }
            }

            ///@@line_break
            #[doc = #with_doc]
            ///
            /// `build` is given this builder, re-targeted to write into a [`Slot`].
            /// Fields already set stay set. `build` sets whatever is left, then calls `finish`.
            //
            // `#[inline(always)]` because this is a no-op at runtime
            #[inline(always)]
            pub fn with(
                mut self,
                build: impl for<'slot> FnOnce(
                    #builder_ident<'a, Slot<'slot, #struct_ty>, State>,
                ) -> SlotFilled<'slot>,
            ) -> Target::Output {
                // Reborrow, so that the target is live again once `build` returns and can be finished.
                // `node_id` was written when this builder was created, and the reborrowed builder
                // shares that node, so it is not written again.
                let inner = #builder_ident { target: self.target.reborrow(), marker: PhantomData };
                build(inner);
                // SAFETY: The `SlotFilled` token `build` returned can only have come from finishing
                // the builder above, which writes this same node, so it is fully initialized
                unsafe { self.target.assume_filled() }
            }

            ///@@line_break
            #[inline(always)]
            fn transition<NewState>(self) -> #builder_ident<'a, Target, NewState> {
                #builder_ident { target: self.target, marker: PhantomData }
            }
        }
    }
}

/// Generate the setter method(s) for one field of a builder.
///
/// Fields which hold an AST struct inline get a second `<field>_with` method, which writes
/// the field in place instead of taking it by value.
fn generate_setter(setter: &Setter, struct_name: &str, builder_ident: &Ident) -> TokenStream {
    let Setter { name, param_ty, value, field_path, doc, states, with_ty, .. } = setter;

    let ident = create_ident(name);
    let return_ty = setter_return_ty(builder_ident, states.clone());

    let setter_method = quote! {
        ///@@line_break
        #[doc = #doc]
        #[inline(always)]
        pub fn #ident(mut self, #ident: #param_ty) -> #return_ty {
            // SAFETY: The target points to memory in the arena laid out for this node type,
            // so this field is valid for writing
            unsafe { (&raw mut (*self.target.as_mut_ptr()).#field_path).write(#value) };
            self.transition()
        }
    };

    let Some(with_ty) = with_ty else { return setter_method };
    let with_method = generate_with_setter(setter, with_ty, struct_name, &return_ty);
    quote!( #setter_method #with_method )
}

/// Generate a `<field>_with` method, which fills a field in place.
fn generate_with_setter(
    setter: &Setter,
    field_ty: &TokenStream,
    struct_name: &str,
    return_ty: &TokenStream,
) -> TokenStream {
    let field_name = &setter.name;
    let fn_ident = format_ident!("{field_name}_with");
    let field_path = &setter.field_path;

    let doc1 = format!(" Set `{field_name}` on the [`{struct_name}`] being built, in place.");
    let doc2 = format!(" `fill` is given a [`Slot`] for the `{field_name}` field.");

    quote! {
        ///@@line_break
        #[doc = #doc1]
        ///
        #[doc = #doc2]
        /// It must fill the slot, and return the [`SlotFilled`] token which that produces.
        #[inline(always)]
        pub fn #fn_ident(
            mut self,
            fill: impl for<'slot> FnOnce(Slot<'slot, #field_ty>) -> SlotFilled<'slot>,
        ) -> #return_ty {
            // SAFETY: The target points to memory in the arena laid out for this node type, so this
            // field is valid for writing.
            // Offsetting a non-null pointer by a field offset cannot produce null.
            let slot = unsafe { Slot::new(&raw mut (*self.target.as_mut_ptr()).#field_path) };
            fill(slot);
            // The `SlotFilled` token returned by `fill` can only have come from filling this slot,
            // so the field is initialized
            self.transition()
        }
    }
}

/// Generate a `defaults` method, which sets every field that has a default value.
///
/// Returns `None` if the struct has no default fields.
///
/// Fields which have been set already are skipped, so `defaults` can be called at any point
/// in the chain. `FieldState::IS_SET` is a constant, so each branch folds away entirely.
fn generate_defaults_setter(
    setters: &[Setter],
    struct_name: &str,
    builder_ident: &Ident,
) -> Option<TokenStream> {
    let defaults = setters.iter().filter(|setter| setter.is_default).collect::<Vec<_>>();
    if defaults.is_empty() {
        return None;
    }

    // Default fields are not necessarily contiguous, so this walks the states it sets one by one,
    // rather than covering a range
    let return_ty =
        setter_return_ty(builder_ident, defaults.iter().map(|setter| setter.states.start));

    let writes = defaults.iter().map(|setter| {
        let field_ident = fields_state_field_ident(setter.states.start);
        let Setter { field_path, .. } = setter;
        quote! {
            ///@@line_break
            if !State::#field_ident::IS_SET {
                // SAFETY: The target points to memory in the arena laid out for this node type,
                // so this field is valid for writing
                unsafe { (&raw mut (*self.target.as_mut_ptr()).#field_path).write(Default::default()) };
            }
        }
    });

    let names = defaults.iter().map(|setter| format!("`{}`", setter.name)).collect::<Vec<_>>();
    let field_names = match names.as_slice() {
        [name] => name.clone(),
        [first, last] => format!("{first} and {last}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
        [] => unreachable!(),
    };

    let doc1 = format!(
        " Set {field_names} field{} of the [`{struct_name}`]",
        if names.len() > 1 { "s" } else { "" }
    );
    let doc2 = if names.len() > 1 {
        " to their default values, if they've not been set already."
    } else {
        " to its default value, if it's not been set already."
    };

    Some(quote! {
        ///@@line_break
        #[doc = #doc1]
        #[doc = #doc2]
        #[inline(always)]
        pub fn defaults(mut self) -> #return_ty {
            #(#writes)*

            ///@@line_break
            self.transition()
        }
    })
}

/// Generate the `into_<variant>` extension trait for an enum, so that a field holding the enum
/// can be built in place rather than filled with a value.
///
/// Each method narrows a `Slot` for the enum to a `Slot` for one variant's payload, writing the
/// discriminant on the way. A boxed variant lands on a `Slot<Box<T>>`, which
/// [`into_contents`](oxc_allocator::Slot::into_contents) then takes to a `Slot<T>`.
///
/// This is a trait, one per enum, rather than an inherent impl on `Slot`, because `Slot` belongs
/// to `oxc_allocator`. It cannot be one trait shared by every enum, because the method names
/// collide - `Expression` and `SimpleAssignmentTarget` both have an `into_identifier`,
/// returning slots for different payloads.
///
/// A fieldless enum gets nothing - it is a `u8`, so a field holding one is written with its setter.
/// Fieldless variants of a fieldful enum are skipped for the same reason: there is no payload to
/// narrow to, so the callee fills the slot with the variant instead.
fn generate_enum_slot_trait(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let enum_name = enum_def.name();
    let enum_ty = enum_def.ty(schema);
    let trait_ident = enum_slot_trait_ident(enum_name);

    // An enum with no lifetime of its own doesn't constrain `'a`, and none of its payloads can
    // hold one either, so there's no `'a` to introduce.
    //
    // `'a: 'slot` - the arena outlives the borrow of a place within it. That is implied on the
    // impl below, by `Slot<'slot, Enum<'a>>` holding a `&'slot mut`, but implied bounds do not
    // reach the trait's default method bodies, and `into_contents` needs it.
    let has_lifetime = enum_def.has_lifetime(schema);
    let lifetime_decl = has_lifetime.then(|| quote!( 'a: 'slot, ));
    let lifetime_arg = has_lifetime.then(|| quote!( 'a, ));

    // Bodies are default bodies on the trait, so the impl below is empty. They are sound for any
    // implementor because the `EnumSlot<'slot, Enum>` supertrait pins the enum they narrow within,
    // and `EnumSlot` is sealed, so `Slot` is the only implementor there can be.
    let methods = enum_def
        .all_variants(schema)
        .filter_map(|variant| {
            let payload_type = variant.field_type(schema)?;
            let payload_ty = payload_type.ty(schema);
            let into_fn_ident = format_ident!("into_{}", variant.snake_name());
            let discriminant = number_lit(variant.discriminant);
            let variant_name = variant.name();
            let doc = format!(
                " Narrow the [`Slot`] to one for the payload of [`{enum_name}::{variant_name}`]."
            );

            let build_method =
                generate_enum_slot_build_method(enum_name, variant, payload_type, schema);

            Some(quote! {
                ///@@line_break
                #[doc = #doc]
                #[inline(always)]
                fn #into_fn_ident(self) -> Slot<'slot, #payload_ty> {
                    unsafe { self.into_variant(#discriminant) }
                }

                #build_method
            })
        })
        .collect::<TokenStream>();

    let article = article_for(enum_name);
    let trait_doc = format!(
        " Narrow a [`Slot`] for {article} [`{enum_name}`] to one for a variant's payload.",
    );

    quote! {
        ///@@line_break
        #[doc = #trait_doc]
        ///
        /// Implemented only for [`Slot`], which belongs to `oxc_allocator`, so these cannot be
        /// inherent methods. The `EnumSlot` bound supplies `into_variant`, and pins it to
        /// this enum, which is what makes the method bodies below sound.
        pub trait #trait_ident<#lifetime_decl 'slot>: EnumSlot<'slot, #enum_ty> {
            #methods
        }

        ///@@line_break
        impl<#lifetime_decl 'slot> #trait_ident<#lifetime_arg 'slot> for Slot<'slot, #enum_ty> {}
    }
}

/// Generate a `build_<variant>` shortcut for a variant of an enum, which narrows the [`Slot`]
/// and starts a builder on the node in one step.
///
/// `slot.into_numeric_literal().into_contents(builder).build(builder)` becomes
/// `slot.build_numeric_literal(builder)`. The `into_contents` step is only there for a boxed
/// variant, and getting it wrong is a type error rather than anything subtle, but it is noise
/// at every call site.
///
/// Taking `&impl GetAstBuilder` and calling `builder()` also means the caller needs only that
/// trait. Spelled out, the chain needs [`GetAllocator`] too, for `into_contents` - which a type
/// that merely holds a builder (a parser or traversal context) does not necessarily have.
///
/// Returns `None` for a variant whose payload is not a node with a builder of its own.
///
/// [`GetAllocator`]: oxc_allocator::GetAllocator
fn generate_enum_slot_build_method(
    enum_name: &str,
    variant: &VariantDef,
    payload_type: &TypeDef,
    schema: &Schema,
) -> Option<TokenStream> {
    // Peel the `Box` off a boxed variant. What is left has to be a struct with a builder.
    let (inner_type, is_boxed) = match payload_type {
        TypeDef::Box(box_def) => (box_def.inner_type(schema), true),
        type_def => (type_def, false),
    };
    let TypeDef::Struct(struct_def) = inner_type else { return None };
    if !has_builder(StructOrEnum::Struct(struct_def)) {
        return None;
    }

    let fn_ident = format_ident!("build_{}", variant.snake_name());
    let into_fn_ident = format_ident!("into_{}", variant.snake_name());
    let builder_ident = builder_type_ident(struct_def.name());
    let struct_ty = struct_def.ty(schema);
    // Allocate via `&Allocator` (not `builder`), so `into_contents` shares a monomorphization
    // with every other `&Allocator`-based allocation
    let into_contents = is_boxed.then(|| quote!( .into_contents(&builder.allocator()) ));

    let article = article_for(enum_name);
    let variant_name = variant.name();
    let doc =
        format!(" Build {article} [`{enum_name}::{variant_name}`] in place in this [`Slot`].");

    Some(quote! {
        ///@@line_break
        #[doc = #doc]
        #[inline(always)]
        fn #fn_ident(
            self,
            builder: &impl GetAstBuilder<'a>,
        ) -> #builder_ident<'a, Slot<'slot, #struct_ty>, NoFieldsSet> {
            let builder = builder.builder();
            self.#into_fn_ident() #into_contents .build(builder)
        }
    })
}

/// A setter method on a builder type.
struct Setter {
    /// Method name e.g. `span_start`
    name: String,
    /// Type of the method's parameter e.g. `u32`
    param_ty: TokenStream,
    /// Expression the method writes. Usually just the param, but a default field takes its
    /// innermost type, so the param has to be wrapped e.g. `Cell::new(Some(scope_id))`.
    value: TokenStream,
    /// Path from the node to the memory the method writes e.g. `span.start`
    field_path: TokenStream,
    /// Doc comment for the method
    doc: String,
    /// Field states which the method sets to `Set`
    states: Range<usize>,
    /// Type of the field, if it should get a `<field>_with` method
    with_ty: Option<TokenStream>,
    /// `true` if the field has a default value, so `defaults` can set it
    is_default: bool,
}

/// Get the field states and setter methods for a struct's builder.
///
/// Every field gets 1 state and 1 setter, except:
///
/// * `node_id`, which `build` fills in itself, so gets neither.
/// * `span`, which gets 2 states and 3 setters, as its 2 halves are usually known at
///   different times. Other [`Span`] fields (e.g. `TSThisParameter::this_span`) are known
///   all at once, so they get a single setter like any other field.
///
/// Each state is named after the setter which sets that state alone.
fn get_states_and_setters<'s>(
    struct_def: &'s StructDef,
    node_id_cell_type_id: TypeId,
    span_type_id: TypeId,
    schema: &Schema,
) -> (Vec<&'s str>, Vec<Setter>) {
    let struct_name = struct_def.name();
    let mut state_names = vec![];
    let mut setters = vec![];

    for field in &struct_def.fields {
        if field.type_id == node_id_cell_type_id {
            continue;
        }

        let name = field.name();
        let ident = field.ident();
        let type_def = field.type_def(schema);
        let ty = type_def.ty(schema);
        let index = state_names.len();

        if field.type_id == span_type_id && name == "span" {
            // The 2 halves of `span` are the only states which aren't named after a field
            let (start_state, end_state) = ("span_start", "span_end");
            state_names.push(start_state);
            state_names.push(end_state);

            // The 2 halves, then both at once. `value` is the param ident in each case,
            // which is the setter's own name.
            let span_setters = [
                (start_state, quote!(u32), quote!(span.start), " start", index..index + 1),
                (end_state, quote!(u32), quote!(span.end), " end", index + 1..index + 2),
                ("span", ty, quote!(span), "", index..index + 2),
            ];
            setters.extend(span_setters.map(|(name, param_ty, field_path, half, states)| {
                let ident = create_safe_ident(name);
                Setter {
                    name: name.to_string(),
                    param_ty,
                    value: quote!( #ident ),
                    field_path,
                    doc: format!(" Set `span`{half} on the [`{struct_name}`] being built."),
                    states,
                    with_ty: None,
                    is_default: false,
                }
            }));
        } else {
            state_names.push(name);

            let is_default = is_default_field(field, schema);
            let (param_ty, value) = if is_default {
                let inner_ty = type_def.innermost_type(schema).ty(schema);
                (inner_ty, wrap_default_field_value(quote!( #ident ), type_def, schema))
            } else if let TypeDef::Primitive(primitive_def) = type_def
                && matches!(primitive_def.name(), "Str" | "Ident")
            {
                (quote!( impl Into<#ty> ), quote!( #ident.into() ))
            } else {
                (ty, quote!( #ident ))
            };

            setters.push(Setter {
                name: name.to_string(),
                param_ty,
                value,
                field_path: quote!( #ident ),
                doc: format!(" Set `{name}` on the [`{struct_name}`] being built."),
                states: index..index + 1,
                with_ty: get_with_field_ty(field, schema),
                is_default,
            });
        }
    }

    (state_names, setters)
}

/// Get the type of a field which should have a `<field>_with` method, if it should have one.
///
/// A field gets one if the callee producing it can write into the slot *before* doing its work,
/// because then the slot pointer is dead by the time any of that work happens, and it costs the
/// callee nothing. Anything which is allocated qualifies: its pointer exists before its contents
/// do. A value which is computed does not - it isn't known until the end, so the slot pointer has
/// to survive the whole body, which costs a callee-saved register that the caller's saving does
/// not repay.
///
/// So, after peeling off the transparent `Option` and `Cell` wrappers:
///
/// * A `Box` or a `Vec` is an allocation.
/// * An enum with fields is a node, or a pointer to one.
/// * A struct is a node, unless it's small enough to be a scalar pair - which in the AST means
///   only `Span`.
/// * Anything else is computed: primitives, fieldless enums.
///
/// Plus a catch-all for anything over 16 bytes, which is returned via a hidden pointer whatever
/// its shape, so passing a slot cannot lose.
///
/// Sizes are for 64-bit platforms.
fn get_with_field_ty(field: &FieldDef, schema: &Schema) -> Option<TokenStream> {
    let field_type = field.type_def(schema);

    let is_filled_in_place = if field_type.layout_64().size > 16 {
        true
    } else {
        let mut type_def = field_type;
        loop {
            type_def = match type_def {
                TypeDef::Option(option_def) => option_def.inner_type(schema),
                TypeDef::Cell(cell_def) => cell_def.inner_type(schema),
                _ => break,
            };
        }

        match type_def {
            TypeDef::Box(_) | TypeDef::Vec(_) => true,
            TypeDef::Enum(enum_def) => !enum_def.is_fieldless(),
            TypeDef::Struct(struct_def) => {
                struct_def.fields.len() > 1 && struct_def.layout_64().size > 8
            }
            _ => false,
        }
    };

    is_filled_in_place.then(|| field_type.ty(schema))
}

/// Generate the statement in `new` which writes the `node_id` field, through the target.
///
/// Every AST type which is visited has a `node_id` field, and only visited types get builders.
fn generate_node_id_write(struct_def: &StructDef, node_id_cell_type_id: TypeId) -> TokenStream {
    let field = struct_def
        .fields
        .iter()
        .find(|field| field.type_id == node_id_cell_type_id)
        .expect("Node type with a builder has no `node_id` field");
    let field_ident = field.ident();

    quote! {
        // SAFETY: The target points to memory in the arena laid out for this node type,
        // so the `node_id` field is valid for writing
        unsafe { init_node_id(&raw mut (*target.as_mut_ptr()).#field_ident, builder) };
    }
}

/// Get the builder type a setter returns - the current `Target`, and `State` with the states
/// at `indices` set e.g. `BinaryExpressionBuilder<'a, Target, State::Set3>`.
///
/// One projection per state, applied in turn. `State::SetN` is enough for the first, as the
/// `impl` block's own bound says what `State` is. Each one after that is off a projection,
/// which needs the trait spelled out.
///
/// A setter usually sets a single state, so gets a single projection. `span` sets the first 2,
/// which `Set1And2` covers in one. `defaults` sets one per default field, and chains.
fn setter_return_ty(builder_ident: &Ident, indices: impl Iterator<Item = usize>) -> TokenStream {
    let indices = indices.collect::<Vec<_>>();

    let state = if indices == [0, 1] {
        quote!(State::Set1And2)
    } else {
        let mut indices = indices.into_iter();
        let first =
            fields_state_set_ident(indices.next().expect("Setter must set at least 1 state"));
        let mut state = quote!( State::#first );
        for index in indices {
            let set_ident = fields_state_set_ident(index);
            state = quote!( <#state as FieldsState>::#set_ident );
        }
        state
    };

    quote!( #builder_ident<'a, Target, #state> )
}

/// Get name of the associated type giving the state of field `index` e.g. `Field3`.
fn fields_state_field_ident(index: usize) -> Ident {
    format_ident!("Field{}", index + 1)
}

/// Get name of the associated type giving this state with field `index` set e.g. `Set3`.
fn fields_state_set_ident(index: usize) -> Ident {
    format_ident!("Set{}", index + 1)
}

/// Get name of the builder type for a struct.
fn builder_type_ident(struct_name: &str) -> Ident {
    format_ident!("{struct_name}Builder")
}

/// Get name of the `Slot` extension trait for an enum.
fn enum_slot_trait_ident(enum_name: &str) -> Ident {
    format_ident!("{enum_name}Slot")
}

/// Get name of the `*IsSet` marker trait for a field state e.g. `SpanStartIsSet`.
fn marker_trait_name(state_name: &str) -> String {
    format!("{}IsSet", state_name.to_case(Case::Pascal))
}

/// Get whether a field has a default value, so it doesn't have to be provided by the caller.
///
/// A field is default if it's marked `#[builder(default)]`, or its innermost type is.
fn is_default_field(field: &FieldDef, schema: &Schema) -> bool {
    field.builder.is_default
        || match field.type_def(schema).innermost_type(schema) {
            TypeDef::Struct(struct_def) => struct_def.builder.is_default,
            TypeDef::Enum(enum_def) => enum_def.builder.is_default,
            _ => false,
        }
}

/// Get whether builder methods are generated for a type.
fn has_builder(type_def: StructOrEnum) -> bool {
    match type_def {
        StructOrEnum::Struct(struct_def) => {
            !struct_def.builder.skip && struct_def.visit.has_visitor()
        }
        StructOrEnum::Enum(enum_def) => !enum_def.builder.skip && enum_def.visit.has_visitor(),
    }
}

/// Get [`TypeId`] of `Cell<NodeId>` - the type of the `node_id` field which every node has.
fn node_id_cell_type_id(schema: &Schema) -> TypeId {
    schema.type_by_name("NodeId").as_struct().unwrap().containers.cell_id.unwrap()
}
