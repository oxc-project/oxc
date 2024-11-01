//! TypeScript Definitions
//!
//! - [AST Spec](https://github.com/typescript-eslint/typescript-eslint/tree/main/packages/ast-spec)
//! - [Archived TypeScript spec](https://github.com/microsoft/TypeScript/blob/3c99d50da5a579d9fa92d02664b1b66d4ff55944/doc/spec-ARCHIVED.md)
#![allow(missing_docs)] // FIXME

// NB: `#[span]`, `#[scope(...)]`,`#[visit(...)]` and `#[generate_derive(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in `tasks/ast_tools` and `crates/oxc_traverse/scripts`. See docs in those crates.
// Read [`macro@oxc_ast_macros::ast`] for more information.

use std::cell::Cell;

use oxc_allocator::{Box, CloneIn, Vec};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_span::{cmp::ContentEq, hash::ContentHash, Atom, GetSpan, GetSpanMut, Span};
use oxc_syntax::scope::ScopeId;

use super::{inherit_variants, js::*, jsx::*, literal::*};

/// TypeScript `this` parameter
///
/// ## Example
/// ```ts
/// type T = (this: string, a: number) => void
/// //        ^^^^^^^^^^^^
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - `this` parameters](https://www.typescriptlang.org/docs/handbook/2/functions.html#this-parameters)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSThisParameter<'a> {
    pub span: Span,
    #[estree(skip)]
    pub this_span: Span,
    /// Type type the `this` keyword will have in the function
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
///
/// ## Reference
/// * [TypeScript Handbook - Enums](https://www.typescriptlang.org/docs/handbook/enums.html)
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSEnumDeclaration<'a> {
    pub span: Span,
    pub id: BindingIdentifier<'a>,
    #[scope(enter_before)]
    pub members: Vec<'a, TSEnumMember<'a>>,
    /// `true` for const enums
    pub r#const: bool,
    pub declare: bool,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

/// Enum Member
///
/// A member property in a [`TSEnumDeclaration`].
///
/// ## Example
/// ```ts
/// enum Foo {
/// //  _ id
///     A = 1,
/// //      ^ initializer
///     B // initializer will be `None`
///
/// }
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Enums](https://www.typescriptlang.org/docs/handbook/enums.html)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSEnumMember<'a> {
    pub span: Span,
    pub id: TSEnumMemberName<'a>,
    pub initializer: Option<Expression<'a>>,
}

inherit_variants! {
/// TS Enum Member Name
///
/// Used in [`TSEnumMember`]. Inherits variants from [`Expression`]. See [`ast` module docs] for
/// explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
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
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTypeAnnotation<'a> {
    /// starts at the `:` token and ends at the end of the type annotation
    pub span: Span,
    /// The actual type in the annotation
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
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSLiteralType<'a> {
    pub span: Span,
    pub literal: TSLiteral<'a>,
}

/// A literal in a [`TSLiteralType`].
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
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
///
/// ## Examples
/// ```ts
/// // Foo is a TSTypeAlias
/// type Foo = number | string
/// //         ^^^^^^^^^^^^^^^ TSType::TSUnionType
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
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
/// ```ts
/// type GetProperty<T extends string> =
/// //  _ check_type
///     T extends `${string}.${infer U}`  // <- extends_type
///         ? U                           // <- true_type
///         : never;                      // <- false_type
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Conditional Types](https://www.typescriptlang.org/docs/handbook/2/conditional-types.html)
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSConditionalType<'a> {
    pub span: Span,
    /// The type before `extends` in the test expression.
    pub check_type: TSType<'a>,
    /// The type `check_type` is being tested against.
    #[scope(enter_before)]
    pub extends_type: TSType<'a>,
    /// The type evaluated to if the test is true.
    pub true_type: TSType<'a>,
    /// The type evaluated to if the test is false.
    #[scope(exit_before)]
    pub false_type: TSType<'a>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

/// TypeScript Union Type
///
/// ## Example
/// ```ts
///  string | string[] | (() => string) | { s: string }
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Union Types](https://www.typescriptlang.org/docs/handbook/typescript-in-5-minutes-func.html#unions)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSUnionType<'a> {
    pub span: Span,
    /// The types in the union.
    pub types: Vec<'a, TSType<'a>>,
}

