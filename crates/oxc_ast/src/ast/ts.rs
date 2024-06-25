//! TypeScript Definitions
//!
//! [AST Spec](https://github.com/typescript-eslint/typescript-eslint/tree/main/packages/ast-spec)
//! [Archived TypeScript spec](https://github.com/microsoft/TypeScript/blob/3c99d50da5a579d9fa92d02664b1b66d4ff55944/doc/spec-ARCHIVED.md)

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use oxc_allocator::{Box, Vec};
use oxc_macros::ast_node;
use oxc_span::{Atom, GetSpan, Span};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use super::{inherit_variants, js::*, jsx::*, literal::*};
use crate::dummy;
use crate::traverse::{ast::*, GCell, SharedBox, SharedVec};

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface TSIndexSignatureName extends Span {
    type: "Identifier",
    name: Atom,
    typeAnnotation: TSTypeAnnotation,
}
"#;

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSThisParameter<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub this: Box<'a, IdentifierName<'a>>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

/// Enum Declaration
///
/// `const_opt` enum `BindingIdentifier` { `EnumBody_opt` }
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSEnumDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: Box<'a, BindingIdentifier<'a>>,
    pub members: Vec<'a, TSEnumMember<'a>>,
    /// Valid Modifiers: `const`, `export`, `declare`
    pub modifiers: Modifiers<'a>,
}

