//! Generator for `oxc_formatter`.
//!
//! Boundary with the handwritten side:
//!
//! - The node lists here decide which fragments of the `fmt` skeleton are EMITTED (comment-printing ownership, parentheses frames)
//!   - they change the shape of the generated code, nothing else
//! - Behavior the skeleton queries per node lives on `FormatWrite` (`write`, `suppressed_span`, `write_suppressed`)
//!   - defaults cover the common case, overrides sit next to the node's `write` and need no regeneration
//!   - EXCEPT: expression-shaped nodes bypass `write_suppressed` entirely
//!     (their suppressed path goes to `write_suppressed_expression`, see below),
//!     so an override on such a node would silently never be called
//!
//! Add a new list only for a variation the emitted shape must express;
//! for anything a node merely answers, add a trait method with a total default.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    Codegen, Generator,
    generators::{define_generator, formatter::ast_nodes::get_node_type},
    output::Output,
    schema::{Def, EnumDef, Schema, StructDef, StructOrEnum, TypeDef, TypeId},
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
    // Manually prints it because class's decorators can be appears before `export class Cls {}`.
    "ExportDeclaration",
    "ExportNamedDeclaration",
    "ExportFromDeclaration",
    "ExportDefaultDeclaration",
    //
    "JSXElement",
    "JSXFragment",
    //
    "TemplateElement",
];

// `ExpressionStatement` prints leading comments in its `write` implementation,
// so the ASI-guard semicolon can be printed before a leading type cast comment.
const AST_NODE_WITHOUT_PRINTING_LEADING_COMMENTS_LIST: &[&str] =
    &["TSUnionType", "ExpressionStatement"];

const AST_NODE_NEEDS_PARENTHESES: &[&str] = &[
    "TSTypeAssertion",
    "TSInferType",
    "TSConditionalType",
    "TSUnionType",
    "TSIntersectionType",
    "TSConstructorType",
    "TSTypeQuery",
    "TSFunctionType",
    "TSTypeOperator",
];

pub struct FormatterFormatGenerator;

define_generator!(FormatterFormatGenerator);

impl Generator for FormatterFormatGenerator {
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let parenthesis_type_ids = get_needs_parentheses_type_ids(schema);

        let impls = schema
            .structs_and_enums()
            .filter_map(|type_def| match type_def {
                StructOrEnum::Struct(struct_def)
                    if struct_def.visit.has_visitor() && !struct_def.builder.skip =>
                {
                    Some(generate_struct_implementation(struct_def, &parenthesis_type_ids, schema))
                }
                StructOrEnum::Enum(enum_def) if enum_def.visit.has_visitor() => {
                    Some(generate_enum_implementation(enum_def, schema))
                }
                _ => None,
            })
            .collect::<TokenStream>();