/// TypeScript Intersection Type
///
/// ## Example
/// ```ts
/// type Colorful = { color: string };
/// type Circle = { radius: number };
///
/// // `types` will be `[Colorful, Circle]`
/// type ColorfulCircle = Colorful & Circle;
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Intersection Types](https://www.typescriptlang.org/docs/handbook/2/objects.html#intersection-types)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSIntersectionType<'a> {
    pub span: Span,
    pub types: Vec<'a, TSType<'a>>,
}

/// Parenthesized Type
///
/// Like [`ParenthesizedExpression`], but for types.
///
/// ## Example
/// ```ts
/// type Foo = (string | number);
/// //          ^^^^^^^^^^^^^^^^ type_annotation
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSParenthesizedType<'a> {
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
/// ## References
/// * [TypeScript Handbook - Keyof Types](https://www.typescriptlang.org/docs/handbook/2/keyof-types.html)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTypeOperator<'a> {
    pub span: Span,
    pub operator: TSTypeOperatorOperator,
    /// The type being operated on
    pub type_annotation: TSType<'a>,
}

/// Operator in a [`TSTypeOperator`].
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
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
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSArrayType<'a> {
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
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSIndexedAccessType<'a> {
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
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTupleType<'a> {
    pub span: Span,
    pub element_types: Vec<'a, TSTupleElement<'a>>,
}

/// TypeScript Named Tuple Member
///
/// ## Example
/// ```ts
/// type Foo = [first: string, second: number];
/// //          ^^^^^^^^^^^^^
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Tuple Types](https://www.typescriptlang.org/docs/handbook/2/objects.html#tuple-types)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSNamedTupleMember<'a> {
    pub span: Span,
    pub element_type: TSTupleElement<'a>,
    pub label: IdentifierName<'a>,
    pub optional: bool,
}

/// TypeScript Optional Type
///
/// Note that this does not cover optional object or class properties.
///
/// ## Example
/// ```ts
/// type Foo = [number?]
/// //          ^^^^^^ type_annotation
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSOptionalType<'a> {
    pub span: Span,
    pub type_annotation: TSType<'a>,
}

/// TypeScript Rest Type
///
/// ## Example
/// ```ts
/// //                  ___________ this is the rest type
/// type Foo = [number, ...string[]]
/// //                     ^^^^^^^^ type_annotation
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSRestType<'a> {
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
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub enum TSTupleElement<'a> {
    // Discriminants start at 64, so that `TSTupleElement::is_ts_type` is a single
    // bitwise AND operation on the discriminant (`discriminant & 63 != 0`).
    TSOptionalType(Box<'a, TSOptionalType<'a>>) = 64,
    TSRestType(Box<'a, TSRestType<'a>>) = 65,
    // `TSType` variants added here by `inherit_variants!` macro
    @inherit TSType
}
}

/// TypeScript `any` keyword
///
/// ## Example
/// ```ts
/// type Foo = any;
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Any Type](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#any)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSAnyKeyword {
    pub span: Span,
}

/// TypeScript `string` keyword
///
/// ## Example
/// ```ts
/// type Foo = string;
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Everyday Types](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#the-primitives-string-number-and-boolean)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSStringKeyword {
    pub span: Span,
}

/// TypeScript `boolean` keyword
///
/// ## Example
/// ```ts
/// type Foo = boolean;
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Everyday Types](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#the-primitives-string-number-and-boolean)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSBooleanKeyword {
    pub span: Span,
}

/// TypeScript `number` keyword
///
/// ## Example
/// ```ts
/// type Foo = boolean;
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Everyday Types](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#the-primitives-string-number-and-boolean)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSNumberKeyword {
    pub span: Span,
}

/// TypeScript `never` Keyword
///
/// ## Example
/// ```ts
/// type Foo<T> = T extends string ? never : T;
/// //                               ^^^^^
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Advanced Topics](https://www.typescriptlang.org/docs/handbook/type-compatibility.html#advanced-topics)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSNeverKeyword {
    pub span: Span,
}

/// TypeScript `intrinsic` Keyword
///
/// Intrinsic types are built into TypeScript and are not user-defined.
/// ## Example
/// `type Uppercase<T extends character> = intrinsic;`
///
/// ### References
/// * [TypeScript Handbook - Intrinsic String Manipulation
/// Types](https://www.typescriptlang.org/docs/handbook/2/template-literal-types.html#intrinsic-string-manipulation-types)
/// * [microsoft/TypeScript #40580](https://github.com/microsoft/TypeScript/pull/40580)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSIntrinsicKeyword {
    pub span: Span,
}

