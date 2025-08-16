//! Generator for `oxc_formatter`.
//!

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    Codegen, Generator,
    generators::define_generator,
    output::{Output, output_path},
    schema::{Def, Schema, TypeDef},
};

const FORMATTER_CRATE_PATH: &str = "crates/oxc_formatter";

/// Based on the prettier printing comments algorithm, these nodes don't need to print comments.
const AST_NODE_WITHOUT_PRINTING_COMMENTS_LIST: &[&str] = &[
    "FormalParameters",
    "FunctionBody",
    "ClassBody",
    "CatchParameter",
    "CatchClause",
    "Decorator",
    // Manually prints it, because class's decorators can be appears before `export class Cls {}`.
    "ExportNamedDeclaration",
    "ExportDefaultDeclaration",
    "TSClassImplements",
];

const NEEDS_PARENTHESES: &[&str] = &[
    "Class",
    "Function",
    "NumericLiteral",
    "SimpleAssignmentTarget",
    "StringLiteral",
    "TSTypeAssertion",
];

const NEEDS_IMPLEMENTING_FMT_WITH_OPTIONS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "ArrowFunctionExpression" => "FormatJsArrowFunctionExpressionOptions",
    "Function" => "FormatFunctionOptions",
};

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

        let options = NEEDS_IMPLEMENTING_FMT_WITH_OPTIONS.values().map(|o| {
            let ident = format_ident!("{}", o);
            quote! { , #ident }
        });

        let output = quote! {
            use oxc_ast::ast::*;

            ///@@line_break
            use crate::{
                formatter::{
                    Buffer, Format, FormatResult, Formatter,
                    trivia::FormatTrailingComments,
                },
                parentheses::NeedsParentheses,
                generated::ast_nodes::{AstNode, SiblingNode},
                write::{FormatWrite #(#options)*},
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

    let is_program = type_def.as_struct().is_some_and(|s| s.name == "Program");
    let do_not_print_comment = AST_NODE_WITHOUT_PRINTING_COMMENTS_LIST.contains(&type_def.name());

    let leading_comments = if type_def.is_enum() || is_program || do_not_print_comment {
        quote! {}
    } else {
        quote! {
            self.format_leading_comments(f)?;
        }
    };

    let trailing_comments = if type_def.is_enum() || do_not_print_comment {
        quote! {}
    } else if is_program {
        quote! {
            FormatTrailingComments::Comments(f.context().comments().unprinted_comments()).fmt(f)?;
        }
    } else {
        quote! {
            self.format_trailing_comments(f)?;
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

    let generate_fmt_implementation = |has_options: bool| {
        let write_implementation = if has_options {
            quote! {
                self.write_with_options(options, f)
            }
        } else {
            quote! {
                self.write(f)
            }
        };
        if needs_parentheses_before.is_empty() && trailing_comments.is_empty() {
            quote! {
                #write_implementation
            }
        } else {
            quote! {
                #leading_comments
                #needs_parentheses_before
                let result = #write_implementation;
                #needs_parentheses_after
                #trailing_comments
                result
            }
        }
    };

    let fmt_implementation = generate_fmt_implementation(false);
    let fmt_options =
        NEEDS_IMPLEMENTING_FMT_WITH_OPTIONS.get(type_def_name).map(|str| format_ident!("{}", str));
    let fmt_with_options_implementation = if let Some(ref fmt_options) = fmt_options {
        let implementation = generate_fmt_implementation(true);
        quote! {
            ///@@line_break
            fn fmt_with_options(&self, options: #fmt_options, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
                #implementation
            }
        }
    } else {
        quote! {}
    };

    let option_type = fmt_options.map_or_else(|| quote! {}, |ident| quote! {, #ident});

    quote! {
        ///@@line_break
        impl<'a> Format<'a #option_type> for #type_ty {
            fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
                #fmt_implementation
            }

            #fmt_with_options_implementation
        }
    }
}
