//! TypeScript Definitions
//!
//! [AST Spec](https://github.com/typescript-eslint/typescript-eslint/tree/main/packages/ast-spec)
//! [Archived TypeScript spec](https://github.com/microsoft/TypeScript/blob/3c99d50da5a579d9fa92d02664b1b66d4ff55944/doc/spec-ARCHIVED.md)

// NB: `#[visited_node]` and `#[scope]` attributes on AST nodes do not do anything to the code in this file.
// They are purely markers for codegen used in `oxc_traverse`. See docs in that crate.

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use std::{cell::Cell, hash::Hash};

use oxc_allocator::{Box, Vec};
use oxc_ast_macros::visited_node;
use oxc_span::{Atom, Span};
use oxc_syntax::scope::ScopeId;
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use super::{inherit_variants, js::*, jsx::*, literal::*};

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface TSIndexSignatureName extends Span {
    type: "Identifier",
    name: Atom,
    typeAnnotation: TSTypeAnnotation,
}
"#;

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSThisParameter<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub this: IdentifierName<'a>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

/// Enum Declaration
///
/// `const_opt` enum `BindingIdentifier` { `EnumBody_opt` }
#[visited_node]
#[scope]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSEnumDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier<'a>,
    #[scope(enter_before)]
    pub members: Vec<'a, TSEnumMember<'a>>,
    pub r#const: bool,
    pub declare: bool,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSEnumMember<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: TSEnumMemberName<'a>,
    pub initializer: Option<Expression<'a>>,
}

