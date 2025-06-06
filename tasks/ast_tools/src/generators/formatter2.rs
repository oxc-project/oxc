//! Generator for `oxc_formatter`.
//!

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    Codegen, Generator,
    output::{Output, output_path},
    schema::{Def, Schema, TypeDef},
};

use super::define_generator;

const FORMATTER_CRATE_PATH: &str = "crates/oxc_formatter";

pub struct FormatterFormatGenerator2;

define_generator!(FormatterFormatGenerator2);

impl Generator for FormatterFormatGenerator2 {
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let impls = schema
            .types
            .iter()
            .filter(|type_def| match type_def {
                TypeDef::Struct(struct_def) => {
                    struct_def.visit.has_visitor() && !struct_def.builder.skip
                }
                TypeDef::Enum(enum_def) => enum_def.visit.has_visitor(),
                _ => false,
            })
            .map(|type_def| implementation(type_def, schema))
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
                #name(&'a AstNode<'a, 'b, #name #lifetime>),
            }
        });

        let dummy_variant = quote! {
            Self::DUMMY() => panic!("Should never be called on a dummy node"),
        };

        let span_match_arms = ast_nodes_names.iter().map(|(name, _)| {
            quote! {
                Self::#name(n) => n.inner.span(),
            }
        });

        let parent_match_arms = ast_nodes_names.iter().map(|(name, _)| {
            quote! {
                Self::#name(n) => n.parent(),
            }
        });

        let output = quote! {
            #![allow(clippy::undocumented_unsafe_blocks)]

            use std::mem::transmute;
            ///@@line_break
            use oxc_ast::{AstKind, ast::*};
            use oxc_allocator::{Allocator, Vec};
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
            pub enum AstNodes<'a, 'b> {
                DUMMY(),
                #(#ast_nodes_variants)*
            }

            impl <'a, 'b> AstNodes<'a, 'b> {
                pub fn span(&self) -> Span {
                    match self {
                        #dummy_variant
                        #(#span_match_arms)*
                    }
                }

                pub fn parent(&self) -> &'a Self {
                    match self {
                        #dummy_variant
                        #(#parent_match_arms)*
                    }
                }
            }

            ///@@line_break
            pub struct AstNode<'a, 'b, T> {
                inner: &'b T,
                parent: &'a AstNodes<'a, 'b>,
                allocator: &'a Allocator,
            }

            ///@@line_break
            impl<'a,'b, T>  AstNode<'a, 'b, T> {
                pub fn new(inner: &'b T, parent: &'a AstNodes<'a, 'b>, allocator: &'a Allocator) -> Self {
                    AstNode { inner, parent, allocator }
                }

                pub fn inner(&self) -> &'b T {
                    self.inner
                }

                pub fn parent(&self) -> &'a AstNodes<'a, 'b> {
                    self.parent
                }
            }

            ///@@line_break
            impl<'a, 'b, T> AstNode<'a, 'b, Option<T>> {
                pub fn as_ref(&self) -> Option<&'a AstNode<'a, 'b, T>> {
                    self.allocator
                        .alloc(self.inner.as_ref().map(|inner| AstNode {
                            inner,
                            parent: self.parent,
                            allocator: self.allocator,
                        }))
                        .as_ref()
                }
            }


            ///@@line_break
            impl<'a, 'b, T> AstNode<'a, 'b, Vec<'a, T>> {

                ///@@line_break
                pub fn is_empty(&self) -> bool {
                    self.inner.is_empty()
                }

                ///@@line_break
                pub fn len(&self) -> usize {
                    self.inner.len()
                }

                ///@@line_break
                pub fn iter(&self) -> AstNodeIterator<'a, 'b, T> {
                    AstNodeIterator { inner: self.inner.iter(), parent: self.parent, allocator: self.allocator }
                }

                ///@@line_break
                pub fn first(&self) -> Option<&'a AstNode<'a, 'b, T>> {
                    self.allocator
                        .alloc(self.inner.first().map(|inner| AstNode {
                            inner,
                            parent: self.parent,
                            allocator: self.allocator,
                        }))
                        .as_ref()
                }

                ///@@line_break
                pub fn last(&self) -> Option<&'a AstNode<'a, 'b, T>> {
                    self.allocator
                        .alloc(self.inner.last().map(|inner| AstNode {
                            inner,
                            parent: self.parent,
                            allocator: self.allocator,
                        }))
                        .as_ref()
                }
            }


            ///@@line_break
            pub struct AstNodeIterator<'a, 'b, T> {
                inner: std::slice::Iter<'b, T>,
                parent: &'a AstNodes<'a, 'b>,
                allocator: &'a Allocator,
            }

            ///@@line_break
            impl<'a, 'b, T> Iterator for AstNodeIterator<'a, 'b, T> {
                type Item = &'a AstNode<'a, 'b, T>;
                fn next(&mut self) -> Option<Self::Item> {
                    let allocator = self.allocator;
                    self.allocator
                        .alloc(self.inner.next().map(|inner| AstNode { parent: self.parent, inner, allocator }))
                        .as_ref()
                }
            }

            ///@@line_break
            impl<'a, 'b, T> IntoIterator for &AstNode<'a, 'b, Vec<'a, T>> {
                type Item = &'a AstNode<'a, 'b, T>;
                type IntoIter = AstNodeIterator<'a, 'b, T>;
                fn into_iter(self) -> Self::IntoIter {
                    AstNodeIterator::<T> { inner: self.inner.iter(), parent: self.parent, allocator: self.allocator }
                }
            }


            ///@@line_break
            #impls
        };

        Output::Rust { path: output_path(FORMATTER_CRATE_PATH, "ast_nodes.rs"), tokens: output }
    }
}

