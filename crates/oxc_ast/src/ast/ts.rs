//! TypeScript Definitions
//!
//! [AST Spec](https://github.com/typescript-eslint/typescript-eslint/tree/main/packages/ast-spec)
//! [Archived TypeScript spec](https://github.com/microsoft/TypeScript/blob/3c99d50da5a579d9fa92d02664b1b66d4ff55944/doc/spec-ARCHIVED.md)

// NB: `#[span]`, `#[scope(...)]` and `#[visit(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in
// `tasks/ast_codegen` and `crates/oxc_traverse/scripts`. See docs in those crates.

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use std::{cell::Cell, hash::Hash};

use oxc_allocator::{Box, Vec};
use oxc_ast_macros::ast;
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

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSThisParameter<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub this: IdentifierName<'a>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

/// Enum Declaration
///
/// `const_opt` enum `BindingIdentifier` { `EnumBody_opt` }
///
/// ## Examples
///
/// ```ts
/// enum Foo {
///     A,
///     B
/// }
/// // `Bar` has `r#const` set to `true`
/// const enum Bar {
///     A,
///     B
/// }
/// ```
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSEnumDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub id: BindingIdentifier<'a>,
    #[scope(enter_before)]
    pub members: Vec<'a, TSEnumMember<'a>>,
    pub r#const: bool,
    pub declare: bool,
    pub scope_id: Cell<Option<ScopeId>>,
}

/// Enum Member
///
/// ## Example
///
/// ```ts
/// enum Foo {
/// //  _ id
///     A = 1,
/// //      ^ initializer
///     B // initializer will be `None`
///
/// }
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSEnumMember<'a> {
    #[serde(flatten)]
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
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum TSEnumMemberName<'a> {
    StaticIdentifier(Box<'a, IdentifierName<'a>>) = 64,
    StaticStringLiteral(Box<'a, StringLiteral<'a>>) = 65,
    StaticTemplateLiteral(Box<'a, TemplateLiteral<'a>>) = 66,
    // Invalid Grammar `enum E { 1 }`
    StaticNumericLiteral(Box<'a, NumericLiteral<'a>>) = 67,
    // Invalid Grammar `enum E { [computed] }`
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// TypeScript Type Annotation
///
/// An annotation on a variable declaration, parameter, etc.
///
/// ## Example
/// ```ts
/// const x: number = 1;
/// //     ^^^^^^^^
///
/// function foo(x: number): number { return x; }
/// //            ^^^^^^^^ ^^^^^^^^
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTypeAnnotation<'a> {
    #[serde(flatten)]
    /// starts at the `:` token and ends at the end of the type annotation
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

/// TypeScript Literal Type
///
/// A type that is a literal value. Wraps a [`TSLiteral`].
///
/// ## Example
/// ```ts
/// const x: 'foo' = 'foo';
/// //       ^^^^^
///
/// type NonZero<N> = N extends 0 ? never : N;
/// //                          ^
/// type Three = NonZero<3>;
/// //                   ^
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSLiteralType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub literal: TSLiteral<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged, rename_all = "camelCase")]
pub enum TSLiteral<'a> {
    BooleanLiteral(Box<'a, BooleanLiteral>) = 0,
    NullLiteral(Box<'a, NullLiteral>) = 1,
    NumericLiteral(Box<'a, NumericLiteral<'a>>) = 2,
    BigIntLiteral(Box<'a, BigIntLiteral<'a>>) = 3,
    RegExpLiteral(Box<'a, RegExpLiteral<'a>>) = 4,
    StringLiteral(Box<'a, StringLiteral<'a>>) = 5,
    TemplateLiteral(Box<'a, TemplateLiteral<'a>>) = 6,
    UnaryExpression(Box<'a, UnaryExpression<'a>>) = 7,
}

/// TypeScript Type
///
/// This is the root-level type for TypeScript types, kind of like [`Expression`] is for
/// expressions.
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged, rename_all = "camelCase")]
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
    TSParenthesizedType(Box<'a, TSParenthesizedType<'a>>) = 34,
    // JSDoc
    JSDocNullableType(Box<'a, JSDocNullableType<'a>>) = 35,
    JSDocNonNullableType(Box<'a, JSDocNonNullableType<'a>>) = 36,
    JSDocUnknownType(Box<'a, JSDocUnknownType>) = 37,
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
            | $ty::TSParenthesizedType(_)
            | $ty::JSDocNullableType(_)
            | $ty::JSDocNonNullableType(_)
            | $ty::JSDocUnknownType(_)
    };
}
pub use match_ts_type;