#[ast_node]
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
/// Inherits variants from [`Expression`].
#[ast_node]
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

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeAnnotation<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSLiteralType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub literal: TSLiteral<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged, rename_all = "camelCase"))]
pub enum TSLiteral<'a> {
    BooleanLiteral(Box<'a, BooleanLiteral>),
    NullLiteral(Box<'a, NullLiteral>),
    NumericLiteral(Box<'a, NumericLiteral<'a>>),
    BigintLiteral(Box<'a, BigIntLiteral<'a>>),
    RegExpLiteral(Box<'a, RegExpLiteral<'a>>),
    StringLiteral(Box<'a, StringLiteral<'a>>),
    TemplateLiteral(Box<'a, TemplateLiteral<'a>>),
    UnaryExpression(Box<'a, UnaryExpression<'a>>),
}

#[ast_node]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged, rename_all = "camelCase"))]
pub enum TSType<'a> {
    // Keyword
    TSAnyKeyword(Box<'a, TSAnyKeyword>) = 0,
    TSBigIntKeyword(Box<'a, TSBigIntKeyword>) = 1,
    TSBooleanKeyword(Box<'a, TSBooleanKeyword>) = 2,
    TSNeverKeyword(Box<'a, TSNeverKeyword>) = 3,
    TSNullKeyword(Box<'a, TSNullKeyword>) = 4,
    TSNumberKeyword(Box<'a, TSNumberKeyword>) = 5,
    TSObjectKeyword(Box<'a, TSObjectKeyword>) = 6,
    TSStringKeyword(Box<'a, TSStringKeyword>) = 7,
    TSSymbolKeyword(Box<'a, TSSymbolKeyword>) = 8,
    TSThisType(Box<'a, TSThisType>) = 9,
    TSUndefinedKeyword(Box<'a, TSUndefinedKeyword>) = 10,
    TSUnknownKeyword(Box<'a, TSUnknownKeyword>) = 11,
    TSVoidKeyword(Box<'a, TSVoidKeyword>) = 12,
    // Compound
    TSArrayType(Box<'a, TSArrayType<'a>>) = 13,
    TSConditionalType(Box<'a, TSConditionalType<'a>>) = 14,
    TSConstructorType(Box<'a, TSConstructorType<'a>>) = 15,
    TSFunctionType(Box<'a, TSFunctionType<'a>>) = 16,
    TSImportType(Box<'a, TSImportType<'a>>) = 17,
    TSIndexedAccessType(Box<'a, TSIndexedAccessType<'a>>) = 18,
    TSInferType(Box<'a, TSInferType<'a>>) = 19,
    TSIntersectionType(Box<'a, TSIntersectionType<'a>>) = 20,
    TSLiteralType(Box<'a, TSLiteralType<'a>>) = 21,
    TSMappedType(Box<'a, TSMappedType<'a>>) = 22,
    TSNamedTupleMember(Box<'a, TSNamedTupleMember<'a>>) = 23,
    TSQualifiedName(Box<'a, TSQualifiedName<'a>>) = 24,
    TSTemplateLiteralType(Box<'a, TSTemplateLiteralType<'a>>) = 25,
    TSTupleType(Box<'a, TSTupleType<'a>>) = 26,
    TSTypeLiteral(Box<'a, TSTypeLiteral<'a>>) = 27,
    TSTypeOperatorType(Box<'a, TSTypeOperator<'a>>) = 28,
    TSTypePredicate(Box<'a, TSTypePredicate<'a>>) = 29,
    TSTypeQuery(Box<'a, TSTypeQuery<'a>>) = 30,
    TSTypeReference(Box<'a, TSTypeReference<'a>>) = 31,
    TSUnionType(Box<'a, TSUnionType<'a>>) = 32,
    // JSDoc
    JSDocNullableType(Box<'a, JSDocNullableType<'a>>) = 33,
    JSDocUnknownType(Box<'a, JSDocUnknownType>) = 34,
}

/// Macro for matching `TSType`'s variants.
#[macro_export]
macro_rules! match_ts_type {
    ($ty:ident) => {
        $ty::TSAnyKeyword(_)
            | $ty::TSBigIntKeyword(_)
            | $ty::TSBooleanKeyword(_)
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
            | $ty::JSDocUnknownType(_)
    };
}
pub use match_ts_type;

impl<'a> TSType<'a> {
    pub fn is_const_type_reference(&self) -> bool {
        matches!(self, TSType::TSTypeReference(reference) if reference.type_name.is_const())
    }

    /// Check if type maybe `undefined`
    pub fn is_maybe_undefined(&self) -> bool {
        match self {
            TSType::TSUndefinedKeyword(_) => true,
            TSType::TSUnionType(un) => {
                un.types.iter().any(|t| matches!(t, TSType::TSUndefinedKeyword(_)))
            }
            _ => false,
        }
    }
}

/// `SomeType extends OtherType ? TrueType : FalseType;`
///
/// <https://www.typescriptlang.org/docs/handbook/2/conditional-types.html#handbook-content>
#[ast_node]
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
#[ast_node]
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
#[ast_node]
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
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeOperator<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub operator: TSTypeOperatorOperator,
    pub type_annotation: TSType<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, layout_inspect::Inspect)]
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
#[ast_node]
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
#[ast_node]
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
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTupleType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub element_types: Vec<'a, TSTupleElement<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSNamedTupleMember<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub element_type: TSType<'a>,
    pub label: Box<'a, IdentifierName<'a>>,
    pub optional: bool,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSOptionalType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

#[ast_node]
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
/// Inherits variants from [`TSType`].
#[ast_node]
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

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSAnyKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSStringKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSBooleanKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSNumberKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSNeverKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSUnknownKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSNullKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSUndefinedKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSVoidKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSSymbolKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSThisType {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct TSObjectKeyword {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[ast_node]
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
#[ast_node]
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
#[ast_node]
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

impl<'a> TSTypeName<'a> {
    pub fn get_first_name(name: &TSTypeName<'a>) -> IdentifierReference<'a> {
        match name {
            TSTypeName::IdentifierReference(name) => (*name).clone(),
            TSTypeName::QualifiedName(name) => TSTypeName::get_first_name(&name.left),
            // TODO: consider adding a Dummy trait to sugar this into `T::Dummy()`
            TSTypeName::Dummy => IdentifierReference::new(Span::default(), Atom::from("")),
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
            TSTypeName::Dummy => dummy!(),
        }
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSQualifiedName<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub left: TSTypeName<'a>,
    pub right: Box<'a, IdentifierName<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeParameterInstantiation<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub params: Vec<'a, TSType<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeParameter<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: Box<'a, BindingIdentifier<'a>>,
    pub constraint: Option<TSType<'a>>,
    pub default: Option<TSType<'a>>,
    pub r#in: bool,
    pub out: bool,
    pub r#const: bool,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeParameterDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub params: Vec<'a, TSTypeParameter<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeAliasDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: Box<'a, BindingIdentifier<'a>>,
    pub type_annotation: TSType<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    /// Valid Modifiers: `declare`, `export`
    pub modifiers: Modifiers<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, layout_inspect::Inspect)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum TSAccessibility {
    Private,
    Protected,
    Public,
}

#[ast_node]
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
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInterfaceDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: Box<'a, BindingIdentifier<'a>>,
    pub body: Box<'a, TSInterfaceBody<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
    /// Valid Modifiers: `export`, `default`, `declare`
    pub modifiers: Modifiers<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInterfaceBody<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, TSSignature<'a>>,
}

#[ast_node]
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

#[ast_node]
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

#[ast_node]
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

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSCallSignatureDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub this_param: Option<Box<'a, TSThisParameter<'a>>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, layout_inspect::Inspect)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum TSMethodSignatureKind {
    Method,
    Get,
    Set,
}

#[ast_node]
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
    pub this_param: Option<Box<'a, TSThisParameter<'a>>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[ast_node]
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

#[ast_node]
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

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInterfaceHeritage<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[ast_node]
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

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged, rename_all = "camelCase"))]
pub enum TSTypePredicateName<'a> {
    Identifier(Box<'a, IdentifierName<'a>>),
    This(Box<'a, TSThisType>),
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSModuleDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: TSModuleDeclarationName<'a>,
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
    /// Valid Modifiers: `declare`, `export`
    pub modifiers: Modifiers<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, layout_inspect::Inspect)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum TSModuleDeclarationKind {
    Global,
    Module,
    Namespace,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum TSModuleDeclarationName<'a> {
    Identifier(Box<'a, IdentifierName<'a>>),
    StringLiteral(Box<'a, StringLiteral<'a>>),
}

impl<'a> TSModuleDeclarationName<'a> {
    pub fn as_atom(&self) -> Atom<'a> {
        match self {
            Self::Identifier(ident) => ident.name.clone(),
            Self::StringLiteral(lit) => lit.value.clone(),
            Self::Dummy => Atom::from("Dummy TSModuleDeclarationName"),
        }
    }
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum TSModuleDeclarationBody<'a> {
    TSModuleDeclaration(Box<'a, TSModuleDeclaration<'a>>),
    TSModuleBlock(Box<'a, TSModuleBlock<'a>>),
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSModuleBlock<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub body: Vec<'a, Statement<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeLiteral<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub members: Vec<'a, TSSignature<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInferType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_parameter: Box<'a, TSTypeParameter<'a>>,
}

#[ast_node]
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
/// Inherits variants from [`TSTypeName`].
#[ast_node]
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

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSImportType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub argument: TSType<'a>,
    pub qualifier: Option<TSTypeName<'a>>,
    pub attributes: Option<Box<'a, TSImportAttributes<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSImportAttributes<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub elements: Vec<'a, TSImportAttribute<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSImportAttribute<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: TSImportAttributeName<'a>,
    pub value: Expression<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum TSImportAttributeName<'a> {
    Identifier(Box<'a, IdentifierName<'a>>),
    StringLiteral(Box<'a, StringLiteral<'a>>),
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSFunctionType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub this_param: Option<Box<'a, TSThisParameter<'a>>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Box<'a, TSTypeAnnotation<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
}

#[ast_node]
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

#[ast_node]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, layout_inspect::Inspect)]
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

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTemplateLiteralType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub quasis: Vec<'a, TemplateElement<'a>>,
    pub types: Vec<'a, TSType<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSAsExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSSatisfiesExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSTypeAssertion<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSImportEqualsDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: Box<'a, BindingIdentifier<'a>>,
    pub module_reference: TSModuleReference<'a>,
    pub import_kind: ImportOrExportKind,
}

inherit_variants! {
/// TS Module Reference
///
/// Inherits variants from [`TSTypeName`].
#[ast_node]
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

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSExternalModuleReference<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Box<'a, StringLiteral<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSNonNullExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct Decorator<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

impl<'a> Decorator<'a> {
    /// Get the name of the decorator
    /// ```ts
    /// @decorator
    /// @decorator.a.b
    /// @decorator(xx)
    /// @decorator.a.b(xx)
    /// The name of the decorator is `decorator`
    /// ```
    pub fn name(&self) -> Option<&str> {
        match &self.expression {
            Expression::Identifier(ident) => Some(&ident.name),
            expr @ match_member_expression!(Expression) => {
                expr.to_member_expression().static_property_name()
            }
            Expression::CallExpression(call) => {
                call.callee.get_member_expr().map(|member| member.static_property_name())?
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, layout_inspect::Inspect)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
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

impl ModifierKind {
    pub fn is_typescript_syntax(&self) -> bool {
        !matches!(self, Self::Async | Self::Default | Self::Export | Self::Static)
    }
}

#[derive(Debug, Hash, layout_inspect::Inspect)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct Modifier {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub kind: ModifierKind,
}

#[derive(Debug, Default, Hash, layout_inspect::Inspect)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(transparent))]
pub struct Modifiers<'a> {
    inner: Option<Vec<'a, Modifier>>,
}

impl<'a> Modifiers<'a> {
    pub fn new(modifiers: Vec<'a, Modifier>) -> Self {
        Self { inner: Some(modifiers) }
    }

    pub fn empty() -> Self {
        Self { inner: None }
    }

    pub fn is_none(&self) -> bool {
        self.inner.is_none()
    }

    pub fn contains(&self, target: ModifierKind) -> bool {
        self.inner
            .as_ref()
            .map_or(false, |modifiers| modifiers.iter().any(|modifier| modifier.kind == target))
    }

    pub fn is_contains_declare(&self) -> bool {
        self.contains(ModifierKind::Declare)
    }

    pub fn remove_type_modifiers(&mut self) {
        if let Some(list) = &mut self.inner {
            list.retain(|m| !m.kind.is_typescript_syntax());
        }
    }
}

/// Export Assignment in non-module files
///
/// `export = foo`
#[ast_node]
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
#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSNamespaceExportDeclaration<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub id: Box<'a, IdentifierName<'a>>,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct TSInstantiationExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Box<'a, TSTypeParameterInstantiation<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, layout_inspect::Inspect)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(rename_all = "camelCase"))]
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

// [`JSDoc`](https://github.com/microsoft/TypeScript/blob/54a554d8af2657630307cbfa8a3e4f3946e36507/src/compiler/types.ts#L393)

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct JSDocNullableType<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub type_annotation: TSType<'a>,
    pub postfix: bool,
}

#[ast_node]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct JSDocUnknownType {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}