fn implementation(type_def: &TypeDef, schema: &Schema) -> TokenStream {
    let type_ty = type_def.ty(schema);

    if let Some(struct_def) = type_def.as_struct() {
        let has_kind = struct_def.kind.has_kind;
        let mut functions = quote! {};
        let struct_name = struct_def.ident();
        functions.extend(struct_def.fields.iter().filter_map(|field| {
            let field_type = field.type_def(schema);
            let is_option = field_type.is_option();
            let field_type = field_type
                .as_option()
                .map(|option_def| option_def.inner_type(schema))
                .unwrap_or(field_type);
            let is_box = field_type.is_box();
            let mut is_reference = true;

            let field_name = &field.ident();
            let mut is_not_ast_node = false;

            let return_type = match field_type {
                TypeDef::Struct(struct_def) => {
                    is_reference = !struct_def.derives.contains(&String::from("Copy"));
                    is_not_ast_node = !is_reference || !struct_def.visit.has_visitor();

                    struct_def.ty(schema)
                }
                TypeDef::Box(box_def) => {
                    let Some(inner_type) = box_def.inner_type(schema).as_struct() else {
                        unreachable!()
                    };

                    inner_type.ty(schema)
                }
                TypeDef::Vec(vec_def) => vec_def.ty(schema),
                TypeDef::Enum(enum_def) => {
                    is_not_ast_node = !enum_def.visit.has_visitor();
                    is_reference = !enum_def.derives.contains(&String::from("Copy"));

                    enum_def.ty(schema)
                }
                TypeDef::Primitive(primitive_def) => {
                    is_not_ast_node = true;
                    is_reference = false;

                    primitive_def.ty(schema)
                }
                TypeDef::Option(_) => {
                    unreachable!()
                }
                _ => return None,
            };

            let parent = if has_kind {
                quote! {
                    self.allocator.alloc(AstNodes::#struct_name(unsafe { transmute(self) }))
                }
            } else {
                quote! { self.parent }
            };

            let reference_symbol = if is_reference {
                quote! { & }
            } else {
                quote! {}
            };

            let body = if is_not_ast_node {
                if is_option && is_reference {
                    quote! {
                        self.inner.#field_name.as_ref()
                    }
                } else {
                    quote! {
                        #reference_symbol self.inner.#field_name
                    }
                }
            } else if is_option {
                let inner = if is_box {
                    quote! { inner.as_ref() }
                } else {
                    quote! { inner }
                };
                quote! {
                    self.allocator.alloc(self.inner.#field_name.as_ref().map(|inner| AstNode {
                        inner: #inner,
                        allocator: self.allocator,
                        parent: #parent,
                    })).as_ref()
                }
            } else {
                let inner = if is_box {
                    quote! {
                        self.inner.#field_name.as_ref()
                    }
                } else {
                    quote! {
                        &self.inner.#field_name
                    }
                };
                quote! {
                    self.allocator.alloc(AstNode {
                        inner: #inner,
                        allocator: self.allocator,
                        parent: #parent,
                    })
                }
            };

            let return_type = if is_not_ast_node {
                quote! { #reference_symbol #return_type }
            } else {
                quote! { &AstNode<'a, 'b, #return_type> }
            };

            let return_type = if is_option {
                quote! { Option<#return_type> }
            } else {
                return_type
            };

            Some(quote! {
                ///@@line_break
                pub fn #field_name(&self) -> #return_type {
                    #body
                }
            })
        }));

        return quote! {
            impl<'a, 'b> AstNode<'a, 'b, #type_ty> {
                #functions
            }
        };
    }

    let enum_ident = type_def.ident();
    let enum_def = type_def.as_enum().unwrap();

    let parent = if enum_def.kind.has_kind {
        quote! {
            let parent = self.allocator.alloc(AstNodes::#enum_ident(unsafe { transmute(self) }))
        }
    } else {
        quote! { let parent = self.parent }
    };

    let variant_match_arms = enum_def.variants.iter().filter_map(|variant| {
        let variant_name = &variant.ident();
        let field_type = variant.field_type(schema).unwrap();
        let is_box = field_type.is_box();
        let node_type = field_type
            .maybe_inner_type(schema)
            .map(|inner_type| inner_type.ident())
            .unwrap_or_else(|| field_type.ident());
        let inner = if is_box {
            quote! { s.as_ref() }
        } else {
            quote! { s }
        };
        let implementation = if has_kind(&field_type, schema) {
            quote! {
                AstNodes::#node_type(self.allocator.alloc(AstNode {
                    inner: #inner,
                    parent,
                    allocator: self.allocator,
                }))
            }
        } else {
            quote! {
                panic!("No Node for enum variant yet, please see `tasks/ast_tools/src/generators/ast_kind.rs`")
            }
        };

        Some(quote! {

            #enum_ident::#variant_name(s) => {
                #implementation
            },
        })
    });

    let inherits_match_arms = enum_def.inherits_types(schema).map(|inherited_type| {
        let inherited_enum = inherited_type.as_enum().unwrap();
        let inherited_enum_inner_type = inherited_enum
            .maybe_inner_type(schema)
            .map(|t| t.ident())
            .unwrap_or_else(|| inherited_enum.ident());

        let inherits_snake_name = inherited_enum.snake_name();
        let match_ident = format_ident!("match_{inherits_snake_name}");

        let to_fn_ident = format_ident!("to_{inherits_snake_name}");

        let implementation = if inherited_enum.kind.has_kind {
            quote! {
                AstNodes::#inherited_enum_inner_type(self.allocator.alloc(AstNode {
                    inner: it.#to_fn_ident(),
                    parent,
                    allocator: self.allocator,
                }))
            }
        } else {
            quote! {
                    return self
                        .allocator
                        .alloc(AstNode {
                            inner: it.#to_fn_ident(),
                            parent,
                            allocator: self.allocator,
                        })
                        .as_ast_nodes();
            }
        };

        let match_arm = quote! {
            it @ #match_ident!(#enum_ident) => {
                #implementation
            },
        };

        match_arm
    });

    let node_type = quote! { AstNode<'a, 'b, #type_ty> };

    // Enum
    let as_ast_nodes_fn = quote! {
        ///@@line_break
        impl<'a, 'b> #node_type {
            pub fn as_ast_nodes(&self) -> &AstNodes<'a, 'b> {
                #parent;
                let node = match self.inner {
                    #(#variant_match_arms)*
                    #(#inherits_match_arms)*
                };
                self.allocator.alloc(node)
            }
        }
    };

    let variant_match_arms = enum_def.variants.iter().filter_map(|variant| {
        let variant_name = &variant.ident();
        let field_type = variant.field_type(schema).unwrap();
        let node_type = field_type
            .maybe_inner_type(schema)
            .map(|inner_type| inner_type.ident())
            .unwrap_or_else(|| field_type.ident());

        Some(quote! {
            #enum_ident::#variant_name(s) => {
                AstNode::<'a, 'b, #node_type> {
                    inner: s,
                    parent,
                    allocator: self.allocator,
                }.fmt(f)
            },
        })
    });

    let inherits_match_arms = enum_def.inherits_types(schema).map(|inherits_type| {
        let inherits_type = inherits_type.as_enum().unwrap();
        let inherits_inner_type = inherits_type
            .maybe_inner_type(schema)
            .map(|t| t.ident())
            .unwrap_or_else(|| inherits_type.ident());

        let inherits_snake_name = inherits_type.snake_name();
        let match_ident = format_ident!("match_{inherits_snake_name}");

        let to_fn_ident = format_ident!("to_{inherits_snake_name}");
        let match_arm = quote! {
            it @ #match_ident!(#enum_ident) => {
                AstNode::<'a, 'b, #inherits_inner_type> {
                    inner: it.#to_fn_ident(),
                    parent,
                    allocator: self.allocator,
                }.fmt(f)
            },
        };

        match_arm
    });

    // let get_child_func = quote! {
    //     ///@@line_break
    //     impl<'a, 'b> #node_type{
    //         fn get_child<T>(&self, inner: &'b T) -> Option<&AstNode<'a, 'b, T>> {
    //             #parent;
    //             self.allocator.alloc(AstNode {
    //                 inner,
    //                 parent,
    //                 allocator: self.allocator,
    //             })
    //         }
    //     }
    // };

    let impl_format_write = quote! {
        impl<'a, 'b> FormatWrite<'a> for #node_type {
            fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
                #parent;
                match self.inner {
                    #(#variant_match_arms)*
                    #(#inherits_match_arms)*
                }
            }
        }
    };

    let impl_get_span = quote! {
        impl<'a, 'b> GetSpan for #node_type {
            fn span(&self) -> oxc_span::Span {
                self.inner.span()
            }
        }
    };

    quote! {
        // #get_child_func
        #as_ast_nodes_fn
        #impl_format_write
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
