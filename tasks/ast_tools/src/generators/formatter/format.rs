//! Generator for `oxc_formatter`.
//!

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    Codegen, Generator,
    generators::define_generator,
    output::{Output, output_path},
    schema::{Def, Schema, TypeDef},
};

const FORMATTER_CRATE_PATH: &str = "crates/oxc_formatter";

const NEEDS_PARENTHESES: &[&str] = &[
    "Class",
    "Function",
    "NumericLiteral",
    "SimpleAssignmentTarget",
    "StringLiteral",
    "TSTypeAssertion",
];

pub struct FormatterFormatGenerator;

define_generator!(FormatterFormatGenerator);

impl Generator for FormatterFormatGenerator {
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

        let output = quote! {
            use oxc_ast::ast::*;

            ///@@line_break
            use crate::{
                formatter::{
                    Buffer, Format, FormatResult, Formatter,
                    trivia::{format_leading_comments, format_trailing_comments},
                },
                parentheses::NeedsParentheses,
                generated::ast_nodes::AstNode,
                write::FormatWrite,
            };

            #impls
        };

        Output::Rust { path: output_path(FORMATTER_CRATE_PATH, "format.rs"), tokens: output }
    }
}

fn implementation(type_def: &TypeDef, schema: &Schema) -> TokenStream {
    let type_ty = type_def.ty(schema);
    let type_ty = quote! {
        AstNode::<'a, #type_ty>
    };

    let has_kind = match type_def {
        TypeDef::Struct(struct_def) => struct_def.kind.has_kind,
        TypeDef::Enum(enum_def) => enum_def.kind.has_kind,
        _ => unreachable!(),
    };

    if !has_kind {
        return quote! {
            ///@@line_break
            impl<'a> Format<'a> for #type_ty {
                fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
                    self.write(f)
                }
            }
        };
    }

    let leading_comments = if type_def.is_enum() {
        quote! {}
    } else {
        quote! {
            format_leading_comments(self.span().start).fmt(f)?;
        }
    };

    let trailing_comments = if type_def.is_enum() {
        quote! {}
    } else {
        quote! {
            format_trailing_comments(self.span().end).fmt(f)?;
        }
    };

    let type_def_name = type_def.name();
    let needs_parentheses =
        type_def_name.ends_with("Expression") || NEEDS_PARENTHESES.contains(&type_def_name);
    let needs_parentheses_before = if needs_parentheses {
        quote! {
            let needs_parentheses = self.needs_parentheses(f);
            if needs_parentheses {
                "(".fmt(f)?;
            }

        }
    } else {
        quote! {}
    };

    let needs_parentheses_after = if needs_parentheses {
        quote! {
            if needs_parentheses {
                ")".fmt(f)?;
            }

        }
    } else {
        quote! {}
    };

    let implementation = if needs_parentheses_before.is_empty() && trailing_comments.is_empty() {
        quote! {
            self.write(f)
        }
    } else {
        quote! {
            #leading_comments
            #needs_parentheses_before
            let result = self.write(f);
            #needs_parentheses_after
            #trailing_comments
            result
        }
    };

    quote! {
        ///@@line_break
        impl<'a> Format<'a> for #type_ty {
            fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
                #implementation
            }
        }
    }
}
