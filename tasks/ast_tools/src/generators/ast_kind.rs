use itertools::Itertools;
use quote::quote;
use syn::{parse_quote, Arm, Ident, Type, Variant};

use crate::{
    codegen::LateCtx,
    output,
    schema::{GetIdent, ToType, TypeDef},
    util::ToIdent,
    Generator, GeneratorOutput,
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
            .filter(|it| it.markers.visit.visit_as.is_some())
            .map(|var| {
                let field = var.fields.first().unwrap();
                let type_name = field.typ.name().inner_name();
                (
                    var.markers.visit.visit_as.clone().expect("Already checked"),
                    parse_quote!(#type_name<'a>),
                )
            })
            .collect_vec(),
        TypeDef::Struct(struct_) => struct_
            .fields
            .iter()
            .filter(|it| it.markers.visit.visit_as.is_some())
            .map(|field| {
                let type_name = field.typ.name().inner_name().to_ident();
                (
                    field.markers.visit.visit_as.clone().expect("Already checked"),
                    parse_quote!(#type_name<'a>),
                )
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
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let have_kinds: Vec<(Ident, Type)> = ctx
            .schema()
            .into_iter()
            .filter(|it| it.visitable())
            .filter(
                |maybe_kind| matches!(maybe_kind, kind @ (TypeDef::Enum(_) | TypeDef::Struct(_)) if kind.visitable())
            )
            .flat_map(|it| process_types(it, ctx))
            .filter(blacklist)
            .chain(aliased_nodes())
            .collect();

        let (types, kinds): (Vec<Variant>, Vec<Variant>) = have_kinds
            .iter()
            .enumerate()
            .map(|(index, (ident, typ))| {
                let id = u8::try_from(index).unwrap();
                let type_variant = parse_quote!(#ident = #id);
                let kind_variant = parse_quote!(#ident(&'a #typ) = AstType::#ident as u8);
                (type_variant, kind_variant)
            })
            .unzip();

        let span_matches: Vec<Arm> = have_kinds
            .iter()
            .map(|(ident, _)| parse_quote!(Self :: #ident(it) => it.span()))
            .collect_vec();

        let header = generated_header!();

        GeneratorOutput::Stream((
            output(crate::AST_CRATE, "ast_kind.rs"),
            quote! {
                #header

                use oxc_span::{GetSpan, Span};

                ///@@line_break
                #[allow(clippy::wildcard_imports)]
                use crate::ast::*;

                ///@@line_break
                /// AST node type
                ///@ SAFETY: Soundness of [`AstKind::ast_type`], [`AstKind::payload`],
                ///@ and [`AstKind::from_type_and_payload`] methods rely on this type being `#[repr(u8)]`
                ///@ and having same discriminants as `AstKind`.
                #[derive(Debug, Clone, Copy)]
                #[repr(u8)]
                pub enum AstType {
                    #(#types),*,
                }

                ///@@line_break
                /// Untyped AST node kind
                ///@ SAFETY: Soundness of [`AstKind::ast_type`], [`AstKind::payload`],
                ///@ and [`AstKind::from_type_and_payload`] methods rely on this type being `#[repr(C, u8)]`
                ///@ and having same discriminants as `AstType`.
                #[derive(Debug, Clone, Copy)]
                #[repr(C, u8)]
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
            },
        ))
    }
}