/// TypeScript Conditional Type
///
/// ## Example
///
/// ```ts
/// SomeType extends OtherType ? TrueType : FalseType;
/// ```
///
/// <https://www.typescriptlang.org/docs/handbook/2/conditional-types.html#handbook-content>
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSConditionalType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub check_type: TSType<'a>,
    pub extends_type: TSType<'a>,
    pub true_type: TSType<'a>,
    pub false_type: TSType<'a>,
    pub scope_id: Cell<Option<ScopeId>>,
}

/// TypeScript Union Type
///
/// ## Example
///
/// ```ts
///  string | string[] | (() => string) | { s: string }
/// ```
///
/// <https://www.typescriptlang.org/docs/handbook/typescript-in-5-minutes-func.html#unions>
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSUnionType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub types: Vec<'a, TSType<'a>>,
}

/// type `ColorfulCircle` = Colorful & Circle;
///
/// <https://www.typescriptlang.org/docs/handbook/2/objects.html#intersection-types>
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSIntersectionType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub types: Vec<'a, TSType<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSParenthesizedType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

/// TypeScript Type Operators
///
/// Includes
/// - `keyof`
/// - `unique`
/// - `readonly`
///
/// <https://www.typescriptlang.org/docs/handbook/2/keyof-types.html>
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTypeOperator<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub operator: TSTypeOperatorOperator,
    pub type_annotation: TSType<'a>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum TSTypeOperatorOperator {
    Keyof = 0,
    Unique = 1,
    Readonly = 2,
}

/// TypeScript Array Type
///
/// Does not include tuple types, which are stored as [`TSTupleType`].
///
/// ## Example
///
/// ```ts
/// let myArray: string[] = ["hello", "world"];
/// ```
///
/// <https://www.typescriptlang.org/docs/handbook/2/objects.html#the-array-type>
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSArrayType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub element_type: TSType<'a>,
}

/// TypeScript Index Access Type
///
/// This is the type equivalent to expression member access.
///
/// ## Example
///
/// ```ts
/// type I1 = Person["age" | "name"];
/// ```
///
/// <https://www.typescriptlang.org/docs/handbook/2/indexed-access-types.html#handbook-content>
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSIndexedAccessType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub object_type: TSType<'a>,
    pub index_type: TSType<'a>,
}

/// TypeScript Tuple Type
///
/// ## Example
///
/// ```ts
/// type `StringNumberPair` = [string, number];
/// ```
///
/// <https://www.typescriptlang.org/docs/handbook/2/objects.html#tuple-types>
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTupleType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub element_types: Vec<'a, TSTupleElement<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSNamedTupleMember<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub element_type: TSTupleElement<'a>,
    pub label: IdentifierName<'a>,
    pub optional: bool,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSOptionalType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSRestType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

