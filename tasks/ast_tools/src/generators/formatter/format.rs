//! Generator for `oxc_formatter`.
//!

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    Codegen, Generator,
    generators::{define_generator, formatter::ast_nodes::get_node_type},
    output::{Output, output_path},
    schema::{Def, EnumDef, Schema, StructDef, TypeDef},
};

const FORMATTER_CRATE_PATH: &str = "crates/oxc_formatter";

/// Based on the prettier printing comments algorithm, these nodes don't need to print comments.
const AST_NODE_WITHOUT_PRINTING_COMMENTS_LIST: &[&str] = &[
    "Program",
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
    //
    "JSXElement",
    "JSXFragment",
    //
    "TemplateElement",
];

const NEEDS_PARENTHESES: &[&str] = &[
    "Class",
    "Function",
    "NumericLiteral",
    "SimpleAssignmentTarget",
    "StringLiteral",
    "TSTypeAssertion",
    "IdentifierReference",
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
            .filter_map(|type_def| match type_def {
                TypeDef::Struct(struct_def)
                    if struct_def.visit.has_visitor() && !struct_def.builder.skip =>
                {
                    Some(generate_struct_implementation(struct_def, schema))
                }
                TypeDef::Enum(enum_def) if enum_def.visit.has_visitor() => {
                    Some(generate_enum_implementation(enum_def, schema))
                }
                _ => None,
            })
            .collect::<TokenStream>();

        let options = NEEDS_IMPLEMENTING_FMT_WITH_OPTIONS.values().map(|o| {
            let ident = format_ident!("{}", o);
            quote! { , #ident }
        });

        let output = quote! {
            #![expect(clippy::match_same_arms)]
            use oxc_ast::ast::*;

            ///@@line_break
            use crate::{
                formatter::{
                    Buffer, Format, FormatResult, Formatter,
                    trivia::FormatTrailingComments,
                },
                parentheses::NeedsParentheses,
                generated::ast_nodes::{AstNode, AstNodes, transmute_self},
                write::{FormatWrite #(#options)*},
            };

            #impls
        };

        Output::Rust { path: output_path(FORMATTER_CRATE_PATH, "format.rs"), tokens: output }
    }
}

fn generate_struct_implementation(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    let type_ty = struct_def.ty(schema);
    let type_ty = quote! {
        AstNode::<'a, #type_ty>
    };

    let struct_name = struct_def.name();
    let do_not_print_comment = AST_NODE_WITHOUT_PRINTING_COMMENTS_LIST.contains(&struct_name);

    let leading_comments = if do_not_print_comment {
        quote! {}
    } else {
        quote! {
            self.format_leading_comments(f)?;
        }
    };

    let trailing_comments = if do_not_print_comment {
        quote! {}
    } else {
        quote! {
            self.format_trailing_comments(f)?;
        }
    };

    let needs_parentheses =
        struct_name.ends_with("Expression") || NEEDS_PARENTHESES.contains(&struct_name);
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
        NEEDS_IMPLEMENTING_FMT_WITH_OPTIONS.get(struct_name).map(|str| format_ident!("{}", str));
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

fn generate_enum_implementation(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
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

    quote! {
        ///@@line_break
        impl<'a> Format<'a> for #node_type {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
                let allocator = self.allocator;
                #parent;
                match self.inner {
                    #(#variant_match_arms)*
                    #(#inherits_match_arms)*
                }
            }
        }
    }
}