/// TypeScript `unknown` Keyword
///
/// This is like `any`, but is not assignable to anything except `any` and `unknown`.
///
/// ## Example
/// ```ts
/// type Foo = unknown;
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Advanced Topics](https://www.typescriptlang.org/docs/handbook/type-compatibility.html#advanced-topics)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSUnknownKeyword {
    pub span: Span,
}

/// TypeScript `null` Keyword
///
/// ## Example
/// ```ts
/// type Foo = string | null;
/// //                  ^^^^
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Everyday Types](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#null-and-undefined)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSNullKeyword {
    pub span: Span,
}

/// TypeScript `null` Keyword
///
/// ## Example
/// ```ts
/// type Foo = string | undefined;
/// //                  ^^^^^^^^^
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Everyday Types](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#null-and-undefined)
/// ## Reference
/// * [TypeScript Handbook - Everyday Types](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#null-and-undefined)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSUndefinedKeyword {
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSVoidKeyword {
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSSymbolKeyword {
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSThisType {
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSObjectKeyword {
    pub span: Span,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSBigIntKeyword {
    pub span: Span,
}

/// TypeScript Type Reference
///
/// ## Example
/// ```ts
/// type C = A;
/// type D = B.a;
/// type E = D.c.b.a;
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTypeReference<'a> {
    pub span: Span,
    pub type_name: TSTypeName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// TypeName:
///     IdentifierReference
///     NamespaceName . IdentifierReference
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
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

/// TypeScript Qualified Name
///
/// A [type reference](TSTypeReference) qualified by a namespace.
///
/// ## Example
/// ```ts
/// type Foo = A.B.C;
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSQualifiedName<'a> {
    pub span: Span,
    pub left: TSTypeName<'a>,
    pub right: IdentifierName<'a>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTypeParameterInstantiation<'a> {
    pub span: Span,
    pub params: Vec<'a, TSType<'a>>,
}

/// TypeScript Type Parameter
///
/// This is a type parameter in a generic type or function.
///
/// ## Example
/// ```ts
/// //                 ______ constraint
/// type Box<T extends string = 'foo'> = { value: T };
/// // name  ^                  ^^^^^ default
///
/// function add<in T>(a: T, b: T): T { return a + b; }
/// //           ^^ in: true
/// ```
///
/// ## References
/// * [TypeScript Handbook - Generics](https://www.typescriptlang.org/docs/handbook/2/generics.html)
/// * [TypeScript Handbook - Variance Annotations](https://www.typescriptlang.org/docs/handbook/2/generics.html#variance-annotations)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTypeParameter<'a> {
    pub span: Span,
    /// The name of the parameter, e.g. `T` in `type Foo<T> = ...`.
    pub name: BindingIdentifier<'a>,
    /// Constrains what types can be passed to the type parameter.
    pub constraint: Option<TSType<'a>>,
    /// Default value of the type parameter if no type is provided when using the type.
    pub default: Option<TSType<'a>>,
    /// Was an `in` modifier keyword present?
    pub r#in: bool,
    /// Was an `out` modifier keyword present?
    pub out: bool,
    /// Was a `const` modifier keyword present?
    pub r#const: bool,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTypeParameterDeclaration<'a> {
    pub span: Span,
    pub params: Vec<'a, TSTypeParameter<'a>>,
}

/// TypeScript Type Alias Declaration Statement
///
/// ## Example
/// ```ts
/// //   _____ id
/// type Maybe<T> = T | null | undefined;
/// //         ^ type_parameters
/// ```
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTypeAliasDeclaration<'a> {
    pub span: Span,
    /// Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    pub id: BindingIdentifier<'a>,
    #[scope(enter_before)]
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub type_annotation: TSType<'a>,
    pub declare: bool,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
pub enum TSAccessibility {
    Private = 0,
    Protected = 1,
    Public = 2,
}

/// TypeScript Class Interface Heritage
///
/// `implements` clause of a [class declaration](Class).
///
/// ## Example
/// ```ts
/// //                   ___ expression
/// class Foo implements Bar, Baz<number, string> {}
/// //            type_parameters ^^^^^^^^^^^^^^
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSClassImplements<'a> {
    pub span: Span,
    pub expression: TSTypeName<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// TypeScriptInterface Declaration
///
///   interface `BindingIdentifier` `TypeParameters_opt` `InterfaceExtendsClause_opt` `ObjectType`
///
/// ## Example
/// ```ts
/// //                       ___ extends
/// interface Foo<T> extends Bar {
/// //     id ^^^ ^ type_parameters
/// }
/// ```
///
/// ## References
/// * [TypeScript in 5 Minutes - Interfaces](https://www.typescriptlang.org/docs/handbook/typescript-tooling-in-5-minutes.html#interfaces)
/// * [TypeScript Handbook - Interfaces](https://www.typescriptlang.org/docs/handbook/2/objects.html#interfaces)
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSInterfaceDeclaration<'a> {
    pub span: Span,
    /// The identifier (name) of the interface.
    pub id: BindingIdentifier<'a>,
    /// Other interfaces/types this interface extends.
    #[scope(enter_before)]
    pub extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
    /// Type parameters that get bound to the interface.
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub body: Box<'a, TSInterfaceBody<'a>>,
    /// `true` for `declare interface Foo {}`
    pub declare: bool,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

/// Body of a [`TSInterfaceDeclaration`].
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSInterfaceBody<'a> {
    pub span: Span,
    pub body: Vec<'a, TSSignature<'a>>,
}

/// TypeScript Property Signature
///
/// Used in [classes](Class), [interfaces](TSInterfaceDeclaration), [mapped types](TSMappedType),
/// etc. Part of a [`TSSignature`].
///
/// ## Example
/// ```ts
/// interface Foo {
/// //  ___ key
///     bar: number
/// //     ^^^^^^^^ type_annotation
///     baz?: string          // <- optional
///     readony bang: boolean // <- readonly
/// }
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSPropertySignature<'a> {
    pub span: Span,
    pub computed: bool,
    pub optional: bool,
    pub readonly: bool,
    pub key: PropertyKey<'a>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
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
/// [playground link](https://oxc-playground.netlify.app/?code=3YCAAIC9gICAgICAgIC6nsrEgtem3AB/pQsrWlLnujiFhkHVtfeFMq5RMD7X5AzJnZ5R/ecQ5KG1FUFjzXvrxFXH0m6HpS+Ob3TC8gQXeRQygA%3D%3D)
/// ```ts
/// type MapOf<T> = {
/// //   _________ parameters (vec with 1 element)
///     [K: string]: T
/// //               - type_annotation
/// }
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSIndexSignature<'a> {
    pub span: Span,
    pub parameters: Vec<'a, TSIndexSignatureName<'a>>,
    pub type_annotation: Box<'a, TSTypeAnnotation<'a>>,
    pub readonly: bool,
    pub r#static: bool,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSCallSignatureDeclaration<'a> {
    pub span: Span,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub this_param: Option<TSThisParameter<'a>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
pub enum TSMethodSignatureKind {
    Method = 0,
    Get = 1,
    Set = 2,
}

/// TypeScript Method Signature
///
/// Similar to a [`TSFunctionType`], but only for method shorthand syntax.
///
/// ## Example
/// ```ts
/// interface Foo {
///     bar(a: number): string;
/// //  ^^^ key
/// }
/// ```
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSMethodSignature<'a> {
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub computed: bool,
    pub optional: bool,
    pub kind: TSMethodSignatureKind,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub this_param: Option<Box<'a, TSThisParameter<'a>>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

/// TypeScript Constructor Signature Declaration
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSConstructSignatureDeclaration<'a> {
    pub span: Span,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(type = "Identifier")]
pub struct TSIndexSignatureName<'a> {
    pub span: Span,
    pub name: Atom<'a>,
    pub type_annotation: Box<'a, TSTypeAnnotation<'a>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSInterfaceHeritage<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// TypeScript Type Predicate
///
/// ## Examples
/// ```ts
/// function isString(x: unknown): x is string {
/// //              parameter_name ^    ^^^^^^ type_annotation
///     return typeof x === 'string';
/// }
/// ```
///
/// ```ts
/// function assertString(x: unknown): asserts x is string {
/// //                                 ^^^^^^^ asserts: true
///     if (typeof x !== 'string') throw new TypeError('x is not a string');
/// }
/// ```
///
/// ## References
/// * [TypeScript Handbook - Type Predicates](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#using-type-predicates)
/// * [TypeScript Handbook - Assertion Functions](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-7.html#assertion-functions)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTypePredicate<'a> {
    pub span: Span,
    /// The identifier the predicate operates on
    pub parameter_name: TSTypePredicateName<'a>,
    /// Does this predicate include an `asserts` modifier?
    ///
    /// ## Example
    /// ```ts
    /// declare function isString(x: any): asserts x is string; // true
    /// ```
    pub asserts: bool,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub enum TSTypePredicateName<'a> {
    Identifier(Box<'a, IdentifierName<'a>>) = 0,
    This(TSThisType) = 1,
}

/// TypeScript Module and Namespace Declarations
///
/// ## Examples
/// ```ts
/// declare module 'foo' {
/// // kind ^^^^^^ ^^^^^ id
/// }
/// ```
///
/// ```ts
/// namespace Foo { }
/// declare namespace Bar { }
/// ```
///
/// ```ts
/// declare global {
///     interface Window {
///        customProp: string;
///     }
/// }
/// ```
///
/// ## References
/// * [TypeScript Handbook - Namespaces](https://www.typescriptlang.org/docs/handbook/2/modules.html#namespaces)
/// * [TypeScript Handbook - Module Augmentation](https://www.typescriptlang.org/docs/handbook/declaration-merging.html#module-augmentation)
/// * [TypeScript Handbook - Global Augmentation](https://www.typescriptlang.org/docs/handbook/declaration-merging.html#global-augmentation)
#[ast(visit)]
#[scope(
    flags(ScopeFlags::TsModuleBlock),
    strict_if(self.body.as_ref().is_some_and(TSModuleDeclarationBody::is_strict)),
)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSModuleDeclaration<'a> {
    pub span: Span,
    /// The name of the module/namespace being declared.
    ///
    /// Note that for `declare global {}`, no symbol will be created for the module name.
    pub id: TSModuleDeclarationName<'a>,
    #[scope(enter_before)]
    pub body: Option<TSModuleDeclarationBody<'a>>,
    /// The keyword used to define this module declaration.
    ///
    /// Helps discriminate between global overrides vs module declarations vs namespace
    /// declarations.
    ///
    /// ```ts
    /// namespace Foo {}
    /// ^^^^^^^^^
    /// module 'foo' {}
    /// ^^^^^^
    /// declare global {}
    ///         ^^^^^^
    /// ```
    pub kind: TSModuleDeclarationKind,
    pub declare: bool,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
