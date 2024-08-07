use itertools::Itertools;
use quote::quote;
use syn::{parse_quote, Arm, Ident, Type, Variant};

use crate::{
    output,
    schema::{GetIdent, ToType, TypeDef},
    util::ToIdent,
    Generator, GeneratorOutput, LateCtx,
};

use super::{define_generator, generated_header};

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

pub fn blacklist((ident, _): &(Ident, Type)) -> bool {
    !BLACK_LIST.contains(&ident.to_string().as_str())
}

pub fn aliased_nodes() -> [(Ident, Type); 1] {
    use syn::parse_quote as pq;
    [(pq!(ExpressionArrayElement), pq!(Expression<'a>))]
}

pub fn process_types(def: &TypeDef, _: &LateCtx) -> Vec<(Ident, Type)> {
    let aliases = match def {
        TypeDef::Enum(enum_) => enum_
            .variants
            .iter()
            // .map(|it| (it, get_visit_markers(&it.attrs).transpose().unwrap()))
            .filter(|it| it.markers.visit.as_ref().is_some_and(|mk| mk.visit_as.is_some()))
            .filter_map(|var| {
                var.markers.visit.as_ref().map(|markers| {
                    let field = var.fields.first().unwrap();
                    let type_name = field.typ.name().inner_name();
                    (
                        markers.visit_as.clone().expect("Already checked"),
                        parse_quote!(#type_name<'a>),
                    )
                })
            })
            .collect_vec(),
        TypeDef::Struct(struct_) => struct_
            .fields
            .iter()
            // .map(|it| (it, get_visit_markers(&it.attrs).transpose().unwrap()))
            .filter(|it| it.markers.visit.as_ref().is_some_and(|mk| mk.visit_as.is_some()))
            .filter_map(|field| {
                field.markers.visit.as_ref().map(|markers| {
                    let type_name = field.typ.name().inner_name().to_ident();
                    (
                        markers.visit_as.clone().expect("Already checked"),
                        parse_quote!(#type_name<'a>),
                    )
                })
            })
            .collect_vec(),
    };

    Some(def)
        .into_iter()
        .map(|def| {
            let ident = def.ident();
            let typ = def.to_type();
            (ident, typ)
        })
        .chain(aliases)
        .collect()
}

impl Generator for AstKindGenerator {
    fn name(&self) -> &'static str {
        stringify!(AstKindGenerator)
    }

    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let have_kinds: Vec<(Ident, Type)> = ctx
            .schema.definitions
            .iter()
            .filter(|it| it.visitable())
            .filter(
                |maybe_kind| matches!(maybe_kind, kind @ (TypeDef::Enum(_) | TypeDef::Struct(_)) if kind.visitable())
            )
            .flat_map(|it| process_types(it, ctx))
            .filter(blacklist)
            .chain(aliased_nodes())
            .collect();

        let types: Vec<Variant> =
            have_kinds.iter().map(|(ident, _)| parse_quote!(#ident)).collect_vec();

        let kinds: Vec<Variant> =
            have_kinds.iter().map(|(ident, typ)| parse_quote!(#ident(&'a #typ))).collect_vec();

        let span_matches: Vec<Arm> = have_kinds
            .iter()
            .map(|(ident, _)| parse_quote!(Self :: #ident(it) => it.span()))
            .collect_vec();

        let header = generated_header!();

        GeneratorOutput::Stream((
            output(crate::AST_CRATE, "ast_kind.rs"),
            quote! {
                #header

                use crate::ast::*;
                use oxc_span::{GetSpan, Span};

                endl!();

                #[derive(Debug, Clone, Copy)]
                pub enum AstType {
                    #(#types),*,
                }

                endl!();

                /// Untyped AST Node Kind
                #[derive(Debug, Clone, Copy)]
                pub enum AstKind<'a> {
                    #(#kinds),*,
                }

                endl!();

                impl<'a> GetSpan for AstKind<'a> {
                    #[allow(clippy::match_same_arms)]
                    fn span(&self) -> Span {
                        match self {
                            #(#span_matches),*,
                        }
                    }
                }
            },
        ))
    }
}
