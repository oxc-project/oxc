//! Generator for `oxc_formatter`.
//! Generates the `AstNodes` and `AstNode` types.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    Codegen, Generator,
    generators::define_generator,
    output::Output,
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, TypeId},
};

const FORMATTER_CRATE_PATH: &str = "crates/oxc_formatter";

pub fn formatter_output_path(file_name: &str) -> String {
    format!("{FORMATTER_CRATE_PATH}/src/ast_nodes/generated/{file_name}.rs")
}

pub fn get_node_type(ty: &TokenStream) -> TokenStream {
    quote! { AstNode<'a, #ty> }
}

/// Based on the printing comments algorithm, the last child of these AST nodes don't need to print comments.
/// Without following nodes could lead to only print comments that before the end of the node, which is what we want.
const AST_NODE_WITHOUT_FOLLOWING_NODE_LIST: &[&str] = &[];

const AST_NODE_WITH_FOLLOWING_NODE_LIST: &[&str] = &["Function", "Class"];

pub struct FormatterAstNodesGenerator;

define_generator!(FormatterAstNodesGenerator);

impl Generator for FormatterAstNodesGenerator {
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let no_following_node_type_ids = get_no_following_node_type_ids(schema);

        let impls = schema
            .types
            .iter()
            .filter_map(|type_def| match type_def {
                TypeDef::Struct(struct_def)
                    if struct_def.visit.has_visitor() && !struct_def.builder.skip =>
                {
                    Some(generate_struct_impls(struct_def, &no_following_node_type_ids, schema))
                }
                TypeDef::Enum(enum_def) if enum_def.visit.has_visitor() => {
                    Some(generate_enum_impls(enum_def, schema))
                }
                _ => None,
            })
            .collect::<TokenStream>();

        let ast_nodes_names = schema
            .types
            .iter()
            .filter_map(|type_def| match type_def {
                TypeDef::Struct(struct_def) if struct_def.kind.has_kind => {
                    Some((struct_def.ident(), struct_def.lifetime(schema)))
                }
                TypeDef::Enum(enum_def) if enum_def.kind.has_kind => {
                    Some((enum_def.ident(), enum_def.lifetime(schema)))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        let ast_nodes_variants = ast_nodes_names.iter().map(|(name, lifetime)| {
            quote! {
                #name(&'a AstNode<'a, #name #lifetime>),
            }
        });

        let dummy_variant = quote! {
            Self::Dummy() => panic!("Should never be called on a dummy node"),
        };

        let span_match_arms = ast_nodes_names.iter().map(|(name, _)| {
            if name == "Argument" {
                // This is an enum, so its span is the same as its inner node's span.
                // From experience, we should get the real parent's span.
                quote! { Self::#name(n) => n.parent.span(), }
            } else {
                quote! { Self::#name(n) => n.span(), }
            }
        });

        let parent_match_arms = ast_nodes_names.iter().map(|(name, _)| {
            quote! { Self::#name(n) => n.parent(), }
        });

        let ast_nodes_debug_names = ast_nodes_names.iter().map(|(name, _)| {
            let debug_name = name.to_string();
            quote! { Self::#name(_) => #debug_name, }
        });

        let transmute_self = quote! {
            #[inline]
            pub(super) fn transmute_self<'a, T>(s: &AstNode<'a, T>) -> &'a AstNode<'a, T> {
                // SAFETY: `s` is already allocated in Arena, so transmute from `&` to `&'a` is safe.
                #[expect(clippy::undocumented_unsafe_blocks)]
                unsafe { transmute(s) }
            }
        };

        let output = quote! {
            use std::mem::transmute;
            ///@@line_break
            use oxc_allocator::Vec;
            use oxc_ast::ast::*;
            use oxc_span::{GetSpan, Ident};
            ///@@line_break
            use crate::ast_nodes::AstNode;
            use crate::formatter::{
                Format, Formatter,
                trivia::{format_leading_comments, format_trailing_comments},
            };



            ///@@line_break
            #transmute_self

            ///@@line_break
            #[derive(Clone, Copy)]
            pub enum AstNodes<'a> {
                Dummy(),
                #(#ast_nodes_variants)*
            }

            impl AstNodes<'_> {
                #[inline]
                pub fn span(&self) -> Span {
                    match self {
                        #dummy_variant
                        #(#span_match_arms)*
                    }
                }

                #[inline]
                pub fn parent(&self) -> &Self {
                    match self {
                        #dummy_variant
                        #(#parent_match_arms)*
                    }
                }

                #[inline]
                pub fn debug_name(&self) -> &'static str {
                    match self {
                        Self::Dummy() => "Dummy",
                        #(#ast_nodes_debug_names)*
                    }
                }
            }

            #impls
        };

        Output::Rust { path: formatter_output_path("ast_nodes"), tokens: output }
    }
}

/// Get [`TypeId`]s of types which do not have a following node.
///
/// These are:
/// * All types which are variants of `Statement`.
/// * PLUS types listed in `AST_NODE_WITHOUT_FOLLOWING_NODE_LIST`.
/// * MINUS types listed in `AST_NODE_WITH_FOLLOWING_NODE_LIST`.
fn get_no_following_node_type_ids(schema: &Schema) -> Vec<TypeId> {
    let exclude_type_ids = AST_NODE_WITH_FOLLOWING_NODE_LIST
        .iter()
        .map(|&name| schema.type_names[name])
        .collect::<Vec<_>>();

    let mut type_ids = AST_NODE_WITHOUT_FOLLOWING_NODE_LIST
        .iter()
        .map(|&name| schema.type_names[name])
        .collect::<Vec<_>>();

    let statement_enum = schema.type_by_name("Statement").as_enum().unwrap();
    type_ids.extend(
        statement_enum
            .all_variants(schema)
            .filter_map(|variant| variant.field_type(schema))
            .map(|variant_type| variant_type.innermost_type(schema).id())
            .filter(|type_id| !exclude_type_ids.contains(type_id)),
    );

    type_ids
}

fn generate_struct_impls(
    struct_def: &StructDef,
    no_following_node_type_ids: &[TypeId],
    schema: &Schema,
) -> TokenStream {
    let type_ty = struct_def.ty(schema);
    let has_kind = struct_def.kind.has_kind;
    let struct_name = struct_def.ident();

    let fields = &struct_def.fields;
    let methods = fields.iter().enumerate().filter_map(|(index, field)| {
        if field.name == "span" {
            return None;
        }

        let field_type_def = field.type_def(schema);
        let is_option = field_type_def.is_option();
        let (original_field_type, is_box) = if let TypeDef::Box(box_def) =
            field_type_def.as_option().map_or(field_type_def, |opt| opt.inner_type(schema))
        {
            (box_def.inner_type(schema), true)
        } else {
            (field_type_def.as_option().map_or(field_type_def, |opt| opt.inner_type(schema)), false)
        };

        let field_name = &field.ident();
        let field_inner_ty = original_field_type.ty(schema);

        let (is_not_ast_node, is_copyable) = match original_field_type {
            TypeDef::Struct(s_def) => {
                let is_copyable = is_copyable(&s_def.derives);
                (is_copyable || !s_def.visit.has_visitor(), is_copyable)
            }
            TypeDef::Enum(e_def) => (!e_def.visit.has_visitor(), is_copyable(&e_def.derives)),
            TypeDef::Primitive(_) => (true, true),
            TypeDef::Vec(vec) => {
                (vec.inner_type(schema).as_struct().is_some_and(|s| !s.visit.has_visitor()), false)
            }
            TypeDef::Cell(_) => return None,
            TypeDef::Option(_) | TypeDef::Box(_) | TypeDef::Pointer(_) => {
                unreachable!("Option/Box/pointer should have been unwrapped");
            }
        };

        let parent_expr = if has_kind {
            quote! { AstNodes::#struct_name(transmute_self(self)) }
        } else {
            quote! { self.parent }
        };

        let reference_prefix = if is_copyable {
            quote! {}
        } else {
            quote! { & }
        };

        let body = if is_not_ast_node {
            if is_option && !is_copyable {
                quote! { self.inner.#field_name.as_ref() }
            } else {
                quote! { #reference_prefix self.inner.#field_name }
            }
        } else {
            let inner_access = if is_box {
                quote! { inner.as_ref() }
            } else {
                quote! { inner }
            };
            let field_access = if is_box {
                quote! { self.inner.#field_name.as_ref() }
            } else {
                quote! { &self.inner.#field_name }
            };

            let should_not_have_following_node =
                no_following_node_type_ids.contains(&struct_def.id);

            // Find the next field that can provide a following_span_start
            let mut next_field_index = index + 1;
            let mut following_span_start = None;
            let mut is_optional_chain = false;

            while let Some(next_field) = fields.get(next_field_index) {
                let next_field_type_def = next_field.type_def(schema);
                if let Some(next_following_node_tmp) =
                    generate_next_following_node_start(next_field, next_field_type_def, schema)
                {
                    if next_field_type_def.is_option() || next_field_type_def.is_vec() {
                        let or_else_following_nodes = build_following_node_chain_until_non_option(
                            &fields[next_field_index + 1..],
                            should_not_have_following_node,
                            schema,
                        );
                        // Check if we have an actual chain or a simple case
                        if or_else_following_nodes.is_empty() {
                            // Simple case - use map_or pattern directly (returns u32)
                            following_span_start = Some(
                                generate_simple_following_node_start(
                                    next_field,
                                    next_field_type_def,
                                    schema,
                                )
                                .unwrap(),
                            );
                            // is_optional_chain stays false since we use map_or
                        } else {
                            is_optional_chain = true;
                            following_span_start = Some(quote! {
                                #next_following_node_tmp #or_else_following_nodes
                            });
                        }
                    } else {
                        // Non-optional field - we get a definite u32 value
                        following_span_start = Some(next_following_node_tmp);
                    }
                    break;
                }
                next_field_index += 1;
            }

            // Generate the final following_span_start expression
            let following_span_start_expr = match (following_span_start, is_optional_chain) {
                (Some(expr), true) => {
                    // Optional chain - need .unwrap_or(0)
                    quote! { let following_span_start = #expr.unwrap_or(0); }
                }
                (Some(expr), false) => {
                    // Direct u32 value (from non-optional field or map_or pattern)
                    quote! { let following_span_start = #expr; }
                }
                (None, _) if should_not_have_following_node => {
                    // No following node and type shouldn't have one
                    quote! { let following_span_start = 0; }
                }
                (None, _) => {
                    // No next field found, use parent's following_span_start
                    quote! { let following_span_start = self.following_span_start; }
                }
            };

            if is_option {
                quote! {
                    #following_span_start_expr
                    self.allocator.alloc(self.inner.#field_name.as_ref().map(|inner| AstNode {
                        inner: #inner_access,
                        allocator: self.allocator,
                        parent: #parent_expr,
                        following_span_start
                    })).as_ref()
                }
            } else {
                quote! {
                    #following_span_start_expr
                    self.allocator.alloc(AstNode {
                        inner: #field_access,
                        allocator: self.allocator,
                        parent: #parent_expr,
                        following_span_start
                    })
                }
            }
        };

        let return_type_final = if is_not_ast_node {
            quote! { #reference_prefix #field_inner_ty }
        } else {
            quote! { &AstNode<'a, #field_inner_ty> }
        };

        let return_type_final = if is_option {
            quote! { Option<#return_type_final> }
        } else {
            return_type_final
        };

        Some(quote! {
            ///@@line_break
            #[inline]
            pub fn #field_name(&self) -> #return_type_final {
                #body
            }
        })
    });

    quote! {
        ///@@line_break
        impl<'a> AstNode<'a, #type_ty> {
            #(#methods)*

            ///@@line_break
            pub fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
                format_leading_comments(
                    self.span()
                )
                .fmt(f);
            }

            ///@@line_break
            pub fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
                format_trailing_comments(
                    self.parent.span(),
                    self.inner.span(),
                    self.following_span_start,
                )
                .fmt(f);
            }
        }
    }
}

/// Generates code that returns `Option<u32>` for the start position of the following node.
fn generate_next_following_node_start(
    next_field: &FieldDef,
    next_field_type_def: &TypeDef,
    schema: &Schema,
) -> Option<TokenStream> {
    let next_field_name = format_ident!("{}", next_field.name());
    let next_field_accessor = quote! {
        self.inner.#next_field_name
    };

    let innermost_type = next_field_type_def.innermost_type(schema);
    if match innermost_type {
        TypeDef::Struct(struct_def) => !struct_def.visit.has_visitor(),
        TypeDef::Enum(enum_def) => !enum_def.visit.has_visitor(),
        _ => true,
    } {
        return None;
    }

    // For non-optional fields, return plain u32 expression
    // For optional/vec fields, return Option<u32> expression
    let following_span_start = match next_field_type_def {
        TypeDef::Box(_) | TypeDef::Struct(_) | TypeDef::Enum(_) => {
            // Non-optional: returns plain u32
            quote! { #next_field_accessor.span().start }
        }
        TypeDef::Option(option_def) => {
            let inner_type = option_def.inner_type(schema);
            let inner_type_call = if inner_type.is_box() {
                quote! { as_deref() }
            } else {
                quote! { as_ref() }
            };
            // Optional: returns Option<u32>
            quote! { #next_field_accessor.#inner_type_call.map(|n| n.span().start) }
        }
        TypeDef::Vec(_) => {
            // Vec: returns Option<u32>
            quote! { #next_field_accessor.first().map(|n| n.span().start) }
        }
        _ => return None,
    };

    Some(following_span_start)
}

/// Generate a simple following_span_start expression using map_or (returns u32 directly).
/// Used when there's no chain (just a single optional/vec field).
fn generate_simple_following_node_start(
    next_field: &FieldDef,
    next_field_type_def: &TypeDef,
    schema: &Schema,
) -> Option<TokenStream> {
    let next_field_ident = next_field.ident();
    let next_field_accessor = quote! { self.inner.#next_field_ident };

    // Generate map_or pattern for Optional/Vec fields (returns u32 directly)
    let following_span_start = match next_field_type_def {
        TypeDef::Option(option_def) => {
            let inner_type = option_def.inner_type(schema);
            let inner_type_call = if inner_type.is_box() {
                quote! { as_deref() }
            } else {
                quote! { as_ref() }
            };
            // Using map_or: returns u32 directly
            quote! { #next_field_accessor.#inner_type_call.map_or(0, |n| n.span().start) }
        }
        TypeDef::Vec(_) => {
            // Using map_or: returns u32 directly
            quote! { #next_field_accessor.first().map_or(0, |n| n.span().start) }
        }
        _ => return None,
    };

    Some(following_span_start)
}

fn build_following_node_chain_until_non_option(
    fields: &[FieldDef],
    should_not_have_following_node: bool,
    schema: &Schema,
) -> TokenStream {
    let mut result = TokenStream::new();

    for field in fields {
        let field_type_def = field.type_def(schema);
        if let Some(following_span_start) =
            generate_next_following_node_start(field, field_type_def, schema)
        {
            // Non-optional fields return plain u32, need to wrap in Some() for .or_else()
            // Optional/Vec fields return Option<u32>, use directly
            if field_type_def.is_option() || field_type_def.is_vec() {
                result = quote! {
                    #result.or_else(|| #following_span_start)
                };
            } else {
                // Non-optional field terminates the chain
                result = quote! {
                    #result.or_else(|| Some(#following_span_start))
                };
                return result;
            }
        }
    }

    if should_not_have_following_node {
        result
    } else {
        quote! { #result.or(Some(self.following_span_start)) }
    }
}

fn generate_enum_impls(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let enum_ident = enum_def.ident();
    let type_ty = enum_def.ty(schema);

    let parent_decl = if enum_def.kind.has_kind {
        quote! { let parent = self.allocator.alloc(AstNodes::#enum_ident(transmute_self(self))); }
    } else {
        quote! { let parent = self.parent; }
    };

    let variant_match_arms = enum_def.variants.iter().map(|variant| {
        let variant_name = &variant.ident();
        let field_type = variant.field_type(schema).unwrap();
        let is_box = field_type.is_box();
        let node_type_ident = field_type
            .maybe_inner_type(schema)
            .map_or_else(|| field_type.ident(), TypeDef::ident);

        let inner_expr = if is_box { quote! { s.as_ref() } } else { quote! { s } };

        let implementation = if has_kind(field_type, schema) {
            quote! {
                AstNodes::#node_type_ident(self.allocator.alloc(AstNode {
                    inner: #inner_expr,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        } else {
            // This panic might indicate a need for further refinement or configuration in your schema/generation
            quote! {
                panic!("No kind for current enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`")
            }
        };
        quote! { #enum_ident::#variant_name(s) => { #implementation }, }
    });

    let inherits_match_arms = enum_def.inherits_types(schema).map(|inherited_type| {
        let inherited_enum_def = inherited_type.as_enum().unwrap();
        let inherited_enum_inner_type_ident = inherited_enum_def
            .maybe_inner_type(schema)
            .map_or_else(|| inherited_enum_def.ident(), TypeDef::ident);

        let inherits_snake_name = inherited_enum_def.snake_name();
        let match_ident = format_ident!("match_{inherits_snake_name}");
        let to_fn_ident = format_ident!("to_{inherits_snake_name}");

        let implementation = if inherited_enum_def.kind.has_kind {
            quote! {
                AstNodes::#inherited_enum_inner_type_ident(self.allocator.alloc(AstNode {
                    inner: it.#to_fn_ident(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }))
            }
        } else {
            quote! {
                return self.allocator.alloc(AstNode {
                    inner: it.#to_fn_ident(),
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }).as_ast_nodes();
            }
        };
        quote! { it @ #match_ident!(#enum_ident) => { #implementation }, }
    });

    let node_type = get_node_type(&type_ty);
    let implementation = if variant_match_arms.len() == 0 {
        quote! {
            #[expect(clippy::needless_return)]
            match self.inner {
                #(#inherits_match_arms)*
            }
        }
    } else {
        quote! {
            let node = match self.inner {
                #(#variant_match_arms)*
                #(#inherits_match_arms)*
            };
            self.allocator.alloc(node)
        }
    };
    quote! {
        ///@@line_break
        impl<'a> #node_type {
            #[inline]
            pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
                #parent_decl
                #implementation
            }
        }
    }
}

fn has_kind(type_def: &TypeDef, schema: &Schema) -> bool {
    match type_def {
        TypeDef::Struct(struct_def) => struct_def.kind.has_kind,
        TypeDef::Enum(enum_def) => enum_def.kind.has_kind,
        TypeDef::Box(box_def) => has_kind(box_def.inner_type(schema), schema),
        _ => false,
    }
}

fn is_copyable(devices: &[String]) -> bool {
    devices.contains(&"Copy".to_string())
}
