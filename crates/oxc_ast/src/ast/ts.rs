//! [AST Spec](https://github.com/typescript-eslint/typescript-eslint/tree/main/packages/ast-spec)

use oxc_allocator::{Box, Vec};
use oxc_span::{Atom, Span};
#[cfg(feature = "serde")]
use serde::Serialize;

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

/// Enum Declaration
///
/// `const_opt` enum `BindingIdentifier` { `EnumBody_opt` }
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSEnumDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier,
    pub members: Vec<'a, TSEnumMember<'a>>,
    /// Valid Modifiers: `const`, `export`, `declare`
    pub modifiers: Modifiers<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSEnumMember<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: TSEnumMemberName<'a>,
    pub initializer: Option<Expression<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum TSEnumMemberName<'a> {
    Identifier(IdentifierName),
    StringLiteral(StringLiteral),
    // Invalid Grammar `enum E { [computed] }`
    ComputedPropertyName(Expression<'a>),
    // Invalid Grammar `enum E { 1 }`
    NumberLiteral(NumberLiteral<'a>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeAnnotation<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSLiteralType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub literal: TSLiteral<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
pub enum TSLiteral<'a> {
    BooleanLiteral(Box<'a, BooleanLiteral>),
    NullLiteral(Box<'a, NullLiteral>),
    NumberLiteral(Box<'a, NumberLiteral<'a>>),
    BigintLiteral(Box<'a, BigintLiteral>),
    RegExpLiteral(Box<'a, RegExpLiteral>),
    StringLiteral(Box<'a, StringLiteral>),
    TemplateLiteral(Box<'a, TemplateLiteral<'a>>),
    UnaryExpression(Box<'a, UnaryExpression<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
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
    TSThisKeyword(Box<'a, TSThisKeyword>),
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
    TSTypeOperatorType(Box<'a, TSTypeOperatorType<'a>>),
    TSTypePredicate(Box<'a, TSTypePredicate<'a>>),
    TSTypeQuery(Box<'a, TSTypeQuery<'a>>),
    TSTypeReference(Box<'a, TSTypeReference<'a>>),
    TSUnionType(Box<'a, TSUnionType<'a>>),
    // JSDoc
    JSDocNullableType(Box<'a, JSDocNullableType<'a>>),
    JSDocUnknownType(Box<'a, JSDocUnknownType>),
}

impl<'a> TSType<'a> {
    #[must_use]
    pub fn is_const_type_reference(&self) -> bool {
        matches!(self, TSType::TSTypeReference(reference) if reference.type_name.is_const())
    }
}

/// `SomeType extends OtherType ? TrueType : FalseType;`
///
/// <https://www.typescriptlang.org/docs/handbook/2/conditional-types.html#handbook-content>
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
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
pub struct TSIntersectionType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub types: Vec<'a, TSType<'a>>,
}

/// keyof unique readonly
///
/// <https://www.typescriptlang.org/docs/handbook/2/keyof-types.html>
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename = "TSTypeOperator"))]
pub struct TSTypeOperatorType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub operator: TSTypeOperator,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "lowercase"))]
pub enum TSTypeOperator {
    Keyof,
    Unique,
    Readonly,
}

impl TSTypeOperator {
    #[must_use]
    pub fn from_src(src: &str) -> Option<Self> {
        match src {
            "keyof" => Some(Self::Keyof),
            "unique" => Some(Self::Unique),
            "readonly" => Some(Self::Readonly),
            _ => None,
        }
    }
}

/// `let myArray: string[] = ["hello", "world"];`
///
/// <https://www.typescriptlang.org/docs/handbook/2/objects.html#the-array-type>
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
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
pub struct TSTupleType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub element_types: Vec<'a, TSTupleElement<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSNamedTupleMember<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub element_type: TSType<'a>,
    pub label: IdentifierName,
    pub optional: bool,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSOptionalType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSRestType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
pub enum TSTupleElement<'a> {
    TSType(TSType<'a>),
    TSOptionalType(Box<'a, TSOptionalType<'a>>),
    TSRestType(Box<'a, TSRestType<'a>>),
    TSNamedTupleMember(Box<'a, TSNamedTupleMember<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSAnyKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSStringKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSBooleanKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSNumberKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSNeverKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSUnknownKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSNullKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSUndefinedKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSVoidKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSSymbolKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSThisKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSObjectKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct TSBigIntKeyword {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

/// type C = A;
/// type D = B.a;
/// type E = D.c.b.a;
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeReference<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_name: TSTypeName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum TSTypeName<'a> {
    IdentifierName(Box<'a, IdentifierName>),
    QualifiedName(Box<'a, TSQualifiedName<'a>>),
}

impl<'a> TSTypeName<'a> {
    #[must_use]
    pub fn get_first_name(name: &TSTypeName) -> IdentifierName {
        match name {
            TSTypeName::IdentifierName(name) => (*name).clone(),
            TSTypeName::QualifiedName(name) => TSTypeName::get_first_name(&name.left),
        }
    }

    #[must_use]
    pub fn is_const(&self) -> bool {
        if let TSTypeName::IdentifierName(ident) = self {
            if ident.name == "const" {
                return true;
            }
        }
        false
    }

    #[must_use]
    pub fn is_identifier(&self) -> bool {
        matches!(self, Self::IdentifierName(_))
    }

    #[must_use]
    pub fn is_qualified_name(&self) -> bool {
        matches!(self, Self::QualifiedName(_))
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSQualifiedName<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub left: TSTypeName<'a>,
    pub right: IdentifierName,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeParameterInstantiation<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub params: Vec<'a, TSType<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeParameter<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: BindingIdentifier,
    pub constraint: Option<TSType<'a>>,
    pub default: Option<TSType<'a>>,
    pub r#in: bool,
    pub out: bool,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeParameterDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub params: Vec<'a, Box<'a, TSTypeParameter<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeAliasDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier,
    pub type_annotation: TSType<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    /// Valid Modifiers: `declare`, `export`
    pub modifiers: Modifiers<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSAbstractMethodDefinition<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub method_definition: MethodDefinition<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSAbstractPropertyDefinition<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub property_definition: PropertyDefinition<'a>,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "lowercase"))]
pub enum TSAccessibility {
    Private,
    Protected,
    Public,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
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
pub struct TSInterfaceDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier,
    pub body: Box<'a, TSInterfaceBody<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub extends: Option<Vec<'a, Box<'a, TSInterfaceHeritage<'a>>>>,
    /// Valid Modifiers: `export`, `default`, `declare`
    pub modifiers: Modifiers<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInterfaceBody<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, TSSignature<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
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
pub enum TSSignature<'a> {
    TSIndexSignature(Box<'a, TSIndexSignature<'a>>),
    TSPropertySignature(Box<'a, TSPropertySignature<'a>>),
    TSCallSignatureDeclaration(Box<'a, TSCallSignatureDeclaration<'a>>),
    TSConstructSignatureDeclaration(Box<'a, TSConstructSignatureDeclaration<'a>>),
    TSMethodSignature(Box<'a, TSMethodSignature<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSIndexSignature<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub parameters: Vec<'a, Box<'a, TSIndexSignatureName<'a>>>,
    pub type_annotation: Box<'a, TSTypeAnnotation<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSCallSignatureDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[derive(Debug, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "lowercase"))]
pub enum TSMethodSignatureKind {
    Method,
    Get,
    Set,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSMethodSignature<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub computed: bool,
    pub optional: bool,
    pub kind: TSMethodSignatureKind,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSConstructSignatureDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSIndexSignatureName<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub name: Atom,
    pub type_annotation: Box<'a, TSTypeAnnotation<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInterfaceHeritage<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypePredicate<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub parameter_name: TSTypePredicateName,
    pub asserts: bool,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
pub enum TSTypePredicateName {
    Identifier(IdentifierName),
    This(TSThisKeyword),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSModuleDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: TSModuleDeclarationName,
    pub body: TSModuleDeclarationBody<'a>,
    /// Valid Modifiers: `declare`, `export`
    pub modifiers: Modifiers<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum TSModuleDeclarationName {
    Identifier(IdentifierName),
    StringLiteral(StringLiteral),
}

impl TSModuleDeclarationName {
    #[must_use]
    pub fn name(&self) -> &Atom {
        match self {
            Self::Identifier(ident) => &ident.name,
            Self::StringLiteral(lit) => &lit.value,
        }
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged))]
pub enum TSModuleDeclarationBody<'a> {
    TSModuleDeclaration(Box<'a, TSModuleDeclaration<'a>>),
    TSModuleBlock(Box<'a, TSModuleBlock<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSModuleBlock<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, Statement<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeLiteral<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub members: Vec<'a, TSSignature<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInferType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_parameter: Box<'a, TSTypeParameter<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeQuery<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expr_name: TSTypeName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSImportType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub is_type_of: bool,
    pub parameter: TSType<'a>,
    pub qualifier: Option<TSTypeName<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSFunctionType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Box<'a, TSTypeAnnotation<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
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
pub struct TSMappedType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub type_parameter: Box<'a, TSTypeParameter<'a>>,
    pub name_type: Option<TSType<'a>>,
    pub type_annotation: TSType<'a>,
    pub optional: TSMappedTypeModifierOperator,
    pub readonly: TSMappedTypeModifierOperator,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
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
pub struct TSTemplateLiteralType<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub quasis: Vec<'a, TemplateElement>,
    pub types: Vec<'a, TSType<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSAsExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSSatisfiesExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeAssertion<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSImportEqualsDeclaration<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier,
    pub module_reference: Box<'a, TSModuleReference<'a>>,
    pub is_export: bool,
    pub import_kind: ImportOrExportKind,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(untagged, rename_all = "camelCase"))]
pub enum TSModuleReference<'a> {
    TypeName(TSTypeName<'a>),
    ExternalModuleReference(TSExternalModuleReference),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSExternalModuleReference {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: StringLiteral,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSNonNullExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct Decorator<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
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
pub struct Modifier {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub kind: ModifierKind,
}

#[derive(Debug, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(transparent))]
pub struct Modifiers<'a>(Option<Vec<'a, Modifier>>);

impl<'a> Modifiers<'a> {
    #[must_use]
    pub fn new(modifiers: Vec<'a, Modifier>) -> Self {
        Self(Some(modifiers))
    }

    #[must_use]
    pub fn empty() -> Self {
        Self(None)
    }

    #[must_use]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    #[must_use]
    pub fn contains(&self, target: ModifierKind) -> bool {
        self.0
            .as_ref()
            .map_or(false, |modifiers| modifiers.iter().any(|modifier| modifier.kind == target))
    }
}

/// Export Assignment in non-module files
///
/// `export = foo`
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
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
pub struct TSNamespaceExportDeclaration {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub id: IdentifierName,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInstantiationExpression<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Box<'a, TSTypeParameterInstantiation<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename_all = "camelCase"))]
pub enum ImportOrExportKind {
    Value,
    Type,
}

impl ImportOrExportKind {
    #[must_use]
    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value)
    }

    #[must_use]
    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type)
    }
}
