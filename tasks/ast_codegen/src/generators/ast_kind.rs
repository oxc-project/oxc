use itertools::Itertools;
use quote::quote;
use syn::{parse_quote, Arm, Ident, Type, Variant};

use crate::{schema::RType, CodegenCtx, Generator, GeneratorOutput};

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

pub fn aliased_nodes() -> [(Ident, Type); 3] {
    use syn::parse_quote as pq;
    [
        (pq!(FinallyClause), pq!(BlockStatement<'a>)),
        (pq!(ClassHeritage), pq!(Expression<'a>)),
        (pq!(ExpressionArrayElement), pq!(Expression<'a>)),
    ]
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
            .filter_map(|maybe_kind| match &*maybe_kind.borrow() {
                kind @ (RType::Enum(_) | RType::Struct(_)) if kind.visitable() => {
                    let ident = kind.ident().unwrap().clone();
                    let typ = kind.as_type().unwrap();
                    Some((ident, typ))
                }
                _ => None,
            })
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
