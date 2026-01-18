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
            quote! { Self::#name(n) => n.parent, }
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
            use oxc_span::GetSpan;
            use oxc_syntax::node::NodeId;
            ///@@line_break
            use crate::ast_nodes::AstNode;
            use crate::formatter::{
                Format, Formatter,
                trivia::{format_leading_comments, format_trailing_comments},
            };



            ///@@line_break
            #transmute_self

            ///@@line_break
            pub enum AstNodes<'a> {
                Dummy(),
                #(#ast_nodes_variants)*
            }

            impl <'a> AstNodes<'a> {
                #[inline]
                pub fn span(&self) -> Span {
                    match self {
                        #dummy_variant
                        #(#span_match_arms)*
                    }
                }

                #[inline]
                pub fn parent(&self) -> &'a Self {
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
            quote! { self.allocator.alloc(AstNodes::#struct_name(transmute_self(self))) }
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
            let mut following_span = if should_not_have_following_node {
                quote! {
                    None
                }
            } else {
                quote! {
                    self.following_span
                }
            };

            let mut next_field_index = index + 1;
            while let Some(next_field) = fields.get(next_field_index) {
                let next_field_type_def = next_field.type_def(schema);
                if let Some(next_following_node_tmp) =
                    generate_next_following_node(next_field, next_field_type_def, schema)
                {
                    following_span = if next_field_type_def.is_option()
                        || next_field_type_def.is_vec()
                    {
                        let or_else_following_nodes = build_following_node_chain_until_non_option(
                            &fields[next_field_index + 1..],
                            should_not_have_following_node,
                            schema,
                        );
                        quote! {
                            #next_following_node_tmp #or_else_following_nodes
                        }
                    } else {
                        next_following_node_tmp
                    };
                    break;
                }

                next_field_index += 1;
            }

            if is_option {
                quote! {
                    let following_span = #following_span;
                    self.allocator.alloc(self.inner.#field_name.as_ref().map(|inner| AstNode {
                        inner: #inner_access,
                        allocator: self.allocator,
                        parent: #parent_expr,
                        following_span
                    })).as_ref()
                }
            } else {
                quote! {
                    let following_span = #following_span;
                    self.allocator.alloc(AstNode {
                        inner: #field_access,
                        allocator: self.allocator,
                        parent: #parent_expr,
                        following_span
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
                    self.following_span,
                )
                .fmt(f);
            }
        }
    }
}

fn generate_next_following_node(
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

    let following_span = match next_field_type_def {
        TypeDef::Box(_) | TypeDef::Struct(_) | TypeDef::Enum(_) => {
            quote! { Some(#next_field_accessor.span()) }
        }
        TypeDef::Option(option_def) => {
            let inner_type = option_def.inner_type(schema);
            let inner_type_call = if inner_type.is_box() {
                quote! { as_deref() }
            } else {
                quote! { as_ref() }
            };
            quote! { #next_field_accessor.#inner_type_call.map(GetSpan::span) }
        }
        TypeDef::Vec(_) => {
            quote! { #next_field_accessor.first().map(GetSpan::span) }
        }
        _ => return None,
    };

    Some(following_span)
}

fn build_following_node_chain_until_non_option(
    fields: &[FieldDef],
    should_not_have_following_node: bool,
    schema: &Schema,
) -> TokenStream {
    let mut result = TokenStream::new();

    for field in fields {
        let field_type_def = field.type_def(schema);
        if let Some(following_span) = generate_next_following_node(field, field_type_def, schema) {
            result = quote! {
                #result.or_else(|| #following_span)
            };
            if !field_type_def.is_option() && !field_type_def.is_vec() {
                return result;
            }
        }
    }

    if should_not_have_following_node {
        result
    } else {
        quote! { #result.or(self.following_span) }
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
                    following_span: self.following_span,
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
                    following_span: self.following_span,
                }))
            }
        } else {
            quote! {
                return self.allocator.alloc(AstNode {
                    inner: it.#to_fn_ident(),
                    parent,
                    allocator: self.allocator,
                    following_span: self.following_span,
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
