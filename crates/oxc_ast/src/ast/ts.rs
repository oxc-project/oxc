//! TypeScript Definitions
//!
//! [AST Spec](https://github.com/typescript-eslint/typescript-eslint/tree/main/packages/ast-spec)
//! [Archived TypeScript spec](https://github.com/microsoft/TypeScript/blob/3c99d50da5a579d9fa92d02664b1b66d4ff55944/doc/spec-ARCHIVED.md)

use oxc_allocator::{Box, Vec};
use oxc_span::{Atom, GetSpan, Span};
#[cfg(feature = "serde")]
use serde::Serialize;

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSThisParameter<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub this: IdentifierName<'a>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

/// Enum Declaration
///
/// `const_opt` enum `BindingIdentifier` { `EnumBody_opt` }
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSEnumDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier<'a>,
    pub members: Vec<'a, TSEnumMember<'a>>,
    /// Valid Modifiers: `const`, `export`, `declare`
    pub modifiers: Modifiers<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSEnumMember<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: TSEnumMemberName<'a>,
    pub initializer: Option<Expression<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSEnumMemberName<'a> {
    Identifier(IdentifierName<'a>),
    StringLiteral(StringLiteral<'a>),
    // Invalid Grammar `enum E { [computed] }`
    ComputedPropertyName(Expression<'a>),
    // Invalid Grammar `enum E { 1 }`
    NumericLiteral(NumericLiteral<'a>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTypeAnnotation<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSLiteralType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub literal: TSLiteral<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSLiteral<'a> {
    BooleanLiteral(Box<'a, BooleanLiteral>),
    NullLiteral(Box<'a, NullLiteral>),
    NumericLiteral(Box<'a, NumericLiteral<'a>>),
    BigintLiteral(Box<'a, BigintLiteral<'a>>),
    RegExpLiteral(Box<'a, RegExpLiteral<'a>>),
    StringLiteral(Box<'a, StringLiteral<'a>>),
    TemplateLiteral(Box<'a, TemplateLiteral<'a>>),
    UnaryExpression(Box<'a, UnaryExpression<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSType<'a> {
    // Keyword
    TSAnyKeyword(Box<'a, TSAnyKeyword>),
    TSBigIntKeyword(Box<'a, TSBigIntKeyword>),
    TSBooleanKeyword(Box<'a, TSBooleanKeyword>),
    TSNeverKeyword(Box<'a, TSNeverKeyword>),
    TSNullKeyword(Box<'a, TSNullKeyword>),
    TSNumberKeyword(Box<'a, TSNumberKeyword>),
    TSObjectKeyword(Box<'a, TSObjectKeyword>),
    TSStringKeyword(Box<'a, TSStringKeyword>),
    TSSymbolKeyword(Box<'a, TSSymbolKeyword>),
    TSThisType(Box<'a, TSThisType>),
    TSUndefinedKeyword(Box<'a, TSUndefinedKeyword>),
    TSUnknownKeyword(Box<'a, TSUnknownKeyword>),
    TSVoidKeyword(Box<'a, TSVoidKeyword>),
    // Compound
    TSArrayType(Box<'a, TSArrayType<'a>>),
    TSConditionalType(Box<'a, TSConditionalType<'a>>),
    TSConstructorType(Box<'a, TSConstructorType<'a>>),
    TSFunctionType(Box<'a, TSFunctionType<'a>>),
    TSImportType(Box<'a, TSImportType<'a>>),
    TSIndexedAccessType(Box<'a, TSIndexedAccessType<'a>>),
    TSInferType(Box<'a, TSInferType<'a>>),
    TSIntersectionType(Box<'a, TSIntersectionType<'a>>),
    TSLiteralType(Box<'a, TSLiteralType<'a>>),
    TSMappedType(Box<'a, TSMappedType<'a>>),
    TSQualifiedName(Box<'a, TSQualifiedName<'a>>),
    TSTemplateLiteralType(Box<'a, TSTemplateLiteralType<'a>>),
    TSTupleType(Box<'a, TSTupleType<'a>>),
    TSTypeLiteral(Box<'a, TSTypeLiteral<'a>>),
    TSTypeOperatorType(Box<'a, TSTypeOperator<'a>>),
    TSTypePredicate(Box<'a, TSTypePredicate<'a>>),
    TSTypeQuery(Box<'a, TSTypeQuery<'a>>),
    TSTypeReference(Box<'a, TSTypeReference<'a>>),
    TSUnionType(Box<'a, TSUnionType<'a>>),
    // JSDoc
    JSDocNullableType(Box<'a, JSDocNullableType<'a>>),
    JSDocUnknownType(Box<'a, JSDocUnknownType>),
}

impl<'a> TSType<'a> {
    pub fn is_const_type_reference(&self) -> bool {
        matches!(self, TSType::TSTypeReference(reference) if reference.type_name.is_const())
    }
}

/// `SomeType extends OtherType ? TrueType : FalseType;`
///
/// <https://www.typescriptlang.org/docs/handbook/2/conditional-types.html#handbook-content>
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSConditionalType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub check_type: TSType<'a>,
    pub extends_type: TSType<'a>,
    pub true_type: TSType<'a>,
    pub false_type: TSType<'a>,
}

/// string | string[] | (() => string) | { s: string }
///
/// <https://www.typescriptlang.org/docs/handbook/typescript-in-5-minutes-func.html#unions>
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSUnionType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub types: Vec<'a, TSType<'a>>,
}

/// type `ColorfulCircle` = Colorful & Circle;
///
/// <https://www.typescriptlang.org/docs/handbook/2/objects.html#intersection-types>
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSIntersectionType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub types: Vec<'a, TSType<'a>>,
}

/// keyof unique readonly
///
/// <https://www.typescriptlang.org/docs/handbook/2/keyof-types.html>
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTypeOperator<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub operator: TSTypeOperatorOperator,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "lowercase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSTypeOperatorOperator {
    Keyof,
    Unique,
    Readonly,
}

/// `let myArray: string[] = ["hello", "world"];`
///
/// <https://www.typescriptlang.org/docs/handbook/2/objects.html#the-array-type>
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSArrayType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub element_type: TSType<'a>,
}

/// `type I1 = Person["age" | "name"];`
///
/// <https://www.typescriptlang.org/docs/handbook/2/indexed-access-types.html#handbook-content>
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSIndexedAccessType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub object_type: TSType<'a>,
    pub index_type: TSType<'a>,
}

/// type `StringNumberPair` = [string, number];
///
/// <https://www.typescriptlang.org/docs/handbook/2/objects.html#tuple-types>
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTupleType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub element_types: Vec<'a, TSTupleElement<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSNamedTupleMember<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub element_type: TSType<'a>,
    pub label: IdentifierName<'a>,
    pub optional: bool,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSOptionalType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSRestType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSTupleElement<'a> {
    TSType(TSType<'a>),
    TSOptionalType(Box<'a, TSOptionalType<'a>>),
    TSRestType(Box<'a, TSRestType<'a>>),
    TSNamedTupleMember(Box<'a, TSNamedTupleMember<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSAnyKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSStringKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSBooleanKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSNumberKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSNeverKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSUnknownKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSNullKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSUndefinedKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSVoidKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSSymbolKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSThisType {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSObjectKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSBigIntKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

/// type C = A;
/// type D = B.a;
/// type E = D.c.b.a;
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTypeReference<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_name: TSTypeName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// TypeName:
///     IdentifierReference
///     NamespaceName . IdentifierReference
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSTypeName<'a> {
    IdentifierReference(Box<'a, IdentifierReference<'a>>),
    QualifiedName(Box<'a, TSQualifiedName<'a>>),
}

impl<'a> TSTypeName<'a> {
    pub fn get_first_name(name: &TSTypeName<'a>) -> IdentifierReference<'a> {
        match name {
            TSTypeName::IdentifierReference(name) => (*name).clone(),
            TSTypeName::QualifiedName(name) => TSTypeName::get_first_name(&name.left),
        }
    }

    pub fn is_const(&self) -> bool {
        if let TSTypeName::IdentifierReference(ident) = self {
            if ident.name == "const" {
                return true;
            }
        }
        false
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self, Self::IdentifierReference(_))
    }

    pub fn is_qualified_name(&self) -> bool {
        matches!(self, Self::QualifiedName(_))
    }
}

impl GetSpan for TSTypeName<'_> {
    fn span(&self) -> Span {
        match self {
            TSTypeName::IdentifierReference(ident) => ident.span,
            TSTypeName::QualifiedName(name) => name.span,
        }
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSQualifiedName<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub left: TSTypeName<'a>,
    pub right: IdentifierName<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTypeParameterInstantiation<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub params: Vec<'a, TSType<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTypeParameter<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: BindingIdentifier<'a>,
    pub constraint: Option<TSType<'a>>,
    pub default: Option<TSType<'a>>,
    pub r#in: bool,
    pub out: bool,
    pub r#const: bool,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTypeParameterDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub params: Vec<'a, Box<'a, TSTypeParameter<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTypeAliasDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier<'a>,
    pub type_annotation: TSType<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    /// Valid Modifiers: `declare`, `export`
    pub modifiers: Modifiers<'a>,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "lowercase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSAccessibility {
    Private,
    Protected,
    Public,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSClassImplements<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: TSTypeName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// Interface Declaration
///
///   interface `BindingIdentifier` `TypeParameters_opt` `InterfaceExtendsClause_opt` `ObjectType`
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSInterfaceDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier<'a>,
    pub body: Box<'a, TSInterfaceBody<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub extends: Option<Vec<'a, Box<'a, TSInterfaceHeritage<'a>>>>,
    /// Valid Modifiers: `export`, `default`, `declare`
    pub modifiers: Modifiers<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSInterfaceBody<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, TSSignature<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSPropertySignature<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub computed: bool,
    pub optional: bool,
    pub readonly: bool,
    pub key: PropertyKey<'a>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSSignature<'a> {
    TSIndexSignature(Box<'a, TSIndexSignature<'a>>),
    TSPropertySignature(Box<'a, TSPropertySignature<'a>>),
    TSCallSignatureDeclaration(Box<'a, TSCallSignatureDeclaration<'a>>),
    TSConstructSignatureDeclaration(Box<'a, TSConstructSignatureDeclaration<'a>>),
    TSMethodSignature(Box<'a, TSMethodSignature<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSIndexSignature<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub parameters: Vec<'a, Box<'a, TSIndexSignatureName<'a>>>,
    pub type_annotation: Box<'a, TSTypeAnnotation<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSCallSignatureDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub this_param: Option<TSThisParameter<'a>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[derive(Debug, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "lowercase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSMethodSignatureKind {
    Method,
    Get,
    Set,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSMethodSignature<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub computed: bool,
    pub optional: bool,
    pub kind: TSMethodSignatureKind,
    pub this_param: Option<TSThisParameter<'a>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSConstructSignatureDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSIndexSignatureName<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: Atom<'a>,
    pub type_annotation: Box<'a, TSTypeAnnotation<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSInterfaceHeritage<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTypePredicate<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub parameter_name: TSTypePredicateName<'a>,
    pub asserts: bool,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSTypePredicateName<'a> {
    Identifier(IdentifierName<'a>),
    This(TSThisType),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSModuleDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: TSModuleDeclarationName<'a>,
    pub body: TSModuleDeclarationBody<'a>,
    /// The keyword used to define this module declaration
    /// ```text
    /// namespace Foo {}
    /// ^^^^^^^^^
    /// module 'foo' {}
    /// ^^^^^^
    /// declare global {}
    ///         ^^^^^^
    /// ```
    pub kind: TSModuleDeclarationKind,
    /// Valid Modifiers: `declare`, `export`
    pub modifiers: Modifiers<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "lowercase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSModuleDeclarationKind {
    Global,
    Module,
    Namespace,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSModuleDeclarationName<'a> {
    Identifier(IdentifierName<'a>),
    StringLiteral(StringLiteral<'a>),
}

impl<'a> TSModuleDeclarationName<'a> {
    pub fn name(&self) -> &Atom<'a> {
        match self {
            Self::Identifier(ident) => &ident.name,
            Self::StringLiteral(lit) => &lit.value,
        }
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSModuleDeclarationBody<'a> {
    TSModuleDeclaration(Box<'a, TSModuleDeclaration<'a>>),
    TSModuleBlock(Box<'a, TSModuleBlock<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSModuleBlock<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, Statement<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTypeLiteral<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub members: Vec<'a, TSSignature<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSInferType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_parameter: Box<'a, TSTypeParameter<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTypeQuery<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expr_name: TSTypeName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSImportType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub is_type_of: bool,
    pub argument: TSType<'a>,
    pub qualifier: Option<TSTypeName<'a>>,
    pub attributes: Option<TSImportAttributes<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSImportAttributes<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub elements: Vec<'a, TSImportAttribute<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSImportAttribute<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: TSImportAttributeName<'a>,
    pub value: Expression<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSImportAttributeName<'a> {
    Identifier(IdentifierName<'a>),
    StringLiteral(StringLiteral<'a>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSFunctionType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub this_param: Option<TSThisParameter<'a>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Box<'a, TSTypeAnnotation<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSConstructorType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub r#abstract: bool,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Box<'a, TSTypeAnnotation<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSMappedType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_parameter: Box<'a, TSTypeParameter<'a>>,
    pub name_type: Option<TSType<'a>>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub optional: TSMappedTypeModifierOperator,
    pub readonly: TSMappedTypeModifierOperator,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSMappedTypeModifierOperator {
    True,
    #[cfg_attr(feature = "serde", serde(rename = "+"))]
    Plus,
    #[cfg_attr(feature = "serde", serde(rename = "-"))]
    Minus,
    None,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTemplateLiteralType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub quasis: Vec<'a, TemplateElement<'a>>,
    pub types: Vec<'a, TSType<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSAsExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSSatisfiesExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSTypeAssertion<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSImportEqualsDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier<'a>,
    pub module_reference: Box<'a, TSModuleReference<'a>>,
    pub is_export: bool,
    pub import_kind: ImportOrExportKind,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum TSModuleReference<'a> {
    TypeName(TSTypeName<'a>),
    ExternalModuleReference(TSExternalModuleReference<'a>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSExternalModuleReference<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: StringLiteral<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSNonNullExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct Decorator<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum ModifierKind {
    Abstract,
    Accessor,
    Async,
    Const,
    Declare,
    Default,
    Export,
    In,
    Public,
    Private,
    Protected,
    Readonly,
    Static,
    Out,
    Override,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct Modifier {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub kind: ModifierKind,
}

#[derive(Debug, Default, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(transparent))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct Modifiers<'a>(Option<Vec<'a, Modifier>>);

impl<'a> Modifiers<'a> {
    pub fn new(modifiers: Vec<'a, Modifier>) -> Self {
        Self(Some(modifiers))
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn contains(&self, target: ModifierKind) -> bool {
        self.0
            .as_ref()
            .map_or(false, |modifiers| modifiers.iter().any(|modifier| modifier.kind == target))
    }

    pub fn is_contains_declare(&self) -> bool {
        self.contains(ModifierKind::Declare)
    }
}

/// Export Assignment in non-module files
///
/// `export = foo`
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSExportAssignment<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// Namespace Export Declaration in declaration files
///
/// `export as namespace foo`
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSNamespaceExportDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: IdentifierName<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub struct TSInstantiationExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Box<'a, TSTypeParameterInstantiation<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[cfg_attr(all(feature = "serde", feature = "wasm"), derive(tsify::Tsify))]
pub enum ImportOrExportKind {
    Value,
    Type,
}

impl ImportOrExportKind {
    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value)
    }

    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type)
    }
}
