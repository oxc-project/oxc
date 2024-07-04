use itertools::Itertools;
use quote::quote;
use syn::{parse_quote, Arm, Ident, Type, Variant};

use crate::{schema::RType, util::TypeExt, CodegenCtx, Generator, GeneratorOutput, TypeRef};

use super::generated_header;

pub struct AstKindGenerator;

pub const BLACK_LIST: [&str; 69] = [
    "Expression",
    "ObjectPropertyKind",
    "TemplateElement",
    "ComputedMemberExpression",
    "StaticMemberExpression",
    "PrivateFieldExpression",
    "AssignmentTargetPattern",
    "ArrayAssignmentTarget",
    "ObjectAssignmentTarget",
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
    "TSConditionalType",
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
    "TSConstructSignatureDeclaration",
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
    "TSMappedType",
    "TSModuleReference",
    "TSExportAssignment",
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

pub fn process_types(ty: &TypeRef) -> Vec<(Ident, Type)> {
    let aliases = match &*ty.borrow() {
        RType::Enum(enum_) => enum_
            .item
            .variants
            .iter()
            .filter_map(|it| {
                it.attrs
                    .iter()
                    .find(|it| it.path().is_ident("visit_as"))
                    .map(|attr| (it, attr))
                    .map(|(it, attr)| {
                        assert!(
                            it.fields.len() == 1,
                            "visit_as only supports single argument fields."
                        );
                        let field = it.fields.iter().next().unwrap();
                        let type_name = field.ty.get_ident().inner_ident();
                        (attr.parse_args().unwrap(), parse_quote!(#type_name<'a>))
                    })
            })
            .collect_vec(),
        RType::Struct(struct_) => struct_
            .item
            .fields
            .iter()
            .filter_map(|it| {
                it.attrs
                    .iter()
                    .find(|it| it.path().is_ident("visit_as"))
                    .map(|attr| (it, attr))
                    .map(|(field, attr)| {
                        let type_name = field.ty.get_ident().inner_ident();
                        (attr.parse_args().unwrap(), parse_quote!(#type_name<'a>))
                    })
            })
            .collect_vec(),
        _ => panic!(),
    };

    Some(ty)
        .into_iter()
        .map(|kind| {
            if let kind @ (RType::Enum(_) | RType::Struct(_)) = &*kind.borrow() {
                let ident = kind.ident().unwrap().clone();
                let typ = kind.as_type().unwrap();
                (ident, typ)
            } else {
                panic!()
            }
        })
        .chain(aliases)
        .collect()
}

impl Generator for AstKindGenerator {
    fn name(&self) -> &'static str {
        "AstKindGenerator"
    }

    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput {
        let have_kinds: Vec<(Ident, Type)> = ctx
            .ty_table
            .iter()
            .filter(|it| it.borrow().visitable())
            .filter(
                |maybe_kind| matches!(&*maybe_kind.borrow(), kind @ (RType::Enum(_) | RType::Struct(_)) if kind.visitable())
            )
            .flat_map(process_types)
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

        GeneratorOutput::One(quote! {
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
        })
    }
}