        let output = quote! {
            #![expect(clippy::match_same_arms)]
            use oxc_ast::ast::*;
            use oxc_formatter_core::Format;
            use oxc_span::GetSpan;

            ///@@line_break
            use crate::{
                formatter::{JsFormatContext, JsFormatter, JsFormatterExt as _, trivia::{format_leading_comments, format_trailing_comments}},
                parentheses::NeedsParentheses,
                ast_nodes::AstNode,
                utils::{suppressed::{FormatSuppressedNode, write_suppressed_expression}, typecast::{format_type_cast_comment_node, format_leading_comments_and_open_paren, format_outer_leading_comments_and_open_paren}},
                print::FormatWrite,
            };

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
    let do_not_print_leading_comment = do_not_print_comment
        || AST_NODE_WITHOUT_PRINTING_LEADING_COMMENTS_LIST.contains(&struct_name);

    let needs_parentheses = parenthesis_type_ids.contains(&struct_def.id);

    // For nodes that may get formatter-added parentheses,
    // leading comments are printed by the parentheses block below (ordering depends on the comments),
    // not as a standalone step.
    let leading_comments = (!do_not_print_leading_comment && !needs_parentheses).then(|| {
        quote! {
            self.format_leading_comments(f);
        }
    });
    let trailing_comments = (!do_not_print_comment).then(|| {
        quote! {
            self.format_trailing_comments(f);
        }
    });

    let needs_parentheses_before = if needs_parentheses {
        if do_not_print_comment {
            // The node owns ALL its comment printing (leading and trailing) in `write`;
            // keep the added paren bare and leave every comment to it.
            quote! {
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
            }
        } else if do_not_print_leading_comment {
            // The node prints its own leading comments in `write`,
            // but the ones that belong outside the formatter-added paren (source side / own-line) must
            // print first (`X & /* c */ (A | B)` keeps `c` outside).
            quote! {
                let needs_parentheses = self.needs_parentheses(f);
                format_outer_leading_comments_and_open_paren(self.span(), needs_parentheses, f);
            }
        } else {
            // A leading type cast comment must stay adjacent to the `(` of its cast target inside this node;
            // the helper prints the comments inside the added parentheses in that case,
            // or the cast would rebind to them.
            quote! {
                let needs_parentheses = self.needs_parentheses(f);
                format_leading_comments_and_open_paren(self.span(), self.leading_comments_start(), needs_parentheses, f);
            }
        }
    } else {
        quote! {}
    };

    let needs_parentheses_after = if needs_parentheses {
        quote! {
            if needs_parentheses {
                ")".fmt(f);
            }
        }
    } else {
        quote! {}
    };

    let fmt_implementation = {
        let write_call = quote! {
            self.write(f);
        };

        // `Program` can't be suppressed.
        // `JSXElement` and `JSXFragment` implement suppression formatting in their formatting logic.
        //
        // The check, the suppressed leading comments, and the printed range are all bounded by
        // `FormatWrite::suppressed_span` (default: the node's span),
        // which nodes override when the ignored range starts before their span (class decorators before `export`).
        // `FormatWrite::write_suppressed` (default: print `suppressed_span` verbatim) is overridden by
        // statements whose ignored range excludes the trailing semicolon.
        let suppressed_check = (!matches!(struct_name, "Program" | "JSXElement" | "JSXFragment"))
            .then(|| {
                quote! {
                    let is_suppressed = f.comments().is_suppressed(self.suppressed_span().start);
                }
            });

        // Expression-shaped nodes (formatter parens + own comment printing) hand the whole
        // suppressed sequence to one owner, so the cast-target decision is made once
        // while every comment is still unprinted (see `write_suppressed_expression`).
        // These nodes have no `suppressed_span`/`write_suppressed` overrides (those are statements).
        let suppressed_expression_return =
            (suppressed_check.is_some() && needs_parentheses && !do_not_print_leading_comment)
                .then(|| {
                    quote! {
                        if is_suppressed {
                            write_suppressed_expression(
                                self.span(),
                                self.leading_comments_start(),
                                self.needs_parentheses(f),
                                f,
                            );
                            self.format_trailing_comments(f);
                            return;
                        }
                    }
                });

        let write_implementation =
            if suppressed_check.is_none() || suppressed_expression_return.is_some() {
                write_call
            } else {
                // When `fmt` doesn't print leading/trailing comments itself,
                // the suppressed path still has to print them, or the suppression comment would be lost.
                let suppressed_leading_comments = do_not_print_leading_comment.then(|| {
                    quote! {
                        format_leading_comments(self.suppressed_span()).fmt(f);
                    }
                });
                let suppressed_trailing_comments = do_not_print_comment.then(|| {
                    quote! {
                        self.format_trailing_comments(f);
                    }
                });
                quote! {
                    if is_suppressed {
                        #suppressed_leading_comments
                        self.write_suppressed(f);
                        #suppressed_trailing_comments
                    } else {
                        #write_call
                    }
                }
            };

        let type_cast_comment_formatting = needs_parentheses.then(|| {
            let is_object_or_array_argument =
                if matches!(struct_def.name.as_str(), "ObjectExpression" | "ArrayExpression") {
                    quote! {
                        true
                    }
                } else {
                    quote! { false }
                };

            // With the suppressed early return above, the flag is trivially false here
            let suppressed_check_for_typecast = (suppressed_check.is_some()
                && suppressed_expression_return.is_none())
            .then(|| {
                quote! {
                    !is_suppressed &&
                }
            });

            quote! {
                if #suppressed_check_for_typecast format_type_cast_comment_node(self, #is_object_or_array_argument, f) {
                    return;
                }
            }
        });

        if !needs_parentheses && trailing_comments.is_none() {
            quote! {
                #suppressed_check
                #type_cast_comment_formatting
                #write_implementation
            }
        } else {
            quote! {
                #suppressed_check
                #suppressed_expression_return
                #type_cast_comment_formatting
                #leading_comments
                #needs_parentheses_before
                #write_implementation
                #needs_parentheses_after
                #trailing_comments
            }
        }
    };

    quote! {
        ///@@line_break
        impl<'a> Format<'a, JsFormatContext<'a>> for #type_ty {
            fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
                #fmt_implementation
            }
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
                    following_span_start: self.following_span_start,
                }).fmt(f);
            },
        })
    });

    let inherits_match_arms = enum_def.inherits_enums(schema).map(|inherits_type| {
        let inherits_ident = inherits_type.ident();
        let inherits_snake_name = inherits_type.snake_name();
        let match_ident = format_ident!("match_{inherits_snake_name}");

        let to_fn_ident = format_ident!("to_{inherits_snake_name}");
        let match_arm = quote! {
            it @ #match_ident!(#enum_ident) => {
                let inner = it.#to_fn_ident();
                allocator.alloc(AstNode::<'a, #inherits_ident> {
                    inner,
                    parent,
                    allocator,
                    following_span_start: self.following_span_start,
                }).fmt(f);
            },
        };

        match_arm
    });

    let node_type = get_node_type(&enum_ty);

    let inline_trailing_suppression = match enum_def.name() {
        "Statement" => {
            // Expression statements need specialized ASI-safe suppression handling in
            // `AstNode<ExpressionStatement>::write`.
            quote! {
                if !matches!(self.inner, Statement::ExpressionStatement(_))
                    && f.comments().has_trailing_suppression_comment(self.span().end)
                {
                    format_leading_comments(self.span()).fmt(f);
                    FormatSuppressedNode(self.span()).fmt(f);
                    format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
                        .fmt(f);
                    return;
                }
            }
        }
        "Expression" => {
            quote! {
                if f.comments().has_trailing_suppression_comment(self.span().end) {
                    format_leading_comments(self.span()).fmt(f);
                    FormatSuppressedNode(self.span()).fmt(f);
                    format_trailing_comments(self.parent.span(), self.inner.span(), self.following_span_start)
                        .fmt(f);
                    return;
                }
            }
        }
        _ => quote! {},
    };

    quote! {
        ///@@line_break
        impl<'a> Format<'a, JsFormatContext<'a>> for #node_type {
            #[inline]
            fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
                #inline_trailing_suppression
                let allocator = self.allocator;
                let parent = self.parent;
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