pub enum TSModuleDeclarationKind {
    /// `declare global {}`
    Global = 0,
    /// `declare module 'foo' {}`
    Module = 1,
    /// `namespace Foo {}`
    Namespace = 2,
}

/// The name of a TypeScript [namespace or module declaration](TSModuleDeclaration).
///
/// Note that it is a syntax error for namespace declarations to have a string literal name.
/// Modules may have either kind.
///
/// ## Examples
/// ```ts
/// // TSModuleDeclarationName::StringLiteral
/// declare module "*.css" {
///     const styles: { [key: string]: string };
///     export default styles;
/// }
/// ```
///
/// ```ts
/// // TSModuleDeclarationName::Identifier
/// namespace Foo {
///    export const bar = 42;
/// }
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub enum TSModuleDeclarationName<'a> {
    Identifier(BindingIdentifier<'a>) = 0,
    StringLiteral(StringLiteral<'a>) = 1,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub enum TSModuleDeclarationBody<'a> {
    TSModuleDeclaration(Box<'a, TSModuleDeclaration<'a>>) = 0,
    TSModuleBlock(Box<'a, TSModuleBlock<'a>>) = 1,
}

// See serializer in serialize.rs
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
#[estree(custom_serialize)]
pub struct TSModuleBlock<'a> {
    pub span: Span,
    #[estree(skip)]
    pub directives: Vec<'a, Directive<'a>>,
    pub body: Vec<'a, Statement<'a>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTypeLiteral<'a> {
    pub span: Span,
    pub members: Vec<'a, TSSignature<'a>>,
}

/// TypeScript `infer` type
///
/// Used in a [`TSConditionalType`] to bind a type parameter when some tested type extends a
/// desired type.
///
/// ## Example
/// ```ts
/// type Foo<T> = T extends infer U ? U : never;
/// //                            ^ type_parameter
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Inferring With Conditional Types](https://www.typescriptlang.org/docs/handbook/2/conditional-types.html#inferring-within-conditional-types)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSInferType<'a> {
    pub span: Span,
    /// The type bound when the
    pub type_parameter: Box<'a, TSTypeParameter<'a>>,
}

