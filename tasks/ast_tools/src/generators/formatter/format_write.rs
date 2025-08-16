//! Generator for `oxc_formatter`.
//! This generator implements the `FormatWrite` trait for AST nodes that are enums with visitors.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    Codegen, Generator,
    generators::{define_generator, formatter::ast_nodes::get_node_type},
    output::{Output, output_path},
    schema::{Def, Schema, TypeDef},
};

const NEEDS_PARENTHESES: &[&str] =
    &["AssignmentTarget", "SimpleAssignmentTarget", "AssignmentTargetPattern"];

const FORMATTER_CRATE_PATH: &str = "crates/oxc_formatter";

pub struct FormatterFormatWriteGenerator;

define_generator!(FormatterFormatWriteGenerator);

impl Generator for FormatterFormatWriteGenerator {
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let impls = schema
            .types
            .iter()
            .filter(|type_def| match type_def {
                TypeDef::Enum(enum_def) => enum_def.visit.has_visitor(),
                _ => false,
            })
            .map(|type_def| implementation(type_def, schema))
            .collect::<TokenStream>();

        let output = quote! {
            #![expect(
                clippy::match_same_arms,
            )]

            use oxc_ast::ast::*;
            ///@@line_break
            use crate::{
                formatter::{
                    Buffer, Format, FormatResult, Formatter,
                    trivia::{format_leading_comments, format_trailing_comments},
                },
                parentheses::NeedsParentheses,
                generated::ast_nodes::{AstNode, AstNodes, transmute_self},
                write::FormatWrite,
            };

            #impls
        };

        Output::Rust { path: output_path(FORMATTER_CRATE_PATH, "format_write.rs"), tokens: output }
    }
}

fn implementation(type_def: &TypeDef, schema: &Schema) -> TokenStream {
    let enum_def = type_def.as_enum().unwrap();
    let enum_ident = enum_def.ident();
    let enum_ty = enum_def.ty(schema);

    let variant_match_arms = enum_def.variants.iter().map(|variant| {
        let variant_name = &variant.ident();
        let field_type = variant.field_type(schema).unwrap();
        let node_type =
            field_type.maybe_inner_type(schema).map_or_else(|| field_type.ident(), TypeDef::ident);

        Some(quote! {
            #enum_ident::#variant_name(inner) => {
                allocator.alloc(AstNode::<#node_type> {
                    inner,
                    parent,
                    allocator,
                    following_node: self.following_node,
                }).fmt(f)
            },
        })
    });

    let inherits_match_arms = enum_def.inherits_types(schema).map(|inherits_type| {
        let inherits_type = inherits_type.as_enum().unwrap();
        let inherits_inner_type = inherits_type
            .maybe_inner_type(schema)
            .map_or_else(|| inherits_type.ident(), TypeDef::ident);

        let inherits_snake_name = inherits_type.snake_name();
        let match_ident = format_ident!("match_{inherits_snake_name}");

        let to_fn_ident = format_ident!("to_{inherits_snake_name}");
        let match_arm = quote! {
            it @ #match_ident!(#enum_ident) => {
                let inner = it.#to_fn_ident();
                allocator.alloc(AstNode::<'a, #inherits_inner_type> {
                    inner,
                    parent,
                    allocator,
                    following_node: self.following_node,
                }).fmt(f)
            },
        };

        match_arm
    });

    let parent = if enum_def.kind.has_kind {
        quote! {
            let parent = allocator.alloc(AstNodes::#enum_ident(transmute_self(self)))
        }
    } else {
        quote! { let parent = self.parent }
    };
    let node_type = get_node_type(&enum_ty);

    let inner_match = quote! {
        match self.inner {
            #(#variant_match_arms)*
            #(#inherits_match_arms)*
        }
    };
    let needs_parentheses = NEEDS_PARENTHESES.contains(&enum_def.name.as_str());

    let print_parentheses = |paren: &str| {
        if needs_parentheses {
            quote! {
                if needs_parentheses {
                    #paren.fmt(f)?;
                }
            }
        } else {
            quote!()
        }
    };

    let left_paren = print_parentheses("(");
    let right_paren = print_parentheses(")");

    let body = if needs_parentheses {
        quote! {
            let needs_parentheses = self.needs_parentheses(f);
            #left_paren;
            let result = #inner_match;
            #right_paren;
            result
        }
    } else {
        inner_match
    };

    quote! {
        ///@@line_break
        impl<'a> FormatWrite<'a> for #node_type {
            #[inline]
            fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
                let allocator = self.allocator;
                #parent;
                #body
            }
        }
    }
}