inherit_variants! {
/// TS Tuple Element
///
/// Inherits variants from [`TSType`]. See [`ast` module docs] for explanation of inheritance.
///
/// See [`TSNamedTupleMember`] for named tuple elements.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged, rename_all = "camelCase")]
pub enum TSTupleElement<'a> {
    // Discriminants start at 64, so that `TSTupleElement::is_ts_type` is a single
    // bitwise AND operation on the discriminant (`discriminant & 63 != 0`).
    TSOptionalType(Box<'a, TSOptionalType<'a>>) = 64,
    TSRestType(Box<'a, TSRestType<'a>>) = 65,
    // `TSType` variants added here by `inherit_variants!` macro
    @inherit TSType
}
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSAnyKeyword {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSStringKeyword {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSBooleanKeyword {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSNumberKeyword {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSNeverKeyword {
    #[serde(flatten)]
    pub span: Span,
}

/// `type Uppercase<T extends character> = intrinsic;`
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSIntrinsicKeyword {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSUnknownKeyword {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSNullKeyword {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSUndefinedKeyword {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSVoidKeyword {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSSymbolKeyword {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSThisType {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSObjectKeyword {
    #[serde(flatten)]
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TSBigIntKeyword {
    #[serde(flatten)]
    pub span: Span,
}

/// type C = A;
/// type D = B.a;
/// type E = D.c.b.a;
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTypeReference<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub type_name: TSTypeName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// TypeName:
///     IdentifierReference
///     NamespaceName . IdentifierReference
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
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

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSQualifiedName<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub left: TSTypeName<'a>,
    pub right: IdentifierName<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTypeParameterInstantiation<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub params: Vec<'a, TSType<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTypeParameter<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: BindingIdentifier<'a>,
    pub constraint: Option<TSType<'a>>,
    pub default: Option<TSType<'a>>,
    pub r#in: bool,
    pub out: bool,
    pub r#const: bool,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTypeParameterDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub params: Vec<'a, TSTypeParameter<'a>>,
}

#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTypeAliasDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub id: BindingIdentifier<'a>,
    #[scope(enter_before)]
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub type_annotation: TSType<'a>,
    pub declare: bool,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum TSAccessibility {
    Private = 0,
    Protected = 1,
    Public = 2,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSClassImplements<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: TSTypeName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// Interface Declaration
///
///   interface `BindingIdentifier` `TypeParameters_opt` `InterfaceExtendsClause_opt` `ObjectType`
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSInterfaceDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// The identifier (name) of the interface.
    pub id: BindingIdentifier<'a>,
    #[scope(enter_before)]
    pub extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub body: Box<'a, TSInterfaceBody<'a>>,
    pub declare: bool,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSInterfaceBody<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub body: Vec<'a, TSSignature<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSPropertySignature<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub computed: bool,
    pub optional: bool,
    pub readonly: bool,
    pub key: PropertyKey<'a>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged, rename_all = "camelCase")]
pub enum TSSignature<'a> {
    TSIndexSignature(Box<'a, TSIndexSignature<'a>>) = 0,
    TSPropertySignature(Box<'a, TSPropertySignature<'a>>) = 1,
    TSCallSignatureDeclaration(Box<'a, TSCallSignatureDeclaration<'a>>) = 2,
    TSConstructSignatureDeclaration(Box<'a, TSConstructSignatureDeclaration<'a>>) = 3,
    TSMethodSignature(Box<'a, TSMethodSignature<'a>>) = 4,
}

/// An index signature within a class, type alias, etc.
///
/// ## Example
/// [playground link](https://oxc-project.github.io/oxc/playground/?code=3YCAAIC9gICAgICAgIC6nsrEgtem3AB/pQsrWlLnujiFhkHVtfeFMq5RMD7X5AzJnZ5R/ecQ5KG1FUFjzXvrxFXH0m6HpS+Ob3TC8gQXeRQygA%3D%3D)
/// ```ts
/// type MapOf<T> = {
/// //   _________ parameters (vec with 1 element)
///     [K: string]: T
/// //               - type_annotation
/// }
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSIndexSignature<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub parameters: Vec<'a, TSIndexSignatureName<'a>>,
    pub type_annotation: Box<'a, TSTypeAnnotation<'a>>,
    pub readonly: bool,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSCallSignatureDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub this_param: Option<TSThisParameter<'a>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum TSMethodSignatureKind {
    Method = 0,
    Get = 1,
    Set = 2,
}

#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSMethodSignature<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub computed: bool,
    pub optional: bool,
    pub kind: TSMethodSignatureKind,
    pub this_param: Option<TSThisParameter<'a>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSConstructSignatureDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[serde(tag = "type", rename = "Identifier", rename_all = "camelCase")]
pub struct TSIndexSignatureName<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: Atom<'a>,
    pub type_annotation: Box<'a, TSTypeAnnotation<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSInterfaceHeritage<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTypePredicate<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub parameter_name: TSTypePredicateName<'a>,
    pub asserts: bool,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged, rename_all = "camelCase")]
pub enum TSTypePredicateName<'a> {
    Identifier(Box<'a, IdentifierName<'a>>) = 0,
    This(TSThisType) = 1,
}

#[ast(visit)]
#[scope(
    flags(ScopeFlags::TsModuleBlock),
    strict_if(self.body.as_ref().is_some_and(TSModuleDeclarationBody::is_strict)),
)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSModuleDeclaration<'a> {
    #[serde(flatten)]
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

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum TSModuleDeclarationKind {
    Global = 0,
    Module = 1,
    Namespace = 2,
}

impl TSModuleDeclarationKind {
    pub fn is_global(self) -> bool {
        matches!(self, TSModuleDeclarationKind::Global)
    }
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum TSModuleDeclarationName<'a> {
    Identifier(IdentifierName<'a>) = 0,
    StringLiteral(StringLiteral<'a>) = 1,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum TSModuleDeclarationBody<'a> {
    TSModuleDeclaration(Box<'a, TSModuleDeclaration<'a>>) = 0,
    TSModuleBlock(Box<'a, TSModuleBlock<'a>>) = 1,
}

// See serializer in serialize.rs
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSModuleBlock<'a> {
    #[serde(flatten)]
    pub span: Span,
    #[serde(skip)]
    pub directives: Vec<'a, Directive<'a>>,
    pub body: Vec<'a, Statement<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTypeLiteral<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub members: Vec<'a, TSSignature<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSInferType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub type_parameter: Box<'a, TSTypeParameter<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTypeQuery<'a> {
    #[serde(flatten)]
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
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum TSTypeQueryExprName<'a> {
    TSImportType(Box<'a, TSImportType<'a>>) = 2,
    // `TSTypeName` variants added here by `inherit_variants!` macro
    @inherit TSTypeName
}
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSImportType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub is_type_of: bool, // `typeof import("foo")`
    pub parameter: TSType<'a>,
    pub qualifier: Option<TSTypeName<'a>>,
    pub attributes: Option<TSImportAttributes<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSImportAttributes<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub attributes_keyword: IdentifierName<'a>, // `with` or `assert`
    pub elements: Vec<'a, TSImportAttribute<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSImportAttribute<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: TSImportAttributeName<'a>,
    pub value: Expression<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum TSImportAttributeName<'a> {
    Identifier(IdentifierName<'a>) = 0,
    StringLiteral(StringLiteral<'a>) = 1,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSFunctionType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub this_param: Option<TSThisParameter<'a>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Box<'a, TSTypeAnnotation<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSConstructorType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub r#abstract: bool,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Box<'a, TSTypeAnnotation<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSMappedType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub type_parameter: Box<'a, TSTypeParameter<'a>>,
    pub name_type: Option<TSType<'a>>,
    pub type_annotation: Option<TSType<'a>>,
    pub optional: TSMappedTypeModifierOperator,
    pub readonly: TSMappedTypeModifierOperator,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum TSMappedTypeModifierOperator {
    True = 0,
    #[serde(rename = "+")]
    Plus = 1,
    #[serde(rename = "-")]
    Minus = 2,
    None = 3,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTemplateLiteralType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub quasis: Vec<'a, TemplateElement<'a>>,
    pub types: Vec<'a, TSType<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSAsExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSSatisfiesExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSTypeAssertion<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSImportEqualsDeclaration<'a> {
    #[serde(flatten)]
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
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged, rename_all = "camelCase")]
pub enum TSModuleReference<'a> {
    ExternalModuleReference(Box<'a, TSExternalModuleReference<'a>>) = 2,
    // `TSTypeName` variants added here by `inherit_variants!` macro
    @inherit TSTypeName
}
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSExternalModuleReference<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: StringLiteral<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSNonNullExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// Decorator
///
/// Decorators are annotations on classes, methods, properties, and parameters.
/// They are usually either an [`IdentifierReference`] or an [`CallExpression`].
///
/// ## Example
/// ```ts
/// @Foo                        // class decorator
/// @Bar()                      // class decorator factory
/// class SomeClass {
///     @Freeze                 // property decorator
///     public x: number;
///
///     @MethodDecorator        // method decorator
///     public method(
///         @LogParam x: number // parameter decorator
///     ) {
///       // ...
///     }
/// }
/// ```
///
/// [`IdentifierReference`]: crate::ast::js::IdentifierReference
/// [`CallExpression`]: crate::ast::js::CallExpression
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Decorator<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// Export Assignment in non-module files
///
/// `export = foo`
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSExportAssignment<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// Namespace Export Declaration in declaration files
///
/// `export as namespace foo`
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSNamespaceExportDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub id: IdentifierName<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TSInstantiationExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Box<'a, TSTypeParameterInstantiation<'a>>,
}

/// See [TypeScript - Type-Only Imports and Exports](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-8.html)
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum ImportOrExportKind {
    /// `import { foo } from './foo'`;
    Value = 0,
    /// `import type { foo } from './foo'`;
    Type = 1,
}

// [`JSDoc`](https://github.com/microsoft/TypeScript/blob/54a554d8af2657630307cbfa8a3e4f3946e36507/src/compiler/types.ts#L393)

/// `type foo = ty?` or `type foo = ?ty`
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct JSDocNullableType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub type_annotation: TSType<'a>,
    /// Was `?` after the type annotation?
    pub postfix: bool,
}

/// `type foo = ty!` or `type foo = !ty`
#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct JSDocNonNullableType<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub type_annotation: TSType<'a>,
    pub postfix: bool,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct JSDocUnknownType {
    #[serde(flatten)]
    pub span: Span,
}