/// Type Query
///
/// ## Example
/// ```ts
/// type Foo = typeof Bar;
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Typeof Type Operator](https://www.typescriptlang.org/docs/handbook/2/typeof-types.html)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTypeQuery<'a> {
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
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub enum TSTypeQueryExprName<'a> {
    TSImportType(Box<'a, TSImportType<'a>>) = 2,
    // `TSTypeName` variants added here by `inherit_variants!` macro
    @inherit TSTypeName
}
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSImportType<'a> {
    pub span: Span,
    /// `true` for `typeof import("foo")`
    pub is_type_of: bool,
    pub parameter: TSType<'a>,
    pub qualifier: Option<TSTypeName<'a>>,
    pub attributes: Option<Box<'a, TSImportAttributes<'a>>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSImportAttributes<'a> {
    pub span: Span,
    pub attributes_keyword: IdentifierName<'a>, // `with` or `assert`
    pub elements: Vec<'a, TSImportAttribute<'a>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSImportAttribute<'a> {
    pub span: Span,
    pub name: TSImportAttributeName<'a>,
    pub value: Expression<'a>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub enum TSImportAttributeName<'a> {
    Identifier(IdentifierName<'a>) = 0,
    StringLiteral(StringLiteral<'a>) = 1,
}

/// TypeScript Function Type
///
/// ## Examples
/// ```ts
/// //       __________ this is the TSFunctionType
/// type T = () => void
/// //             ^^^^ return_type
/// ```
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSFunctionType<'a> {
    pub span: Span,
    /// Generic type parameters
    ///
    /// ```ts
    /// type T = <U>(x: U) => U;
    /// //        ^
    /// ```
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    /// `this` parameter
    ///
    /// ```ts
    /// type T = (this: string, a: number) => void;
    /// //        ^^^^^^^^^^^^
    /// ```
    pub this_param: Option<Box<'a, TSThisParameter<'a>>>,
    /// Function parameters. Akin to [`Function::params`].
    pub params: Box<'a, FormalParameters<'a>>,
    /// Return type of the function.
    /// ```ts
    /// type T = () => void;
    /// //             ^^^^
    /// ```
    pub return_type: Box<'a, TSTypeAnnotation<'a>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSConstructorType<'a> {
    pub span: Span,
    pub r#abstract: bool,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Box<'a, TSTypeAnnotation<'a>>,
}

/// TypeScript Mapped Type
///
/// ## Examples
/// ```ts
/// type Maybe<T> = {
/// //        _____ constraint
///     [P in keyof T]?: T[P]
/// //   ^ type_parameter
/// }
/// ```
///
/// ```ts
/// type ReadonlyDefinite<T> = {
/// //           _ type parameter
///    readonly [P in keyof T]-?: T[P]
/// //                        ^^ `optional` modifier
/// };
/// ```
///
/// ## References
/// * [TypeScript Handbook - Mapped Types](https://www.typescriptlang.org/docs/handbook/2/mapped-types.html)
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSMappedType<'a> {
    pub span: Span,
    /// Key type parameter, e.g. `P` in `[P in keyof T]`.
    pub type_parameter: Box<'a, TSTypeParameter<'a>>,
    pub name_type: Option<TSType<'a>>,
    pub type_annotation: Option<TSType<'a>>,
    /// Optional modifier on type annotation
    ///
    /// ## Examples
    /// ```ts
    /// type Foo = { [P in keyof T]?: T[P] }
    /// //                         ^^ True
    /// type Bar = { [P in keyof T]+?: T[P] }
    /// //                         ^^ Plus
    /// type Baz = { [P in keyof T]-?: T[P] }
    /// //                         ^^ Minus
    /// type Qux = { [P in keyof T]: T[P] }
    /// //                         ^ None
    /// ```
    pub optional: TSMappedTypeModifierOperator,
    /// Readonly modifier before keyed index signature
    ///
    /// ## Examples
    /// ```ts
    /// type Foo = { readonly [P in keyof T]: T[P] }  // True
    /// type Bar = { +readonly [P in keyof T]: T[P] } // Plus
    /// type Baz = { -readonly [P in keyof T]: T[P] } // Minus
    /// type Qux = { [P in keyof T]: T[P] }           // None
    /// ```
    pub readonly: TSMappedTypeModifierOperator,
    #[estree(skip)]
    #[clone_in(default)]
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
pub enum TSMappedTypeModifierOperator {
    /// e.g. `?` in `{ [P in K]?: T }`
    True = 0,
    /// e.g. `+?` in `{ [P in K]+?: T }`
    #[estree(rename = "+")]
    Plus = 1,
    /// e.g. `-?` in `{ [P in K]-?: T }`
    #[estree(rename = "-")]
    Minus = 2,
    /// No modifier present
    None = 3,
}

/// TypeScript Template Literal Type
///
/// ## Example
/// ```ts
/// // Each string part is an element in `quasis`, including empty strings at the beginning/end.
/// // In this example, `quasis` has 3 elements: ["", ".", ""]
/// type Dot<T, U> = `${T}.${U}`;
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - Template Literal Types](https://www.typescriptlang.org/docs/handbook/2/template-literal-types.html#handbook-content)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTemplateLiteralType<'a> {
    pub span: Span,
    /// The string parts of the template literal.
    pub quasis: Vec<'a, TemplateElement<'a>>,
    /// The interpolated expressions in the template literal.
    pub types: Vec<'a, TSType<'a>>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSAsExpression<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

/// TypeScript `satisfies` Expression
///
/// ## Example
/// ```ts
/// const user = {
///     id: 0,
///     name: 'Alice',
/// } satisfies User;
/// ```
///
/// ## Reference
/// * [TypeScript Handbook - The `satisfies` Operator](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-9.html#the-satisfies-operator)
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSSatisfiesExpression<'a> {
    pub span: Span,
    /// The value expression being constrained.
    pub expression: Expression<'a>,
    /// The type `expression` must satisfy.
    pub type_annotation: TSType<'a>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSTypeAssertion<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_annotation: TSType<'a>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSImportEqualsDeclaration<'a> {
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
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub enum TSModuleReference<'a> {
    ExternalModuleReference(Box<'a, TSExternalModuleReference<'a>>) = 2,
    // `TSTypeName` variants added here by `inherit_variants!` macro
    @inherit TSTypeName
}
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSExternalModuleReference<'a> {
    pub span: Span,
    pub expression: StringLiteral<'a>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSNonNullExpression<'a> {
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
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct Decorator<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}

