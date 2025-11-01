//! Generator for `oxc_formatter`.
//!

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    Codegen, Generator,
    generators::{define_generator, formatter::ast_nodes::get_node_type},
    output::Output,
    schema::{Def, EnumDef, Schema, StructDef, TypeDef, TypeId},
};

use super::ast_nodes::formatter_output_path;

/// Based on the prettier printing comments algorithm, these nodes don't need to print comments.
const AST_NODE_WITHOUT_PRINTING_COMMENTS_LIST: &[&str] = &[
    "Program",
    "FormalParameters",
    "FunctionBody",
    "ClassBody",
    "CatchParameter",
    "CatchClause",
    "Decorator",
    // Manually prints it because class's decorators can be appears before `export class Cls {}`.
    "ExportNamedDeclaration",
    "ExportDefaultDeclaration",
    "TSClassImplements",
    //
    "JSXElement",
    "JSXFragment",
    //
    "TemplateElement",
];

const AST_NODE_NEEDS_PARENTHESES: &[&str] = &[
    "TSTypeAssertion",
    "TSInferType",
    "TSConditionalType",
    "TSUnionType",
    "TSIntersectionType",
    "TSConstructorType",
    "TSTypeQuery",
    "TSFunctionType",
];

const NEEDS_IMPLEMENTING_FMT_WITH_OPTIONS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "ArrowFunctionExpression" => "FormatJsArrowFunctionExpressionOptions",
    "Function" => "FormatFunctionOptions",
};

pub struct FormatterFormatGenerator;

define_generator!(FormatterFormatGenerator);

impl Generator for FormatterFormatGenerator {
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let parenthesis_type_ids = get_needs_parentheses_type_ids(schema);

        let impls = schema
            .types
            .iter()
            .filter_map(|type_def| match type_def {
                TypeDef::Struct(struct_def)
                    if struct_def.visit.has_visitor() && !struct_def.builder.skip =>
                {
                    Some(generate_struct_implementation(struct_def, &parenthesis_type_ids, schema))
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
            use oxc_span::GetSpan;

            ///@@line_break
            use crate::{
                formatter::{
                    Buffer, Format, FormatResult, Formatter,
                    trivia::FormatTrailingComments,
                },
                parentheses::NeedsParentheses,
                ast_nodes::{AstNode, AstNodes},
                utils::{suppressed::FormatSuppressedNode, typecast::format_type_cast_comment_node},
                write::{FormatWrite #(#options)*},
            };

            use super::ast_nodes::transmute_self;

            #impls
        };

        Output::Rust { path: formatter_output_path("format"), tokens: output }
    }
}

fn generate_struct_implementation(
    struct_def: &StructDef,
    parenthesis_type_ids: &[TypeId],
    schema: &Schema,
) -> TokenStream {
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

    let needs_parentheses = parenthesis_type_ids.contains(&struct_def.id);

    let needs_parentheses_before = if needs_parentheses {
        if matches!(struct_name, "JSXElement" | "JSXFragment") {
            quote! {
                let needs_parentheses = !is_suppressed && self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f)?;
                }
            }
        } else {
            quote! {
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f)?;
                }
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
        let write_call = if has_options {
            quote! {
                self.write_with_options(options, f)
            }
        } else {
            quote! {
                self.write(f)
            }
        };

        // `Program` can't be suppressed.
        // `JSXElement` and `JSXFragment` need special suppression handling before parentheses
        let suppressed_check =
            (!matches!(struct_name, "Program")).then(|| {
                quote! {
                    let is_suppressed = f.comments().is_suppressed(self.span().start);
                }
            });

        let write_implementation = if suppressed_check.is_none() {
            write_call
        } else if trailing_comments.is_empty() {
            quote! {
                if is_suppressed {
                     self.format_leading_comments(f)?;
                    FormatSuppressedNode(self.span()).fmt(f)?;
                     self.format_trailing_comments(f)
                } else {
                    #write_call
                }
            }
        } else {
            quote! {
                if is_suppressed {
                    FormatSuppressedNode(self.span()).fmt(f)
                } else {
                    #write_call
                }
            }
        };

        let type_cast_comment_formatting = parenthesis_type_ids.contains(&struct_def.id).then(|| {
            let is_object_or_array_argument =
                if matches!(struct_def.name.as_str(), "ObjectExpression" | "ArrayExpression") {
                    quote! {
                        true
                    }
                } else {
                    quote! { false }
                };

            let suppressed_check_for_typecast = suppressed_check.is_some().then(|| {
                quote! {
                    !is_suppressed &&
                }
            });

            quote! {
                if #suppressed_check_for_typecast format_type_cast_comment_node(self, #is_object_or_array_argument, f)? {
                    return Ok(());
                }
            }
        });

        if needs_parentheses_before.is_empty() && trailing_comments.is_empty() {
            quote! {
                #suppressed_check
                #type_cast_comment_formatting
                #write_implementation
            }
        } else {
            quote! {
                #suppressed_check
                #type_cast_comment_formatting
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
                    following_span: self.following_span,
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
                    following_span: self.following_span,
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

/// Get [`TypeId`]s of types which do not have a following node.
fn get_needs_parentheses_type_ids(schema: &Schema) -> Vec<TypeId> {
    let mut type_ids =
        AST_NODE_NEEDS_PARENTHESES.iter().map(|&name| schema.type_names[name]).collect::<Vec<_>>();

    let expression_enum = schema.type_by_name("Expression").as_enum().unwrap();
    type_ids.extend(
        expression_enum
            .all_variants(schema)
            .filter_map(|variant| variant.field_type(schema))
            .map(|variant_type| variant_type.innermost_type(schema).id()),
    );

    type_ids
}
