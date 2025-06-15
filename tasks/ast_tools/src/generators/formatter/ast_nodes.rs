//! Generator for `oxc_formatter`.
//! Generates the `AstNodes` and `AstNode` types.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    Codegen, Generator,
    generators::define_generator,
    output::{Output, output_path},
    schema::{Def, EnumDef, Schema, StructDef, TypeDef},
};

pub fn get_node_type(ty: &TokenStream) -> TokenStream {
    quote! { AstNode<'a, #ty> }
}

const FORMATTER_CRATE_PATH: &str = "crates/oxc_formatter";

pub struct FormatterAstNodesGenerator;

define_generator!(FormatterAstNodesGenerator);

impl Generator for FormatterAstNodesGenerator {
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let impls = schema
            .types
            .iter()
            .filter_map(|type_def| match type_def {
                TypeDef::Struct(struct_def)
                    if struct_def.visit.has_visitor() && !struct_def.builder.skip =>
                {
                    Some(generate_struct_impls(struct_def, schema))
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

        let following_node_name_with_param = schema
            .types
            .iter()
            .filter_map(|type_def| {
                let (name, param) = match type_def {
                    TypeDef::Struct(struct_def) if struct_def.visit.has_visitor() => {
                        let name = struct_def.ident();
                        let param = struct_def.ty_with_lifetime(schema, false);
                        (name, param)
                    }
                    TypeDef::Enum(enum_def) if enum_def.visit.has_visitor() => {
                        let name = enum_def.ident();
                        let param = enum_def.ty_with_lifetime(schema, false);
                        (name, param)
                    }
                    _ => return None,
                };
                Some((name, param))
            })
            .collect::<Vec<_>>();

        let following_node_variants = following_node_name_with_param.iter().map(|(name, param)| {
            quote! { #name(&'a #param), }
        });

        let following_node_span_match_arms =
            following_node_name_with_param.iter().map(|(name, _)| {
                quote! { Self::#name(n) => n.span(), }
            });

        let dummy_variant = quote! {
            Self::Dummy() => panic!("Should never be called on a dummy node"),
        };

        let span_match_arms = ast_nodes_names.iter().map(|(name, _)| {
            quote! { Self::#name(n) => n.span(), }
        });

        let parent_match_arms = ast_nodes_names.iter().map(|(name, _)| {
            quote! { Self::#name(n) => n.parent, }
        });

        let following_node_match_arms = ast_nodes_names.iter().map(|(name, _)| {
            quote! { Self::#name(n) => n.following_node.as_ref(), }
        });

        let ast_nodes_debug_names = ast_nodes_names.iter().map(|(name, _)| {
            let debug_name = name.to_string();
            quote! { Self::#name(_) => #debug_name, }
        });

        let transmute_self = quote! {
            #[inline]
            pub(super) fn transmute_self<'a, T>(s: &AstNode<'a, T>) -> &'a AstNode<'a, T> {
                /// * SAFETY: `s` is already allocated in Arena, so transmute from `&` to `&'a` is safe.
                unsafe { transmute(s) }
            }
        };

        let ast_node_ast_nodes_impls = ast_node_and_ast_nodes_impls();
        let ast_node_iterator_impls = ast_node_iterator_impls(schema);

        let output = quote! {
            #![expect(
                clippy::elidable_lifetime_names
            )]

            use std::{mem::transmute, ops::Deref, fmt};
            ///@@line_break
            use oxc_ast::ast::*;
            use oxc_allocator::{Allocator, Vec, Box};
            use oxc_span::GetSpan;

            ///@@line_break
            use crate::{
                formatter::{
                    Buffer, Format, FormatResult, Formatter,
                    trivia::{format_leading_comments, format_trailing_comments},
                },
                parentheses::NeedsParentheses,
                write::FormatWrite,
            };

            ///@@line_break
            #transmute_self

            ///@@line_break
            pub enum AstNodes<'a> {
                Dummy(),
                #(#ast_nodes_variants)*
            }

            #[derive(Clone)]
            pub enum FollowingNode<'a> {
                #(#following_node_variants)*
            }

            impl FollowingNode<'_> {
                ///@@line_break
                pub fn span(&self) -> oxc_span::Span {
                    match self {
                        #(#following_node_span_match_arms)*
                    }
                }
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
                pub fn following_node(&self) -> Option<&FollowingNode<'a>> {
                    match self {
                        Self::Dummy() => None,
                        #(#following_node_match_arms)*
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

            ///@@line_break
            pub struct AstNode<'a, T> {
                pub(super) inner: &'a T,
                pub parent: &'a AstNodes<'a>,
                pub(super) allocator: &'a Allocator,
                pub(super) following_node: Option<FollowingNode<'a>>,
            }

            #ast_node_ast_nodes_impls

            #impls

            #ast_node_iterator_impls
        };

        Output::Rust { path: output_path(FORMATTER_CRATE_PATH, "ast_nodes.rs"), tokens: output }
    }
}

fn generate_struct_impls(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    let type_ty = struct_def.ty(schema);
    let has_kind = struct_def.kind.has_kind;
    let struct_name = struct_def.ident();

    let offset_fields = struct_def
        .fields
        .iter()
        .filter_map(|field| {
            let field_type = field.type_def(schema);
            // TODO: optimize this check
            match field_type.maybe_inner_type(schema).unwrap_or(field_type) {
                TypeDef::Struct(struct_def)
                    if struct_def.visit.has_visitor() && struct_def.name != "Span" => {}
                TypeDef::Enum(enum_def) if enum_def.visit.has_visitor() => {}
                _ => return None,
            }

            Some((
                field,
                format_ident!(
                    "{}_OFFSET_{}",
                    struct_def.snake_name().to_uppercase(),
                    field.camel_name().to_string().to_uppercase()
                ),
            ))
        })
        .collect::<Vec<_>>();

    let mut offset_fields_iter = offset_fields.iter();
    offset_fields_iter.next();

    let methods = struct_def
        .fields
        .iter()
        .filter_map(|field| {
            let field_type_def = field.type_def(schema);
            let is_option = field_type_def.is_option();
            let (original_field_type, is_box) = if let TypeDef::Box(box_def) =
                field_type_def.as_option().map_or(field_type_def, |opt| opt.inner_type(schema))
            {
                (box_def.inner_type(schema), true)
            } else {
                (
                    field_type_def.as_option().map_or(field_type_def, |opt| opt.inner_type(schema)),
                    false,
                )
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
                TypeDef::Vec(vec) => (
                    vec.inner_type(schema).as_struct().is_some_and(|s| !s.visit.has_visitor()),
                    false,
                ),
                TypeDef::Cell(_) => return None,
                TypeDef::Option(_) | TypeDef::Box(_) => {
                    unreachable!("Option/Box should have been unwrapped")
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

                let following_node = if let Some((next_offset_field, next_offset_name)) = offset_fields_iter.next()
                    {
                        let next_field_type = next_offset_field.type_def(schema);
                        let next_field_type_token = next_field_type.ty(schema);
                        let as_type =  quote! {
                            {
                                let ptr = self.inner as *const _ as *const u8;
                                unsafe { &*(ptr.add(#next_offset_name) as *const #next_field_type_token) }
                            }
                        };
                        match next_field_type {
                            TypeDef::Option(option_def) => {
                                let inner_ident = option_def.inner_type(schema).ident();
                                quote!{ (#as_type).as_ref().map(FollowingNode::#inner_ident) }
                            },
                            TypeDef::Box(box_def) => {
                                let inner_ident = box_def.inner_type(schema).ident();
                                quote! { Some(FollowingNode::#inner_ident(#as_type.as_ref())) }
                            },
                            TypeDef::Vec(vec_def) => {
                                let inner_ident = vec_def.inner_type(schema).ident();
                                quote! { (#as_type).first().as_ref().copied().map(FollowingNode::#inner_ident) }
                            },
                            _ => {
                                let next_field_ident = next_field_type.ident();
                                quote! { Some(FollowingNode::#next_field_ident(#as_type)) }
                            }
                        }
                    } else {
                        quote! {
                            self.following_node.clone()
                        }
                    };

                if is_option {
                    quote! {
                        let following_node = #following_node;
                        self.allocator.alloc(self.inner.#field_name.as_ref().map(|inner| AstNode {
                            inner: #inner_access,
                            allocator: self.allocator,
                            parent: #parent_expr,
                            following_node
                        })).as_ref()
                    }
                } else {
                    quote! {
                        let following_node = #following_node;
                        self.allocator.alloc(AstNode {
                            inner: #field_access,
                            allocator: self.allocator,
                            parent: #parent_expr,
                            following_node
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
        })
        .collect::<TokenStream>();

    let offset_match_arms = offset_fields.iter().enumerate().map(|(i, (field, offset_name))| {
        let next_offset_field = offset_fields.get(i + 1);

        let implementation = if let Some((next_offset_field, next_offset_name)) = next_offset_field
        {
            let next_field_type = next_offset_field.type_def(schema);
            let next_field_type_token = next_field_type.ty(schema);
            let as_type =  quote! { unsafe { &*(inner_ptr.add(#next_offset_name) as *const #next_field_type_token) } };
            match next_field_type {
                TypeDef::Option(option_def) => {
                    let inner_ident = option_def.inner_type(schema).ident();
                    quote!{ (#as_type).as_ref().map(FollowingNode::#inner_ident) }
                },
                TypeDef::Box(box_def) => {
                    let inner_ident = box_def.inner_type(schema).ident();
                     quote! { Some(FollowingNode::#inner_ident(#as_type.as_ref())) }
                },
                TypeDef::Vec(vec_def) => {
                    let inner_ident = vec_def.inner_type(schema).ident();
                    quote! { (#as_type).first().as_ref().copied().map(FollowingNode::#inner_ident) }
                },
                _ => {
                    let next_field_ident = next_field_type.ident();
                    quote! { Some(FollowingNode::#next_field_ident(#as_type)) }
                }
            }
        } else {
            quote! {
                None
            }
        };

        quote! {
            #offset_name => #implementation,
        }
    });

    let handle_vec_field = offset_fields
        .iter()
        .filter_map(|(field, _)| {
            if let TypeDef::Vec(vec_def) = field.type_def(schema) {
                let inner_type = vec_def.inner_type(schema);
                dbg!(inner_type.ident(), struct_def.ident());
                if inner_type.ident() == struct_def.ident() {
                    Some(quote! { panic() })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .last();

    let implementation = if offset_fields.is_empty() {
        quote! { None }
    } else {
        quote! {
            let inner_ptr = unsafe { self.parent.inner_ptr() };
            let offset = unsafe { (self.inner as *const _ as *const u8).offset_from_unsigned(inner_ptr) };
            match offset {
                #(#offset_match_arms)*
                _ => {
                    #handle_vec_field
                    // This field is inside a Vec
                }
            }
        }
    };

    let next_field_fn = quote! {
        ///@@line_break
        pub fn next_field(&self) -> Option<FollowingNode<'a>> {
            #implementation
        }
    };

    // 生成字段偏移量常量
    let field_offsets = offset_fields.iter().filter_map(|(field, offset_name)| {
        let field_name = field.ident();
        let offset = quote! {
            const #offset_name: usize = std::mem::offset_of!(#struct_name, #field_name);
        };
        Some(offset)
    });

    quote! {
        ///@@line_break
        #(#field_offsets)*

        ///@@line_break
        impl<'a> AstNode<'a, #type_ty> {
            #methods
            //@@line_break
            // #next_field_fn
        }

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
                    following_node: self.following_node.clone(),
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
                    following_node: self.following_node.clone(),
                }))
            }
        } else {
            quote! {
                return self.allocator.alloc(AstNode {
                    inner: it.#to_fn_ident(),
                    parent,
                    allocator: self.allocator,
                    following_node: self.following_node.clone(),
                }).as_ast_nodes();
            }
        };
        quote! { it @ #match_ident!(#enum_ident) => { #implementation }, }
    });

    let node_type = get_node_type(&type_ty);

    let as_ast_nodes_fn = quote! {
        ///@@line_break
        impl<'a> #node_type {
            #[inline]
            pub fn as_ast_nodes(&self) -> &AstNodes<'a> {
                #parent_decl
                let node = match self.inner {
                    #(#variant_match_arms)*
                    #(#inherits_match_arms)*
                };
                self.allocator.alloc(node)
            }
        }
    };

    let impl_get_span = quote! {
        ///@@line_break
        impl<'a> GetSpan for #node_type {
            #[inline]
            fn span(&self) -> oxc_span::Span {
                self.inner.span()
            }
        }
    };

    quote! {
        #as_ast_nodes_fn
        #impl_get_span
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

fn ast_node_and_ast_nodes_impls() -> TokenStream {
    quote! {
        ///@@line_break
        impl<'a, T: fmt::Debug> fmt::Debug for AstNode<'a, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("AstNode")
                    .field("inner", &self.inner)
                    .field("parent", &self.parent.debug_name())
                    .finish_non_exhaustive()
            }
        }

        ///@@line_break
        impl<'a, T> Deref for AstNode<'a, T> {
            type Target = T;

            fn deref(&self) -> &'a Self::Target {
                self.inner
            }
        }

        ///@@line_break
        impl<'a, T> AsRef<T> for AstNode<'a, T> {
            fn as_ref(&self) -> &'a T {
                self.inner
            }
        }

        ///@@line_break
        impl<'a>  AstNode<'a, Program<'a>> {
            pub fn new(inner: &'a Program<'a>, parent: &'a AstNodes<'a>, allocator: &'a Allocator) -> Self {
                AstNode { inner, parent, allocator, following_node: None }
            }
        }

        ///@@line_break
        impl<'a, T> AstNode<'a, Option<T>> {
            pub fn as_ref(&self) -> Option<&'a AstNode<'a, T>> {
                self.allocator
                    .alloc(self.inner.as_ref().map(|inner| AstNode {
                        inner,
                        parent: self.parent,
                        allocator: self.allocator,
                        following_node: self.following_node.clone(),
                    }))
                    .as_ref()
            }
        }
    }
}

fn ast_node_iterator_impls(schema: &Schema) -> TokenStream {
    let types_used_in_vec = schema
        .types
        .iter()
        .filter_map(|type_def| {
            if let TypeDef::Struct(struct_def) = type_def {
                if !struct_def.visit.has_visitor() {
                    return None;
                }
                Some(struct_def.fields.iter().filter_map(|field| {
                    let mut field_type = field.type_def(schema);
                    if field_type.is_option() {
                        field_type = field_type.as_option().unwrap().inner_type(schema);
                    }
                    if let TypeDef::Vec(vec_def) = field_type {
                        let inner_type_def = vec_def
                            .maybe_inner_type(schema)
                            .unwrap_or_else(|| vec_def.inner_type(schema));

                        match inner_type_def {
                            TypeDef::Struct(inner_struct_def)
                                if !inner_struct_def.visit.has_visitor() =>
                            {
                                None
                            }
                            TypeDef::Enum(inner_enum_def)
                                if !inner_enum_def.visit.has_visitor() =>
                            {
                                None
                            }
                            _ => Some(inner_type_def.id()),
                        }
                    } else {
                        None
                    }
                }))
            } else {
                None
            }
        })
        .flatten()
        .sorted()
        .dedup()
        .collect::<Vec<_>>();

    let impls = types_used_in_vec.iter().map(|type_id| {
        let type_def = &schema.types[*type_id];

        let (type_ident, is_option) = if let Some(option_def) = type_def.as_option() {
            (option_def.inner_type(schema).ident(), true)
        } else {
            (type_def.ident(), false)
        };

        let next_to_following_node = if is_option {
            quote! { .map(|next| next.as_ref().map(FollowingNode::#type_ident)).unwrap_or_default() }
        } else {
            quote! { .map(|t| FollowingNode::#type_ident(t)) }
        };

        let type_ty = type_def.ty(schema);
        quote! {
            impl<'a> AstNode<'a, Vec<'a, #type_ty>> {
                pub fn iter(&self) -> AstNodeIterator<'a, #type_ty> {
                    AstNodeIterator {
                        inner: self.inner.iter().peekable(),
                        parent: self.parent,
                        following_node: self.following_node.clone(),
                        allocator: self.allocator
                    }
                }

                pub fn first(&self) -> Option<&'a AstNode<'a, #type_ty>> {
                    let mut inner_iter = self.inner.iter();
                    self.allocator
                        .alloc(inner_iter.next().map(|inner| AstNode {
                            inner,
                            parent: self.parent,
                            allocator: self.allocator,
                            following_node: inner_iter
                                .next()
                                #next_to_following_node
                                .or_else(|| self.following_node.clone()),
                        }))
                        .as_ref()
                }

                pub fn last(&self) -> Option<&'a AstNode<'a, #type_ty>> {
                    self.allocator
                        .alloc(self.inner.last().map(|inner| AstNode {
                            inner,
                            parent: self.parent,
                            allocator: self.allocator,
                            following_node: self.following_node.clone(),
                        }))
                        .as_ref()
                }
            }

            impl<'a> Iterator for AstNodeIterator<'a, #type_ty> {
                type Item = &'a AstNode<'a, #type_ty>;
                fn next(&mut self) -> Option<Self::Item> {
                    let allocator = self.allocator;
                    allocator
                        .alloc(self.inner.next().map(|inner| AstNode {
                            parent: self.parent,
                            inner,
                            allocator,
                            following_node: self.inner.peek()
                            #next_to_following_node
                            .or_else(|| self.following_node.clone()),
                        }))
                        .as_ref()
                }
            }

            impl<'a> IntoIterator for &AstNode<'a, Vec<'a, #type_ty>>
            {
                type Item = &'a AstNode<'a, #type_ty>;
                type IntoIter = AstNodeIterator<'a, #type_ty>;
                fn into_iter(self) -> Self::IntoIter {
                    AstNodeIterator::<#type_ty> {
                        inner: self.inner.iter().peekable(),
                        parent: self.parent,
                        following_node: self.following_node.clone(),
                        allocator: self.allocator,
                    }
                }
            }
        }
    });

    quote! {
        pub struct AstNodeIterator<'a, T> {
            inner: std::iter::Peekable<std::slice::Iter<'a, T>>,
            parent: &'a AstNodes<'a>,
            following_node: Option<FollowingNode<'a>>,
            allocator: &'a Allocator,
        }

        #(#impls)*
    }
}
