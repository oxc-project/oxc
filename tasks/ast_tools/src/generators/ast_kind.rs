use convert_case::{Case, Casing};
use itertools::Itertools;
use quote::{format_ident, quote};
use syn::{parse_quote, Arm, ImplItemFn, Variant};

use super::define_generator;
use crate::{
    codegen::{generated_header, LateCtx},
    output,
    schema::{GetIdent, ToType},
    Generator, GeneratorOutput,
};

define_generator! {
    pub struct AstKindGenerator;
}

pub const BLACK_LIST: [&str; 61] = [
    "Expression",
    "ObjectPropertyKind",
    "TemplateElement",
    "ComputedMemberExpression",
    "StaticMemberExpression",
    "PrivateFieldExpression",
    "AssignmentTargetRest",
    "AssignmentTargetMaybeDefault",
    "AssignmentTargetProperty",
    "AssignmentTargetPropertyIdentifier",
    "AssignmentTargetPropertyProperty",
    "ChainElement",
    "Statement",
    "Declaration",
    "ForStatementLeft",
    "BindingPattern",
    "BindingPatternKind",
    "BindingProperty",
    "ClassElement",
    "AccessorProperty",
    "ImportDeclarationSpecifier",
    "WithClause",
    "ImportAttribute",
    "ImportAttributeKey",
    "ExportDefaultDeclarationKind",
    "ModuleExportName",
    "TSEnumMemberName",
    "TSLiteral",
    "TSType",
    "TSTypeOperator",
    "TSArrayType",
    "TSTupleType",
    "TSOptionalType",
    "TSRestType",
    "TSTupleElement",
    "TSInterfaceBody",
    "TSSignature",
    "TSIndexSignature",
    "TSCallSignatureDeclaration",
    "TSIndexSignatureName",
    "TSTypePredicate",
    "TSTypePredicateName",
    "TSModuleDeclarationName",
    "TSModuleDeclarationBody",
    "TSTypeQueryExprName",
    "TSImportAttribute",
    "TSImportAttributes",
    "TSImportAttributeName",
    "TSFunctionType",
    "TSConstructorType",
    "TSNamespaceExportDeclaration",
    "JSDocNullableType",
    "JSDocNonNullableType",
    "JSDocUnknownType",
    "JSXExpression",
    "JSXEmptyExpression",
    "JSXAttribute",
    "JSXAttributeName",
    "JSXAttributeValue",
    "JSXChild",
    "JSXSpreadChild",
];

impl Generator for AstKindGenerator {
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let have_kinds = ctx
            .schema()
            .into_iter()
            .filter(|def| {
                let is_visitable = def.visitable();
                let is_blacklisted = BLACK_LIST.contains(&def.name());
                is_visitable && !is_blacklisted
            })
            .map(|def| {
                let ident = def.ident();
                let typ = def.to_type();
                (ident, typ)
            })
            .collect_vec();

        let types = have_kinds.iter().map(|(ident, _)| ident).collect_vec();

        let kinds: Vec<Variant> =
            have_kinds.iter().map(|(ident, typ)| parse_quote!(#ident(&'a #typ))).collect_vec();

        let span_matches: Vec<Arm> = have_kinds
            .iter()
            .map(|(ident, _)| parse_quote!(Self :: #ident(it) => it.span()))
            .collect_vec();

        let as_ast_kind_impls: Vec<ImplItemFn> = have_kinds
            .iter()
            .map(|(ident, typ)| {
                let snake_case_name =
                    format_ident!("as_{}", ident.to_string().to_case(Case::Snake));
                parse_quote!(
                    ///@@line_break
                    #[inline]
                    pub fn #snake_case_name(&self) -> Option<&'a #typ> {
                        if let Self::#ident(v) = self {
                            Some(*v)
                        } else {
                            None
                        }
                    }
                )
            })
            .collect_vec();

        let header = generated_header!();

        GeneratorOutput::Rust {
            path: output(crate::AST_CRATE, "ast_kind.rs"),
            tokens: quote! {
                #header

                use oxc_span::{GetSpan, Span};

                ///@@line_break
                #[allow(clippy::wildcard_imports)]
                use crate::ast::*;

                ///@@line_break
                #[derive(Debug, Clone, Copy)]
                pub enum AstType {
                    #(#types),*,
                }

                ///@@line_break
                /// Untyped AST Node Kind
                #[derive(Debug, Clone, Copy)]
                pub enum AstKind<'a> {
                    #(#kinds),*,
                }

                ///@@line_break
                impl<'a> GetSpan for AstKind<'a> {
                    #[allow(clippy::match_same_arms)]
                    fn span(&self) -> Span {
                        match self {
                            #(#span_matches),*,
                        }
                    }
                }

                ///@@line_break
                impl<'a> AstKind<'a> {
                    #(#as_ast_kind_impls)*
                }
            },
        }
    }
}