/// Export Assignment in non-module files
///
/// `export = foo`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSExportAssignment<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
}

/// Namespace Export Declaration in declaration files
///
/// `export as namespace foo`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSNamespaceExportDeclaration<'a> {
    pub span: Span,
    pub id: IdentifierName<'a>,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct TSInstantiationExpression<'a> {
    pub span: Span,
    pub expression: Expression<'a>,
    pub type_parameters: Box<'a, TSTypeParameterInstantiation<'a>>,
}

/// See [TypeScript - Type-Only Imports and Exports](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-8.html)
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[generate_derive(CloneIn, ContentEq, ContentHash, ESTree)]
pub enum ImportOrExportKind {
    /// `import { foo } from './foo'`;
    Value = 0,
    /// `import type { foo } from './foo'`;
    Type = 1,
}

// [`JSDoc`](https://github.com/microsoft/TypeScript/blob/54a554d8af2657630307cbfa8a3e4f3946e36507/src/compiler/types.ts#L393)

/// `type foo = ty?` or `type foo = ?ty`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSDocNullableType<'a> {
    pub span: Span,
    pub type_annotation: TSType<'a>,
    /// Was `?` after the type annotation?
    pub postfix: bool,
}

/// `type foo = ty!` or `type foo = !ty`
#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSDocNonNullableType<'a> {
    pub span: Span,
    pub type_annotation: TSType<'a>,
    pub postfix: bool,
}

#[ast(visit)]
#[derive(Debug)]
#[generate_derive(CloneIn, GetSpan, GetSpanMut, ContentEq, ContentHash, ESTree)]
pub struct JSDocUnknownType {
    pub span: Span,
}