inherit_variants! {
/// TS Enum Member Name
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[visited_node]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum TSEnumMemberName<'a> {
    StaticIdentifier(Box<'a, IdentifierName<'a>>) = 64,
    StaticStringLiteral(Box<'a, StringLiteral<'a>>) = 65,
    // Invalid Grammar `enum E { 1 }`
    StaticNumericLiteral(Box<'a, NumericLiteral<'a>>) = 66,
    // Invalid Grammar `enum E { [computed] }`
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeAnnotation<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSLiteralType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub literal: TSLiteral<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged, rename_all = "camelCase"))]
pub enum TSLiteral<'a> {
    BooleanLiteral(Box<'a, BooleanLiteral>),
    NullLiteral(Box<'a, NullLiteral>),
    NumericLiteral(Box<'a, NumericLiteral<'a>>),
    BigIntLiteral(Box<'a, BigIntLiteral<'a>>),
    RegExpLiteral(Box<'a, RegExpLiteral<'a>>),
    StringLiteral(Box<'a, StringLiteral<'a>>),
    TemplateLiteral(Box<'a, TemplateLiteral<'a>>),
    UnaryExpression(Box<'a, UnaryExpression<'a>>),
}

#[visited_node]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged, rename_all = "camelCase"))]
pub enum TSType<'a> {
    // Keyword
    TSAnyKeyword(Box<'a, TSAnyKeyword>) = 0,
    TSBigIntKeyword(Box<'a, TSBigIntKeyword>) = 1,
    TSBooleanKeyword(Box<'a, TSBooleanKeyword>) = 2,
    TSIntrinsicKeyword(Box<'a, TSIntrinsicKeyword>) = 3,
    TSNeverKeyword(Box<'a, TSNeverKeyword>) = 4,
    TSNullKeyword(Box<'a, TSNullKeyword>) = 5,
    TSNumberKeyword(Box<'a, TSNumberKeyword>) = 6,
    TSObjectKeyword(Box<'a, TSObjectKeyword>) = 7,
    TSStringKeyword(Box<'a, TSStringKeyword>) = 8,
    TSSymbolKeyword(Box<'a, TSSymbolKeyword>) = 9,
    TSUndefinedKeyword(Box<'a, TSUndefinedKeyword>) = 11,
    TSUnknownKeyword(Box<'a, TSUnknownKeyword>) = 12,
    TSVoidKeyword(Box<'a, TSVoidKeyword>) = 13,
    // Compound
    TSArrayType(Box<'a, TSArrayType<'a>>) = 14,
    TSConditionalType(Box<'a, TSConditionalType<'a>>) = 15,
    TSConstructorType(Box<'a, TSConstructorType<'a>>) = 16,
    TSFunctionType(Box<'a, TSFunctionType<'a>>) = 17,
    TSImportType(Box<'a, TSImportType<'a>>) = 18,
    TSIndexedAccessType(Box<'a, TSIndexedAccessType<'a>>) = 19,
    TSInferType(Box<'a, TSInferType<'a>>) = 20,
    TSIntersectionType(Box<'a, TSIntersectionType<'a>>) = 21,
    TSLiteralType(Box<'a, TSLiteralType<'a>>) = 22,
    TSMappedType(Box<'a, TSMappedType<'a>>) = 23,
    TSNamedTupleMember(Box<'a, TSNamedTupleMember<'a>>) = 24,
    TSQualifiedName(Box<'a, TSQualifiedName<'a>>) = 25,
    TSTemplateLiteralType(Box<'a, TSTemplateLiteralType<'a>>) = 26,
    TSThisType(Box<'a, TSThisType>) = 10,
    TSTupleType(Box<'a, TSTupleType<'a>>) = 27,
    TSTypeLiteral(Box<'a, TSTypeLiteral<'a>>) = 28,
    TSTypeOperatorType(Box<'a, TSTypeOperator<'a>>) = 29,
    TSTypePredicate(Box<'a, TSTypePredicate<'a>>) = 30,
    TSTypeQuery(Box<'a, TSTypeQuery<'a>>) = 31,
    TSTypeReference(Box<'a, TSTypeReference<'a>>) = 32,
    TSUnionType(Box<'a, TSUnionType<'a>>) = 33,
    // JSDoc
    JSDocNullableType(Box<'a, JSDocNullableType<'a>>) = 34,
    JSDocNonNullableType(Box<'a, JSDocNonNullableType<'a>>) = 35,
    JSDocUnknownType(Box<'a, JSDocUnknownType>) = 36,
}

/// Macro for matching `TSType`'s variants.
#[macro_export]
macro_rules! match_ts_type {
    ($ty:ident) => {
        $ty::TSAnyKeyword(_)
            | $ty::TSBigIntKeyword(_)
            | $ty::TSBooleanKeyword(_)
            | $ty::TSIntrinsicKeyword(_)
            | $ty::TSNeverKeyword(_)
            | $ty::TSNullKeyword(_)
            | $ty::TSNumberKeyword(_)
            | $ty::TSObjectKeyword(_)
            | $ty::TSStringKeyword(_)
            | $ty::TSSymbolKeyword(_)
            | $ty::TSThisType(_)
            | $ty::TSUndefinedKeyword(_)
            | $ty::TSUnknownKeyword(_)
            | $ty::TSVoidKeyword(_)
            | $ty::TSArrayType(_)
            | $ty::TSConditionalType(_)
            | $ty::TSConstructorType(_)
            | $ty::TSFunctionType(_)
            | $ty::TSImportType(_)
            | $ty::TSIndexedAccessType(_)
            | $ty::TSInferType(_)
            | $ty::TSIntersectionType(_)
            | $ty::TSLiteralType(_)
            | $ty::TSMappedType(_)
            | $ty::TSNamedTupleMember(_)
            | $ty::TSQualifiedName(_)
            | $ty::TSTemplateLiteralType(_)
            | $ty::TSTupleType(_)
            | $ty::TSTypeLiteral(_)
            | $ty::TSTypeOperatorType(_)
            | $ty::TSTypePredicate(_)
            | $ty::TSTypeQuery(_)
            | $ty::TSTypeReference(_)
            | $ty::TSUnionType(_)
            | $ty::JSDocNullableType(_)
            | $ty::JSDocNonNullableType(_)
            | $ty::JSDocUnknownType(_)
    };
}
pub use match_ts_type;

/// `SomeType extends OtherType ? TrueType : FalseType;`
///
/// <https://www.typescriptlang.org/docs/handbook/2/conditional-types.html#handbook-content>
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSConditionalType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub check_type: TSType<'a>,
    pub extends_type: TSType<'a>,
    pub true_type: TSType<'a>,
    pub false_type: TSType<'a>,
}

/// string | string[] | (() => string) | { s: string }
///
/// <https://www.typescriptlang.org/docs/handbook/typescript-in-5-minutes-func.html#unions>
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSUnionType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub types: Vec<'a, TSType<'a>>,
}

/// type `ColorfulCircle` = Colorful & Circle;
///
/// <https://www.typescriptlang.org/docs/handbook/2/objects.html#intersection-types>
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSIntersectionType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub types: Vec<'a, TSType<'a>>,
}

/// keyof unique readonly
///
/// <https://www.typescriptlang.org/docs/handbook/2/keyof-types.html>
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeOperator<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub operator: TSTypeOperatorOperator,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum TSTypeOperatorOperator {
    Keyof,
    Unique,
    Readonly,
}

/// `let myArray: string[] = ["hello", "world"];`
///
/// <https://www.typescriptlang.org/docs/handbook/2/objects.html#the-array-type>
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSArrayType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub element_type: TSType<'a>,
}

/// `type I1 = Person["age" | "name"];`
///
/// <https://www.typescriptlang.org/docs/handbook/2/indexed-access-types.html#handbook-content>
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSIndexedAccessType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub object_type: TSType<'a>,
    pub index_type: TSType<'a>,
}

/// type `StringNumberPair` = [string, number];
///
/// <https://www.typescriptlang.org/docs/handbook/2/objects.html#tuple-types>
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTupleType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub element_types: Vec<'a, TSTupleElement<'a>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSNamedTupleMember<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub element_type: TSTupleElement<'a>,
    pub label: IdentifierName<'a>,
    pub optional: bool,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSOptionalType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSRestType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

inherit_variants! {
/// TS Tuple Element
///
/// Inherits variants from [`TSType`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[visited_node]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged, rename_all = "camelCase"))]
pub enum TSTupleElement<'a> {
    // Discriminants start at 64, so that `TSTupleElement::is_ts_type` is a single
    // bitwise AND operation on the discriminant (`discriminant & 63 != 0`).
    TSOptionalType(Box<'a, TSOptionalType<'a>>) = 64,
    TSRestType(Box<'a, TSRestType<'a>>) = 65,
    // `TSType` variants added here by `inherit_variants!` macro
    @inherit TSType
}
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSAnyKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSStringKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSBooleanKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSNumberKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSNeverKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

/// `type Uppercase<T extends character> = intrinsic;`
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSIntrinsicKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSUnknownKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSNullKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSUndefinedKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSVoidKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSSymbolKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSThisType {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSObjectKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSBigIntKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

/// type C = A;
/// type D = B.a;
/// type E = D.c.b.a;
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeReference<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_name: TSTypeName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// TypeName:
///     IdentifierReference
///     NamespaceName . IdentifierReference
#[visited_node]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum TSTypeName<'a> {
    IdentifierReference(Box<'a, IdentifierReference<'a>>) = 0,
    QualifiedName(Box<'a, TSQualifiedName<'a>>) = 1,
}

/// Macro for matching `TSTypeName`'s variants.
#[macro_export]
macro_rules! match_ts_type_name {
    ($ty:ident) => {
        $ty::IdentifierReference(_) | $ty::QualifiedName(_)
    };
}
pub use match_ts_type_name;

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSQualifiedName<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub left: TSTypeName<'a>,
    pub right: IdentifierName<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeParameterInstantiation<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub params: Vec<'a, TSType<'a>>,
}

#[visited_node]
#[scope]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeParameter<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: BindingIdentifier<'a>,
    pub constraint: Option<TSType<'a>>,
    pub default: Option<TSType<'a>>,
    pub r#in: bool,
    pub out: bool,
    pub r#const: bool,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeParameterDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub params: Vec<'a, TSTypeParameter<'a>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeAliasDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier<'a>,
    pub type_annotation: TSType<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub declare: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum TSAccessibility {
    Private,
    Protected,
    Public,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSClassImplements<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: TSTypeName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// Interface Declaration
///
///   interface `BindingIdentifier` `TypeParameters_opt` `InterfaceExtendsClause_opt` `ObjectType`
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInterfaceDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier<'a>,
    pub body: Box<'a, TSInterfaceBody<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
    pub declare: bool,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInterfaceBody<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, TSSignature<'a>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSPropertySignature<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub computed: bool,
    pub optional: bool,
    pub readonly: bool,
    pub key: PropertyKey<'a>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged, rename_all = "camelCase"))]
pub enum TSSignature<'a> {
    TSIndexSignature(Box<'a, TSIndexSignature<'a>>),
    TSPropertySignature(Box<'a, TSPropertySignature<'a>>),
    TSCallSignatureDeclaration(Box<'a, TSCallSignatureDeclaration<'a>>),
    TSConstructSignatureDeclaration(Box<'a, TSConstructSignatureDeclaration<'a>>),
    TSMethodSignature(Box<'a, TSMethodSignature<'a>>),
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSIndexSignature<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub parameters: Vec<'a, TSIndexSignatureName<'a>>,
    pub type_annotation: Box<'a, TSTypeAnnotation<'a>>,
    pub readonly: bool,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSCallSignatureDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub this_param: Option<TSThisParameter<'a>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum TSMethodSignatureKind {
    Method,
    Get,
    Set,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSMethodSignature<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
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

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSConstructSignatureDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(
    feature = "serialize",
    serde(tag = "type", rename = "Identifier", rename_all = "camelCase")
)]
pub struct TSIndexSignatureName<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: Atom<'a>,
    pub type_annotation: Box<'a, TSTypeAnnotation<'a>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInterfaceHeritage<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypePredicate<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub parameter_name: TSTypePredicateName<'a>,
    pub asserts: bool,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged, rename_all = "camelCase"))]
pub enum TSTypePredicateName<'a> {
    Identifier(Box<'a, IdentifierName<'a>>),
    This(TSThisType),
}

#[visited_node]
#[scope(
    flags(ScopeFlags::TsModuleBlock),
    strict_if(self.body.as_ref().is_some_and(|body| body.is_strict())),
)]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSModuleDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: TSModuleDeclarationName<'a>,
    #[scope(enter_before)]
    pub body: Option<TSModuleDeclarationBody<'a>>,
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
    pub declare: bool,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum TSModuleDeclarationKind {
    Global,
    Module,
    Namespace,
}

impl TSModuleDeclarationKind {
    pub fn is_global(self) -> bool {
        matches!(self, TSModuleDeclarationKind::Global)
    }
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum TSModuleDeclarationName<'a> {
    Identifier(IdentifierName<'a>),
    StringLiteral(StringLiteral<'a>),
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum TSModuleDeclarationBody<'a> {
    TSModuleDeclaration(Box<'a, TSModuleDeclaration<'a>>),
    TSModuleBlock(Box<'a, TSModuleBlock<'a>>),
}

// See serializer in serialize.rs
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSModuleBlock<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub directives: Vec<'a, Directive<'a>>,
    pub body: Vec<'a, Statement<'a>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeLiteral<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub members: Vec<'a, TSSignature<'a>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInferType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_parameter: Box<'a, TSTypeParameter<'a>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeQuery<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expr_name: TSTypeQueryExprName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

inherit_variants! {
/// TS Type Query Expr Name
///
/// Inherits variants from [`TSTypeName`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[visited_node]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum TSTypeQueryExprName<'a> {
    TSImportType(Box<'a, TSImportType<'a>>) = 2,
    // `TSTypeName` variants added here by `inherit_variants!` macro
    @inherit TSTypeName
}
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSImportType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub is_type_of: bool, // `typeof import("foo")`
    pub parameter: TSType<'a>,
    pub qualifier: Option<TSTypeName<'a>>,
    pub attributes: Option<TSImportAttributes<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSImportAttributes<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub elements: Vec<'a, TSImportAttribute<'a>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSImportAttribute<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: TSImportAttributeName<'a>,
    pub value: Expression<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum TSImportAttributeName<'a> {
    Identifier(IdentifierName<'a>),
    StringLiteral(StringLiteral<'a>),
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSFunctionType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub this_param: Option<TSThisParameter<'a>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Box<'a, TSTypeAnnotation<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSConstructorType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub r#abstract: bool,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Box<'a, TSTypeAnnotation<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSMappedType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_parameter: Box<'a, TSTypeParameter<'a>>,
    pub name_type: Option<TSType<'a>>,
    pub type_annotation: Option<TSType<'a>>,
    pub optional: TSMappedTypeModifierOperator,
    pub readonly: TSMappedTypeModifierOperator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
pub enum TSMappedTypeModifierOperator {
    True,
    #[cfg_attr(feature = "serialize", serde(rename = "+"))]
    Plus,
    #[cfg_attr(feature = "serialize", serde(rename = "-"))]
    Minus,
    None,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTemplateLiteralType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub quasis: Vec<'a, TemplateElement<'a>>,
    pub types: Vec<'a, TSType<'a>>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSAsExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSSatisfiesExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeAssertion<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSImportEqualsDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: BindingIdentifier<'a>,
    pub module_reference: TSModuleReference<'a>,
    pub import_kind: ImportOrExportKind,
}

inherit_variants! {
/// TS Module Reference
///
/// Inherits variants from [`TSTypeName`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[visited_node]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged, rename_all = "camelCase"))]
pub enum TSModuleReference<'a> {
    ExternalModuleReference(Box<'a, TSExternalModuleReference<'a>>) = 2,
    // `TSTypeName` variants added here by `inherit_variants!` macro
    @inherit TSTypeName
}
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSExternalModuleReference<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: StringLiteral<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSNonNullExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct Decorator<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// Export Assignment in non-module files
///
/// `export = foo`
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSExportAssignment<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// Namespace Export Declaration in declaration files
///
/// `export as namespace foo`
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSNamespaceExportDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: IdentifierName<'a>,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInstantiationExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Box<'a, TSTypeParameterInstantiation<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
pub enum ImportOrExportKind {
    Value,
    Type,
}

// [`JSDoc`](https://github.com/microsoft/TypeScript/blob/54a554d8af2657630307cbfa8a3e4f3946e36507/src/compiler/types.ts#L393)

/// `type foo = ty?` or `type foo = ?ty`
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct JSDocNullableType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
    pub postfix: bool,
}

/// `type foo = ty!` or `type foo = !ty`
#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct JSDocNonNullableType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
    pub postfix: bool,
}

#[visited_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct JSDocUnknownType {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}
