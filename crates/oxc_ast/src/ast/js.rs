// NB: `#[span]`, `#[scope(...)]` and `#[visit(...)]` do NOT do anything to the code.
// They are purely markers for codegen used in
// `tasks/ast_codegen` and `crates/oxc_traverse/scripts`. See docs in those crates.

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use std::cell::Cell;

use oxc_allocator::{Box, Vec};
use oxc_ast_macros::ast;
use oxc_span::{Atom, SourceType, Span};
use oxc_syntax::{
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
    },
    reference::{ReferenceFlag, ReferenceId},
    scope::ScopeId,
    symbol::SymbolId,
};

use super::macros::inherit_variants;
use super::*;

#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

/// Represents the root of a JavaScript abstract syntax tree (AST), containing metadata about the source, directives, top-level statements, and scope information.
#[ast(visit)]
#[scope(
    flags(ScopeFlags::Top),
    strict_if(self.source_type.is_strict() || self.directives.iter().any(Directive::is_use_strict)),
)]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Program<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub source_type: SourceType,
    pub hashbang: Option<Hashbang<'a>>,
    pub directives: Vec<'a, Directive<'a>>,
    pub body: Vec<'a, Statement<'a>>,
    pub scope_id: Cell<Option<ScopeId>>,
}

inherit_variants! {
/// Represents a type for AST nodes corresponding to JavaScript's expressions.
///
/// Inherits variants from [`MemberExpression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum Expression<'a> {
    /// See [`BooleanLiteral`] for ast node details.
    BooleanLiteral(Box<'a, BooleanLiteral>) = 0,
    /// See [`NullLiteral`] for ast node details.
    NullLiteral(Box<'a, NullLiteral>) = 1,
    /// See [`NumericLiteral`] for ast node details.
    NumericLiteral(Box<'a, NumericLiteral<'a>>) = 2,
    /// See [`BigIntLiteral`] for ast node details.
    BigIntLiteral(Box<'a, BigIntLiteral<'a>>) = 3,
    /// See [`RegExpLiteral`] for ast node details.
    RegExpLiteral(Box<'a, RegExpLiteral<'a>>) = 4,
    /// See [`StringLiteral`] for ast node details.
    StringLiteral(Box<'a, StringLiteral<'a>>) = 5,
    /// See [`TemplateLiteral`] for ast node details.
    TemplateLiteral(Box<'a, TemplateLiteral<'a>>) = 6,

    /// See [`IdentifierReference`] for ast node details.
    Identifier(Box<'a, IdentifierReference<'a>>) = 7,

    /// See [`MetaProperty`] for ast node details.
    MetaProperty(Box<'a, MetaProperty<'a>>) = 8,
    /// See [`Super`] for ast node details.
    Super(Box<'a, Super>) = 9,

    /// See [`ArrayExpression`] for ast node details.
    ArrayExpression(Box<'a, ArrayExpression<'a>>) = 10,
    /// See [`ArrowFunctionExpression`] for ast node details.
    ArrowFunctionExpression(Box<'a, ArrowFunctionExpression<'a>>) = 11,
    /// See [`AssignmentExpression`] for ast node details.
    AssignmentExpression(Box<'a, AssignmentExpression<'a>>) = 12,
    /// See [`AwaitExpression`] for ast node details.
    AwaitExpression(Box<'a, AwaitExpression<'a>>) = 13,
    /// See [`BinaryExpression`] for ast node details.
    BinaryExpression(Box<'a, BinaryExpression<'a>>) = 14,
    /// See [`CallExpression`] for ast node details.
    CallExpression(Box<'a, CallExpression<'a>>) = 15,
    /// See [`ChainExpression`] for ast node details.
    ChainExpression(Box<'a, ChainExpression<'a>>) = 16,
    /// See [`Class`] for ast node details.
    ClassExpression(Box<'a, Class<'a>>) = 17,
    /// See [`ConditionalExpression`] for ast node details.
    ConditionalExpression(Box<'a, ConditionalExpression<'a>>) = 18,
    /// See [`Function`] for ast node details.
    #[visit(args(flags = ScopeFlags::Function))]
    FunctionExpression(Box<'a, Function<'a>>) = 19,
    /// See [`ImportExpression`] for ast node details.
    ImportExpression(Box<'a, ImportExpression<'a>>) = 20,
    /// See [`LogicalExpression`] for ast node details.
    LogicalExpression(Box<'a, LogicalExpression<'a>>) = 21,
    /// See [`NewExpression`] for ast node details.
    NewExpression(Box<'a, NewExpression<'a>>) = 22,
    /// See [`ObjectExpression`] for ast node details.
    ObjectExpression(Box<'a, ObjectExpression<'a>>) = 23,
    /// See [`ParenthesizedExpression`] for ast node details.
    ParenthesizedExpression(Box<'a, ParenthesizedExpression<'a>>) = 24,
    /// See [`SequenceExpression`] for ast node details.
    SequenceExpression(Box<'a, SequenceExpression<'a>>) = 25,
    /// See [`TaggedTemplateExpression`] for ast node details.
    TaggedTemplateExpression(Box<'a, TaggedTemplateExpression<'a>>) = 26,
    /// See [`ThisExpression`] for ast node details.
    ThisExpression(Box<'a, ThisExpression>) = 27,
    /// See [`UnaryExpression`] for ast node details.
    UnaryExpression(Box<'a, UnaryExpression<'a>>) = 28,
    /// See [`UpdateExpression`] for ast node details.
    UpdateExpression(Box<'a, UpdateExpression<'a>>) = 29,
    /// See [`YieldExpression`] for ast node details.
    YieldExpression(Box<'a, YieldExpression<'a>>) = 30,
    /// See [`PrivateInExpression`] for ast node details.
    PrivateInExpression(Box<'a, PrivateInExpression<'a>>) = 31,

    /// See [`JSXElement`] for ast node details.
    JSXElement(Box<'a, JSXElement<'a>>) = 32,
    /// See [`JSXFragment`] for ast node details.
    JSXFragment(Box<'a, JSXFragment<'a>>) = 33,

    /// See [`TSAsExpression`] for ast node details.
    TSAsExpression(Box<'a, TSAsExpression<'a>>) = 34,
    /// See [`TSSatisfiesExpression`] for ast node details.
    TSSatisfiesExpression(Box<'a, TSSatisfiesExpression<'a>>) = 35,
    /// See [`TSTypeAssertion`] for ast node details.
    TSTypeAssertion(Box<'a, TSTypeAssertion<'a>>) = 36,
    /// See [`TSNonNullExpression`] for ast node details.
    TSNonNullExpression(Box<'a, TSNonNullExpression<'a>>) = 37,
    /// See [`TSInstantiationExpression`] for ast node details.
    TSInstantiationExpression(Box<'a, TSInstantiationExpression<'a>>) = 38,

    // `MemberExpression` variants added here by `inherit_variants!` macro
    @inherit MemberExpression
}
}

/// Macro for matching `Expression`'s variants.
/// Includes `MemberExpression`'s variants.
#[macro_export]
macro_rules! match_expression {
    ($ty:ident) => {
        $ty::BooleanLiteral(_)
            | $ty::NullLiteral(_)
            | $ty::NumericLiteral(_)
            | $ty::BigIntLiteral(_)
            | $ty::RegExpLiteral(_)
            | $ty::StringLiteral(_)
            | $ty::TemplateLiteral(_)
            | $ty::Identifier(_)
            | $ty::MetaProperty(_)
            | $ty::Super(_)
            | $ty::ArrayExpression(_)
            | $ty::ArrowFunctionExpression(_)
            | $ty::AssignmentExpression(_)
            | $ty::AwaitExpression(_)
            | $ty::BinaryExpression(_)
            | $ty::CallExpression(_)
            | $ty::ChainExpression(_)
            | $ty::ClassExpression(_)
            | $ty::ConditionalExpression(_)
            | $ty::FunctionExpression(_)
            | $ty::ImportExpression(_)
            | $ty::LogicalExpression(_)
            | $ty::NewExpression(_)
            | $ty::ObjectExpression(_)
            | $ty::ParenthesizedExpression(_)
            | $ty::SequenceExpression(_)
            | $ty::TaggedTemplateExpression(_)
            | $ty::ThisExpression(_)
            | $ty::UnaryExpression(_)
            | $ty::UpdateExpression(_)
            | $ty::YieldExpression(_)
            | $ty::PrivateInExpression(_)
            | $ty::JSXElement(_)
            | $ty::JSXFragment(_)
            | $ty::TSAsExpression(_)
            | $ty::TSSatisfiesExpression(_)
            | $ty::TSTypeAssertion(_)
            | $ty::TSNonNullExpression(_)
            | $ty::TSInstantiationExpression(_)
            | $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
    };
}
pub use match_expression;

/// `var` in `let var = 1;`
///
/// Fundamental syntactic structure used for naming variables, functions, and properties. It must start with a Unicode letter (including $ and _) and can be followed by Unicode letters, digits, $, or _.
#[ast(visit)]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[serde(tag = "type", rename = "Identifier")]
pub struct IdentifierName<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: Atom<'a>,
}

/// `x` inside `func` in `const x = 0; function func() { console.log(x); }`
///
/// Represents an identifier reference, which is a reference to a variable, function, class, or object.
///
/// See: [13.1 Identifiers](https://tc39.es/ecma262/#sec-identifiers)
#[ast(visit)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[serde(tag = "type", rename = "Identifier")]
pub struct IdentifierReference<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// The name of the identifier being referenced.
    pub name: Atom<'a>,
    /// Reference ID
    ///
    /// Identifies what identifier this refers to, and how it is used. This is
    /// set in the bind step of semantic analysis, and will always be [`None`]
    /// immediately after parsing.
    #[serde(skip)]
    pub reference_id: Cell<Option<ReferenceId>>,
    /// Flags indicating how the reference is used.
    ///
    /// This gets set in the bind step of semantic analysis, and will always be
    /// [`ReferenceFlag::None`] immediately after parsing.
    #[serde(skip)]
    pub reference_flag: ReferenceFlag,
}

/// `x` in `const x = 0;`
///
/// Represents a binding identifier, which is an identifier that is used to declare a variable, function, class, or object.
///
/// See: [13.1 Identifiers](https://tc39.es/ecma262/#sec-identifiers)
#[ast(visit)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[serde(tag = "type", rename = "Identifier")]
pub struct BindingIdentifier<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// The identifier name being bound.
    pub name: Atom<'a>,
    /// Unique identifier for this binding.
    ///
    /// This gets initialized during [`semantic analysis`] in the bind step. If
    /// you choose to skip semantic analysis, this will always be [`None`].
    ///
    /// [`semantic analysis`]: <https://docs.rs/oxc_semantic/latest/oxc_semantic/struct.SemanticBuilder.html>
    #[serde(skip)]
    pub symbol_id: Cell<Option<SymbolId>>,
}

/// `loop` in `loop: while (true) { break loop; }`
///
/// Represents a label identifier, which is an identifier that is used to label a statement.
///
/// See: [13.1 Identifiers](https://tc39.es/ecma262/#sec-identifiers)
#[ast(visit)]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[serde(tag = "type", rename = "Identifier")]
pub struct LabelIdentifier<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: Atom<'a>,
}

/// This Expression
///
/// Corresponds to the `this` keyword.
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ThisExpression {
    #[serde(flatten)]
    pub span: Span,
}

/// `[1, 2, ...[3, 4], null]` in `const array = [1, 2, ...[3, 4], null];`
///
/// Represents an array literal, which can include elements, spread elements, or null values.
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ArrayExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    #[tsify(type = "Array<SpreadElement | Expression | null>")]
    pub elements: Vec<'a, ArrayExpressionElement<'a>>,
    /// Array trailing comma
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Trailing_commas#arrays>
    #[serde(skip)]
    pub trailing_comma: Option<Span>,
}

inherit_variants! {
/// Represents a element in an array literal.
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[serde(untagged)]
pub enum ArrayExpressionElement<'a> {
    /// `...[3, 4]` in `const array = [1, 2, ...[3, 4], null];`
    SpreadElement(Box<'a, SpreadElement<'a>>) = 64,
    /// `<empty>` in `const array = [1, , 2];`
    ///
    /// Array hole for sparse arrays
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Trailing_commas#arrays>
    Elision(Elision) = 65,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// empty slot in `const array = [1, , 2];`
///
/// Array Expression Elision Element
/// Serialized as `null` in JSON AST. See `serialize.rs`.
#[ast(visit)]
#[derive(Debug, Clone, Hash)]
pub struct Elision {
    pub span: Span,
}

/// `{ a: 1 }` in `const obj = { a: 1 };`
///
/// Represents an object literal, which can include properties, spread properties, or computed properties and trailing comma.
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ObjectExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// Properties declared in the object
    pub properties: Vec<'a, ObjectPropertyKind<'a>>,
    #[serde(skip)]
    pub trailing_comma: Option<Span>,
}

/// Represents a property in an object literal.
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum ObjectPropertyKind<'a> {
    /// `a: 1` in `const obj = { a: 1 };`
    ObjectProperty(Box<'a, ObjectProperty<'a>>),
    /// `...{ a: 1 }` in `const obj = { ...{ a: 1 } };`
    SpreadProperty(Box<'a, SpreadElement<'a>>),
}

/// `a: 1` in `const obj = { a: 1 };`
///
/// Represents a property in an object literal.
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ObjectProperty<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub kind: PropertyKind,
    pub key: PropertyKey<'a>,
    pub value: Expression<'a>,
    pub init: Option<Expression<'a>>, // for `CoverInitializedName`
    pub method: bool,
    pub shorthand: bool,
    pub computed: bool,
}

inherit_variants! {
/// Property Key
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum PropertyKey<'a> {
    /// `a` in `class C { static a = 1; }`
    StaticIdentifier(Box<'a, IdentifierName<'a>>) = 64,
    /// `a` in `class C { #a = 1; }`
    PrivateIdentifier(Box<'a, PrivateIdentifier<'a>>) = 65,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// Represents the kind of property in an object literal or class.
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum PropertyKind {
    /// `{ a: 1 }` in `const obj = { a: 1 };`
    Init,
    /// `{ get a() { return 1; } }` in `const obj = { get a() { return 1; } };`
    Get,
    /// `{ set a(value) { this._a = value; } }` in `const obj = { set a(value) { this._a = value; } };`
    Set,
}

/// Template Literal
///
/// This is interpreted by interleaving the expression elements in between the quasi elements.
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TemplateLiteral<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub quasis: Vec<'a, TemplateElement<'a>>,
    pub expressions: Vec<'a, Expression<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct TaggedTemplateExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub tag: Expression<'a>,
    pub quasi: TemplateLiteral<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TemplateElement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub tail: bool,
    pub value: TemplateElementValue<'a>,
}

/// See [template-strings-cooked-vs-raw](https://exploringjs.com/impatient-js/ch_template-literals.html#template-strings-cooked-vs-raw)
#[ast]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct TemplateElementValue<'a> {
    /// A raw interpretation where backslashes do not have special meaning.
    /// For example, \t produces two characters – a backslash and a t.
    /// This interpretation of the template strings is stored in property .raw of the first argument (an Array).
    pub raw: Atom<'a>,
    /// A cooked interpretation where backslashes have special meaning.
    /// For example, \t produces a tab character.
    /// This interpretation of the template strings is stored as an Array in the first argument.
    /// cooked = None when template literal has invalid escape sequence
    pub cooked: Option<Atom<'a>>,
}

/// <https://tc39.es/ecma262/#prod-MemberExpression>
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum MemberExpression<'a> {
    /// `MemberExpression[?Yield, ?Await] [ Expression[+In, ?Yield, ?Await] ]`
    ComputedMemberExpression(Box<'a, ComputedMemberExpression<'a>>) = 48,
    /// `MemberExpression[?Yield, ?Await] . IdentifierName`
    StaticMemberExpression(Box<'a, StaticMemberExpression<'a>>) = 49,
    /// `MemberExpression[?Yield, ?Await] . PrivateIdentifier`
    PrivateFieldExpression(Box<'a, PrivateFieldExpression<'a>>) = 50,
}

/// Macro for matching `MemberExpression`'s variants.
#[macro_export]
macro_rules! match_member_expression {
    ($ty:ident) => {
        $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
    };
}
pub use match_member_expression;

/// `MemberExpression[?Yield, ?Await] [ Expression[+In, ?Yield, ?Await] ]`
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ComputedMemberExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub object: Expression<'a>,
    pub expression: Expression<'a>,
    pub optional: bool, // for optional chaining
}

/// `MemberExpression[?Yield, ?Await] . IdentifierName`
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct StaticMemberExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub object: Expression<'a>,
    pub property: IdentifierName<'a>,
    pub optional: bool, // for optional chaining
}

/// `MemberExpression[?Yield, ?Await] . PrivateIdentifier`
///
/// ## Example
/// ```ts
/// //    _______ object
/// const foo.bar?.#baz
/// //           ↑ ^^^^ field
/// //           optional
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct PrivateFieldExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub object: Expression<'a>,
    pub field: PrivateIdentifier<'a>,
    pub optional: bool, // for optional chaining
}

/// Call Expression
///
/// ## Examples
/// ```ts
/// //        ___ callee
/// const x = foo(1, 2)
///
/// //            ^^^^ arguments
/// const y = foo.bar?.(1, 2)
/// //               ^ optional
///
/// const z = foo<number, string>(1, 2)
/// //            ^^^^^^^^^^^^^^ type_parameters
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct CallExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub arguments: Vec<'a, Argument<'a>>,
    pub callee: Expression<'a>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    pub optional: bool, // for optional chaining
}

/// New Expression
///
/// ## Example
/// ```ts
/// //           callee         arguments
/// //              ↓↓↓         ↓↓↓↓
/// const foo = new Foo<number>(1, 2)
/// //                 ↑↑↑↑↑↑↑↑
/// //                 type_parameters
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct NewExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub callee: Expression<'a>,
    pub arguments: Vec<'a, Argument<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// Meta Property `new.target` | `import.meta`
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct MetaProperty<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub meta: IdentifierName<'a>,
    pub property: IdentifierName<'a>,
}

/// Spread Element
///
/// An array or object spread. Could be used in unpacking or a declaration.
///
/// ## Example
/// ```ts
/// const [first, ...rest] = arr
/// //            ^^^^^^^
/// const obj = { foo: 'foo', ...obj2 }
/// //                        ^^^^^^^
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct SpreadElement<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// The expression being spread.
    pub argument: Expression<'a>,
}

inherit_variants! {
/// Argument
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum Argument<'a> {
    SpreadElement(Box<'a, SpreadElement<'a>>) = 64,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// Update Expression
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct UpdateExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub operator: UpdateOperator,
    pub prefix: bool,
    pub argument: SimpleAssignmentTarget<'a>,
}

/// Unary Expression
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct UnaryExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub operator: UnaryOperator,
    pub argument: Expression<'a>,
}

/// Binary Expression
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct BinaryExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub left: Expression<'a>,
    pub operator: BinaryOperator,
    pub right: Expression<'a>,
}

/// Private Identifier in Shift Expression
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct PrivateInExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub left: PrivateIdentifier<'a>,
    pub operator: BinaryOperator, // BinaryOperator::In
    pub right: Expression<'a>,
}

/// Binary Logical Operators
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct LogicalExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub left: Expression<'a>,
    pub operator: LogicalOperator,
    pub right: Expression<'a>,
}

/// Conditional Expression
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ConditionalExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub test: Expression<'a>,
    pub consequent: Expression<'a>,
    pub alternate: Expression<'a>,
}

/// Assignment Expression
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct AssignmentExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub operator: AssignmentOperator,
    pub left: AssignmentTarget<'a>,
    pub right: Expression<'a>,
}

inherit_variants! {
/// Destructuring Assignment
///
/// Inherits variants from [`SimpleAssignmentTarget`] and [`AssignmentTargetPattern`].
/// See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum AssignmentTarget<'a> {
    // `SimpleAssignmentTarget` variants added here by `inherit_variants!` macro
    @inherit SimpleAssignmentTarget
    // `AssignmentTargetPattern` variants added here by `inherit_variants!` macro
    @inherit AssignmentTargetPattern
}
}

inherit_variants! {
/// Simple Assignment Target
///
/// Inherits variants from [`MemberExpression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum SimpleAssignmentTarget<'a> {
    AssignmentTargetIdentifier(Box<'a, IdentifierReference<'a>>) = 0,
    TSAsExpression(Box<'a, TSAsExpression<'a>>) = 1,
    TSSatisfiesExpression(Box<'a, TSSatisfiesExpression<'a>>) = 2,
    TSNonNullExpression(Box<'a, TSNonNullExpression<'a>>) = 3,
    TSTypeAssertion(Box<'a, TSTypeAssertion<'a>>) = 4,
    TSInstantiationExpression(Box<'a, TSInstantiationExpression<'a>>) = 5,
    // `MemberExpression` variants added here by `inherit_variants!` macro
    @inherit MemberExpression
}
}

/// Macro for matching `AssignmentTarget`'s variants.
/// Includes `SimpleAssignmentTarget`'s and `AssignmentTargetPattern`'s variants.
#[macro_export]
macro_rules! match_assignment_target {
    ($ty:ident) => {
        $ty::AssignmentTargetIdentifier(_)
            | $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
            | $ty::TSAsExpression(_)
            | $ty::TSSatisfiesExpression(_)
            | $ty::TSNonNullExpression(_)
            | $ty::TSTypeAssertion(_)
            | $ty::TSInstantiationExpression(_)
            | $ty::ArrayAssignmentTarget(_)
            | $ty::ObjectAssignmentTarget(_)
    };
}
pub use match_assignment_target;

/// Macro for matching `SimpleAssignmentTarget`'s variants.
/// Includes `MemberExpression`'s variants
#[macro_export]
macro_rules! match_simple_assignment_target {
    ($ty:ident) => {
        $ty::AssignmentTargetIdentifier(_)
            | $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
            | $ty::TSAsExpression(_)
            | $ty::TSSatisfiesExpression(_)
            | $ty::TSNonNullExpression(_)
            | $ty::TSTypeAssertion(_)
            | $ty::TSInstantiationExpression(_)
    };
}
pub use match_simple_assignment_target;

#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum AssignmentTargetPattern<'a> {
    ArrayAssignmentTarget(Box<'a, ArrayAssignmentTarget<'a>>) = 8,
    ObjectAssignmentTarget(Box<'a, ObjectAssignmentTarget<'a>>) = 9,
}

/// Macro for matching `AssignmentTargetPattern`'s variants.
#[macro_export]
macro_rules! match_assignment_target_pattern {
    ($ty:ident) => {
        $ty::ArrayAssignmentTarget(_) | $ty::ObjectAssignmentTarget(_)
    };
}
pub use match_assignment_target_pattern;

// See serializer in serialize.rs
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[serde(tag = "type")]
pub struct ArrayAssignmentTarget<'a> {
    #[serde(flatten)]
    pub span: Span,
    #[tsify(type = "Array<AssignmentTargetMaybeDefault | AssignmentTargetRest | null>")]
    pub elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
    #[serde(skip)]
    pub rest: Option<AssignmentTargetRest<'a>>,
    #[serde(skip)]
    pub trailing_comma: Option<Span>,
}

// See serializer in serialize.rs
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[serde(tag = "type")]
pub struct ObjectAssignmentTarget<'a> {
    #[serde(flatten)]
    pub span: Span,
    #[tsify(type = "Array<AssignmentTargetProperty | AssignmentTargetRest>")]
    pub properties: Vec<'a, AssignmentTargetProperty<'a>>,
    #[serde(skip)]
    pub rest: Option<AssignmentTargetRest<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[serde(tag = "type", rename = "RestElement")]
pub struct AssignmentTargetRest<'a> {
    #[serde(flatten)]
    pub span: Span,
    #[serde(rename = "argument")]
    pub target: AssignmentTarget<'a>,
}

inherit_variants! {
/// Assignment Target Maybe Default
///
/// Inherits variants from [`AssignmentTarget`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum AssignmentTargetMaybeDefault<'a> {
    AssignmentTargetWithDefault(Box<'a, AssignmentTargetWithDefault<'a>>) = 16,
    // `AssignmentTarget` variants added here by `inherit_variants!` macro
    @inherit AssignmentTarget
}
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct AssignmentTargetWithDefault<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub binding: AssignmentTarget<'a>,
    pub init: Expression<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum AssignmentTargetProperty<'a> {
    AssignmentTargetPropertyIdentifier(Box<'a, AssignmentTargetPropertyIdentifier<'a>>),
    AssignmentTargetPropertyProperty(Box<'a, AssignmentTargetPropertyProperty<'a>>),
}

/// Assignment Property - Identifier Reference
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct AssignmentTargetPropertyIdentifier<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub binding: IdentifierReference<'a>,
    pub init: Option<Expression<'a>>,
}

/// Assignment Property - Property Name
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct AssignmentTargetPropertyProperty<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: PropertyKey<'a>,
    pub binding: AssignmentTargetMaybeDefault<'a>,
}

/// Sequence Expression
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct SequenceExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expressions: Vec<'a, Expression<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct Super {
    #[serde(flatten)]
    pub span: Span,
}

/// Await Expression
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct AwaitExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub argument: Expression<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ChainExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: ChainElement<'a>,
}

inherit_variants! {
/// Chain Element
///
/// Inherits variants from [`MemberExpression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum ChainElement<'a> {
    CallExpression(Box<'a, CallExpression<'a>>) = 0,
    // `MemberExpression` variants added here by `inherit_variants!` macro
    @inherit MemberExpression
}
}

/// Parenthesized Expression
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ParenthesizedExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
}

inherit_variants! {
/// Statement
///
/// Inherits variants from [`Declaration`] and [`ModuleDeclaration`].
/// See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum Statement<'a> {
    // Statements
    BlockStatement(Box<'a, BlockStatement<'a>>) = 0,
    BreakStatement(Box<'a, BreakStatement<'a>>) = 1,
    ContinueStatement(Box<'a, ContinueStatement<'a>>) = 2,
    DebuggerStatement(Box<'a, DebuggerStatement>) = 3,
    DoWhileStatement(Box<'a, DoWhileStatement<'a>>) = 4,
    EmptyStatement(Box<'a, EmptyStatement>) = 5,
    ExpressionStatement(Box<'a, ExpressionStatement<'a>>) = 6,
    ForInStatement(Box<'a, ForInStatement<'a>>) = 7,
    ForOfStatement(Box<'a, ForOfStatement<'a>>) = 8,
    ForStatement(Box<'a, ForStatement<'a>>) = 9,
    IfStatement(Box<'a, IfStatement<'a>>) = 10,
    LabeledStatement(Box<'a, LabeledStatement<'a>>) = 11,
    ReturnStatement(Box<'a, ReturnStatement<'a>>) = 12,
    SwitchStatement(Box<'a, SwitchStatement<'a>>) = 13,
    ThrowStatement(Box<'a, ThrowStatement<'a>>) = 14,
    TryStatement(Box<'a, TryStatement<'a>>) = 15,
    WhileStatement(Box<'a, WhileStatement<'a>>) = 16,
    WithStatement(Box<'a, WithStatement<'a>>) = 17,
    // `Declaration` variants added here by `inherit_variants!` macro
    @inherit Declaration
    // `ModuleDeclaration` variants added here by `inherit_variants!` macro
    @inherit ModuleDeclaration
}
}

/// Directive Prologue
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct Directive<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// Directive with any escapes unescaped
    pub expression: StringLiteral<'a>,
    /// Raw content of directive as it appears in source, any escapes left as is
    pub directive: Atom<'a>,
}

/// Hashbang
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct Hashbang<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub value: Atom<'a>,
}

/// Block Statement
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct BlockStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub body: Vec<'a, Statement<'a>>,
    pub scope_id: Cell<Option<ScopeId>>,
}

/// Declarations and the Variable Statement
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum Declaration<'a> {
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>) = 32,
    #[visit(args(flags = ScopeFlags::Function))]
    FunctionDeclaration(Box<'a, Function<'a>>) = 33,
    ClassDeclaration(Box<'a, Class<'a>>) = 34,
    UsingDeclaration(Box<'a, UsingDeclaration<'a>>) = 35,

    TSTypeAliasDeclaration(Box<'a, TSTypeAliasDeclaration<'a>>) = 36,
    TSInterfaceDeclaration(Box<'a, TSInterfaceDeclaration<'a>>) = 37,
    TSEnumDeclaration(Box<'a, TSEnumDeclaration<'a>>) = 38,
    TSModuleDeclaration(Box<'a, TSModuleDeclaration<'a>>) = 39,
    TSImportEqualsDeclaration(Box<'a, TSImportEqualsDeclaration<'a>>) = 40,
}

/// Macro for matching `Declaration`'s variants.
#[macro_export]
macro_rules! match_declaration {
    ($ty:ident) => {
        $ty::VariableDeclaration(_)
            | $ty::FunctionDeclaration(_)
            | $ty::ClassDeclaration(_)
            | $ty::UsingDeclaration(_)
            | $ty::TSTypeAliasDeclaration(_)
            | $ty::TSInterfaceDeclaration(_)
            | $ty::TSEnumDeclaration(_)
            | $ty::TSModuleDeclaration(_)
            | $ty::TSImportEqualsDeclaration(_)
    };
}
pub use match_declaration;

/// Variable Declaration
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct VariableDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub kind: VariableDeclarationKind,
    pub declarations: Vec<'a, VariableDeclarator<'a>>,
    pub declare: bool,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum VariableDeclarationKind {
    Var,
    Const,
    Let,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct VariableDeclarator<'a> {
    #[serde(flatten)]
    pub span: Span,
    #[serde(skip)]
    pub kind: VariableDeclarationKind,
    pub id: BindingPattern<'a>,
    pub init: Option<Expression<'a>>,
    pub definite: bool,
}

/// Using Declaration
/// * <https://github.com/tc39/proposal-explicit-resource-management>
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct UsingDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub is_await: bool,
    #[serde(default)]
    pub declarations: Vec<'a, VariableDeclarator<'a>>,
}

/// Empty Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct EmptyStatement {
    #[serde(flatten)]
    pub span: Span,
}

/// Expression Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ExpressionStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub expression: Expression<'a>,
}

/// If Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct IfStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub test: Expression<'a>,
    pub consequent: Statement<'a>,
    pub alternate: Option<Statement<'a>>,
}

/// Do-While Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct DoWhileStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub body: Statement<'a>,
    pub test: Expression<'a>,
}

/// While Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct WhileStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub test: Expression<'a>,
    pub body: Statement<'a>,
}

/// For Statement
#[ast(visit)]
#[scope(if(self.init.as_ref().is_some_and(ForStatementInit::is_lexical_declaration)))]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ForStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub init: Option<ForStatementInit<'a>>,
    pub test: Option<Expression<'a>>,
    pub update: Option<Expression<'a>>,
    pub body: Statement<'a>,
    pub scope_id: Cell<Option<ScopeId>>,
}

inherit_variants! {
/// For Statement Init
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum ForStatementInit<'a> {
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>) = 64,
    UsingDeclaration(Box<'a, UsingDeclaration<'a>>) = 65,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// For-In Statement
#[ast(visit)]
#[scope(if(self.left.is_lexical_declaration()))]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ForInStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub left: ForStatementLeft<'a>,
    pub right: Expression<'a>,
    pub body: Statement<'a>,
    pub scope_id: Cell<Option<ScopeId>>,
}

inherit_variants! {
/// For Statement Left
///
/// Inherits variants from [`AssignmentTarget`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum ForStatementLeft<'a> {
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>) = 16,
    UsingDeclaration(Box<'a, UsingDeclaration<'a>>) = 17,
    // `AssignmentTarget` variants added here by `inherit_variants!` macro
    @inherit AssignmentTarget
}
}
/// For-Of Statement
#[ast(visit)]
#[scope(if(self.left.is_lexical_declaration()))]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ForOfStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub r#await: bool,
    pub left: ForStatementLeft<'a>,
    pub right: Expression<'a>,
    pub body: Statement<'a>,
    pub scope_id: Cell<Option<ScopeId>>,
}

/// Continue Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ContinueStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub label: Option<LabelIdentifier<'a>>,
}

/// Break Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct BreakStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub label: Option<LabelIdentifier<'a>>,
}

/// Return Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ReturnStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub argument: Option<Expression<'a>>,
}

/// With Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct WithStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub object: Expression<'a>,
    pub body: Statement<'a>,
}

/// Switch Statement
#[ast(visit)]
#[scope]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct SwitchStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub discriminant: Expression<'a>,
    #[scope(enter_before)]
    pub cases: Vec<'a, SwitchCase<'a>>,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct SwitchCase<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub test: Option<Expression<'a>>,
    pub consequent: Vec<'a, Statement<'a>>,
}

/// Labelled Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct LabeledStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub label: LabelIdentifier<'a>,
    pub body: Statement<'a>,
}

/// Throw Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ThrowStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// Try Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct TryStatement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub block: Box<'a, BlockStatement<'a>>,
    pub handler: Option<Box<'a, CatchClause<'a>>>,
    #[visit(as(FinallyClause))]
    pub finalizer: Option<Box<'a, BlockStatement<'a>>>,
}

#[ast(visit)]
#[scope(flags(ScopeFlags::CatchClause), if(self.param.is_some()))]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct CatchClause<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub param: Option<CatchParameter<'a>>,
    pub body: Box<'a, BlockStatement<'a>>,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct CatchParameter<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub pattern: BindingPattern<'a>,
}

/// Debugger Statement
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct DebuggerStatement {
    #[serde(flatten)]
    pub span: Span,
}

/// Destructuring Binding Patterns
/// * <https://tc39.es/ecma262/#prod-BindingPattern>
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub struct BindingPattern<'a> {
    // serde(flatten) the attributes because estree has no `BindingPattern`
    #[serde(flatten)]
    #[tsify(type = "(BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern)")]
    #[span]
    pub kind: BindingPatternKind<'a>,
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub optional: bool,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum BindingPatternKind<'a> {
    /// `const a = 1`
    BindingIdentifier(Box<'a, BindingIdentifier<'a>>),
    /// `const {a} = 1`
    ObjectPattern(Box<'a, ObjectPattern<'a>>),
    /// `const [a] = 1`
    ArrayPattern(Box<'a, ArrayPattern<'a>>),
    /// A defaulted binding pattern, i.e.:
    /// `const {a = 1} = 1`
    /// the assignment pattern is `a = 1`
    /// it has an inner left that has a BindingIdentifier
    AssignmentPattern(Box<'a, AssignmentPattern<'a>>),
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct AssignmentPattern<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub left: BindingPattern<'a>,
    pub right: Expression<'a>,
}

// See serializer in serialize.rs
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[serde(tag = "type")]
pub struct ObjectPattern<'a> {
    #[serde(flatten)]
    pub span: Span,
    #[tsify(type = "Array<BindingProperty | BindingRestElement>")]
    pub properties: Vec<'a, BindingProperty<'a>>,
    #[serde(skip)]
    pub rest: Option<Box<'a, BindingRestElement<'a>>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct BindingProperty<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub key: PropertyKey<'a>,
    pub value: BindingPattern<'a>,
    pub shorthand: bool,
    pub computed: bool,
}

// See serializer in serialize.rs
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[serde(tag = "type")]
pub struct ArrayPattern<'a> {
    #[serde(flatten)]
    pub span: Span,
    #[tsify(type = "Array<BindingPattern | BindingRestElement | null>")]
    pub elements: Vec<'a, Option<BindingPattern<'a>>>,
    #[serde(skip)]
    pub rest: Option<Box<'a, BindingRestElement<'a>>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[serde(tag = "type", rename = "RestElement")]
pub struct BindingRestElement<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub argument: BindingPattern<'a>,
}

/// Function Definitions
#[ast(visit)]
#[scope(
    // `flags` passed in to visitor via parameter defined by `#[visit(args(flags = ...))]` on parents
    flags(flags),
    strict_if(self.is_strict()),
)]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Function<'a> {
    pub r#type: FunctionType,
    #[serde(flatten)]
    pub span: Span,
    pub id: Option<BindingIdentifier<'a>>,
    pub generator: bool,
    pub r#async: bool,
    pub declare: bool,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    /// Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    ///
    /// The JavaScript specification states that you cannot have a parameter called `this`,
    /// and so TypeScript uses that syntax space to let you declare the type for `this` in the function body.
    ///
    /// ```TypeScript
    /// interface DB {
    ///   filterUsers(filter: (this: User) => boolean): User[];
    /// }
    ///
    /// const db = getDB();
    /// const admins = db.filterUsers(function (this: User) {
    ///   return this.admin;
    /// });
    /// ```
    pub this_param: Option<TSThisParameter<'a>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    pub body: Option<Box<'a, FunctionBody<'a>>>,
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum FunctionType {
    FunctionDeclaration,
    FunctionExpression,
    TSDeclareFunction,
    /// <https://github.com/typescript-eslint/typescript-eslint/pull/1289>
    TSEmptyBodyFunctionExpression,
}

/// <https://tc39.es/ecma262/#prod-FormalParameters>
// See serializer in serialize.rs
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Tsify))]
#[serde(tag = "type")]
pub struct FormalParameters<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub kind: FormalParameterKind,
    #[tsify(type = "Array<FormalParameter | FormalParameterRest>")]
    pub items: Vec<'a, FormalParameter<'a>>,
    #[serde(skip)]
    pub rest: Option<Box<'a, BindingRestElement<'a>>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct FormalParameter<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub decorators: Vec<'a, Decorator<'a>>,
    pub pattern: BindingPattern<'a>,
    pub accessibility: Option<TSAccessibility>,
    pub readonly: bool,
    pub r#override: bool,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum FormalParameterKind {
    /// <https://tc39.es/ecma262/#prod-FormalParameters>
    FormalParameter,
    /// <https://tc39.es/ecma262/#prod-UniqueFormalParameters>
    UniqueFormalParameters,
    /// <https://tc39.es/ecma262/#prod-ArrowFormalParameters>
    ArrowFormalParameters,
    /// Part of TypeScript type signatures
    Signature,
}

/// <https://tc39.es/ecma262/#prod-FunctionBody>
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct FunctionBody<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub directives: Vec<'a, Directive<'a>>,
    pub statements: Vec<'a, Statement<'a>>,
}

/// Arrow Function Definitions
#[ast(visit)]
#[scope(
    flags(ScopeFlags::Function | ScopeFlags::Arrow),
    strict_if(self.body.has_use_strict_directive()),
)]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ArrowFunctionExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    pub expression: bool,
    pub r#async: bool,
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    pub params: Box<'a, FormalParameters<'a>>,
    pub return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    /// See `expression` for whether this arrow expression returns an expression.
    pub body: Box<'a, FunctionBody<'a>>,
    pub scope_id: Cell<Option<ScopeId>>,
}

/// Generator Function Definitions
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct YieldExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub delegate: bool,
    pub argument: Option<Expression<'a>>,
}

/// Class Definitions
#[ast(visit)]
#[scope(flags(ScopeFlags::StrictMode))]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Class<'a> {
    pub r#type: ClassType,
    #[serde(flatten)]
    pub span: Span,
    /// Decorators applied to the class.
    ///
    /// Decorators are currently a stage 3 proposal. Oxc handles both TC39 and
    /// legacy TypeScript decorators.
    ///
    /// ## Example
    /// ```ts
    /// @Bar() // <-- Decorator
    /// class Foo {}
    /// ```
    pub decorators: Vec<'a, Decorator<'a>>,
    /// Class identifier, AKA the name
    pub id: Option<BindingIdentifier<'a>>,
    #[scope(enter_before)]
    pub type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    /// Super class. When present, this will usually be an [`IdentifierReference`].
    ///
    /// ## Example
    /// ```ts
    /// class Foo extends Bar {}
    /// //                ^^^
    /// ```
    #[visit(as(ClassHeritage))]
    pub super_class: Option<Expression<'a>>,
    /// Type parameters passed to super class.
    ///
    /// ## Example
    /// ```ts
    /// class Foo<T> extends Bar<T> {}
    /// //                       ^
    /// ```
    pub super_type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    /// Interface implementation clause for TypeScript classes.
    ///
    /// ## Example
    /// ```ts
    /// interface Bar {}
    /// class Foo implements Bar {}
    /// //                   ^^^
    /// ```
    pub implements: Option<Vec<'a, TSClassImplements<'a>>>,
    pub body: Box<'a, ClassBody<'a>>,
    /// Whether the class is abstract
    ///
    /// ## Example
    /// ```ts
    /// class Foo {}          // true
    /// abstract class Bar {} // false
    /// ```
    pub r#abstract: bool,
    /// Whether the class was `declare`ed
    ///
    /// ## Example
    /// ```ts
    /// declare class Foo {}
    /// ```
    pub declare: bool,
    /// Id of the scope created by the [`Class`], including type parameters and
    /// statements within the [`ClassBody`].
    pub scope_id: Cell<Option<ScopeId>>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum ClassType {
    /// Class declaration statement
    /// ```ts
    /// class Foo { }
    /// ```
    ClassDeclaration,
    /// Class expression
    ///
    /// ```ts
    /// const Foo = class {}
    /// ```
    ClassExpression,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ClassBody<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub body: Vec<'a, ClassElement<'a>>,
}

/// Class Body Element
///
/// ## Example
/// ```ts
/// class Foo {
///   [prop: string]: string // ClassElement::TSIndexSignature
///
///   public x: number // ClassElement::PropertyDefinition
///
///   accessor z() { return 5 } // ClassElement::AccessorProperty
///
///   // These are all ClassElement::MethodDefinitions
///   get y() { return 5 }
///   set y(value) { }
///   static foo() {}
///   bar() {}
/// }
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum ClassElement<'a> {
    StaticBlock(Box<'a, StaticBlock<'a>>),
    /// Class Methods
    ///
    /// Includes static and non-static methods, constructors, getters, and setters.
    MethodDefinition(Box<'a, MethodDefinition<'a>>),
    PropertyDefinition(Box<'a, PropertyDefinition<'a>>),
    AccessorProperty(Box<'a, AccessorProperty<'a>>),
    /// Index Signature
    ///
    /// ## Example
    /// ```ts
    /// class Foo {
    ///   [keys: string]: string
    /// }
    /// ```
    TSIndexSignature(Box<'a, TSIndexSignature<'a>>),
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub struct MethodDefinition<'a> {
    /// Method definition type
    ///
    /// This will always be true when an `abstract` modifier is used on the method.
    pub r#type: MethodDefinitionType,
    #[serde(flatten)]
    pub span: Span,
    pub decorators: Vec<'a, Decorator<'a>>,
    pub key: PropertyKey<'a>,
    #[visit(args(flags = match self.kind {
        MethodDefinitionKind::Get => ScopeFlags::Function | ScopeFlags::GetAccessor,
        MethodDefinitionKind::Set => ScopeFlags::Function | ScopeFlags::SetAccessor,
        MethodDefinitionKind::Constructor => ScopeFlags::Function | ScopeFlags::Constructor,
        MethodDefinitionKind::Method => ScopeFlags::Function,
    }))]
    pub value: Box<'a, Function<'a>>, // FunctionExpression
    pub kind: MethodDefinitionKind,
    pub computed: bool,
    pub r#static: bool,
    pub r#override: bool,
    pub optional: bool,
    pub accessibility: Option<TSAccessibility>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum MethodDefinitionType {
    MethodDefinition,
    TSAbstractMethodDefinition,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub struct PropertyDefinition<'a> {
    pub r#type: PropertyDefinitionType,
    #[serde(flatten)]
    pub span: Span,
    /// Decorators applied to the property.
    ///
    /// See [`Decorator`] for more information.
    pub decorators: Vec<'a, Decorator<'a>>,
    /// The expression used to declare the property.
    pub key: PropertyKey<'a>,
    /// Initialized value in the declaration.
    ///
    /// ## Example
    /// ```
    /// class Foo {
    ///   x = 5     // Some(NumericLiteral)
    ///   y: string // None
    ///
    ///   constructor() {
    ///     this.y = "hello"
    ///   }
    /// }
    /// ```
    pub value: Option<Expression<'a>>,
    /// Property was declared with a computed key
    ///
    /// ## Example
    /// ```ts
    /// class Foo {
    ///   ["a"]: string // true
    ///   b: number     // false
    /// }
    /// ```
    pub computed: bool,
    /// Property was declared with a `static` modifier
    pub r#static: bool,
    /// Property is declared with a `declare` modifier.
    ///
    /// ## Example
    /// ```ts
    /// class Foo {
    ///   x: number         // false
    ///   declare y: string // true
    /// }
    ///
    /// declare class Bar {
    ///   x: number         // false
    /// }
    /// ```
    pub declare: bool,
    pub r#override: bool,
    /// `true` when created with an optional modifier (`?`)
    pub optional: bool,
    pub definite: bool,
    /// `true` when declared with a `readonly` modifier
    pub readonly: bool,
    /// Type annotation on the property.
    ///
    /// Will only ever be [`Some`] for TypeScript files.
    pub type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    /// Accessibility modifier.
    ///
    /// Only ever [`Some`] for TypeScript files.
    ///
    /// ## Example
    ///
    /// ```ts
    /// class Foo {
    ///   public w: number     // Some(TSAccessibility::Public)
    ///   private x: string    // Some(TSAccessibility::Private)
    ///   protected y: boolean // Some(TSAccessibility::Protected)
    ///   readonly z           // None
    /// }
    /// ```
    pub accessibility: Option<TSAccessibility>,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum PropertyDefinitionType {
    PropertyDefinition,
    TSAbstractPropertyDefinition,
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum MethodDefinitionKind {
    /// Class constructor
    Constructor,
    /// Static or instance method
    Method,
    /// Getter method
    Get,
    /// Setter method
    Set,
}

/// An identifier for a private class member.
///
/// See: [MDN - Private class fields](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Private_class_fields)
#[ast(visit)]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct PrivateIdentifier<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub name: Atom<'a>,
}

/// Class Static Block
///
/// See: [MDN - Static initialization blocks](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Static_initialization_blocks)
///
/// ## Example
///
/// ```ts
/// class Foo {
///     static {
///         this.someStaticProperty = 5;
///     }
/// }
/// ```
#[ast(visit)]
#[scope(flags(ScopeFlags::ClassStaticBlock))]
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct StaticBlock<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub body: Vec<'a, Statement<'a>>,
    pub scope_id: Cell<Option<ScopeId>>,
}

/// ES6 Module Declaration
///
/// An ESM import or export statement.
///
/// ## Example
///
/// ```ts
/// // ImportDeclaration
/// import { foo } from 'foo';
/// import bar from 'bar';
/// import * as baz from 'baz';
///
/// // Not a ModuleDeclaration
/// export const a = 5;
///
/// const b = 6;
///
/// export { b };             // ExportNamedDeclaration
/// export default b;         // ExportDefaultDeclaration
/// export * as c from './c'; // ExportAllDeclaration
/// export = b;               // TSExportAssignment
/// export as namespace d;    // TSNamespaceExportDeclaration
/// ```
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum ModuleDeclaration<'a> {
    /// `import hello from './world.js';`
    /// `import * as t from './world.js';`
    ImportDeclaration(Box<'a, ImportDeclaration<'a>>) = 64,
    /// `export * as numbers from '../numbers.js'`
    ExportAllDeclaration(Box<'a, ExportAllDeclaration<'a>>) = 65,
    /// `export default 5;`
    ExportDefaultDeclaration(Box<'a, ExportDefaultDeclaration<'a>>) = 66,
    /// `export {five} from './numbers.js';`
    /// `export {six, seven};`
    ExportNamedDeclaration(Box<'a, ExportNamedDeclaration<'a>>) = 67,

    /// `export = 5;`
    TSExportAssignment(Box<'a, TSExportAssignment<'a>>) = 68,
    /// `export as namespace React;`
    TSNamespaceExportDeclaration(Box<'a, TSNamespaceExportDeclaration<'a>>) = 69,
}

/// Macro for matching `ModuleDeclaration`'s variants.
#[macro_export]
macro_rules! match_module_declaration {
    ($ty:ident) => {
        $ty::ImportDeclaration(_)
            | $ty::ExportAllDeclaration(_)
            | $ty::ExportDefaultDeclaration(_)
            | $ty::ExportNamedDeclaration(_)
            | $ty::TSExportAssignment(_)
            | $ty::TSNamespaceExportDeclaration(_)
    };
}
pub use match_module_declaration;

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub enum AccessorPropertyType {
    AccessorProperty,
    TSAbstractAccessorProperty,
}

/// Class Accessor Property
///
/// ## Example
/// ```ts
/// class Foo {
///   accessor y: string
/// }
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
pub struct AccessorProperty<'a> {
    pub r#type: AccessorPropertyType,
    #[serde(flatten)]
    pub span: Span,
    /// Decorators applied to the accessor property.
    ///
    /// See [`Decorator`] for more information.
    pub decorators: Vec<'a, Decorator<'a>>,
    /// The expression used to declare the property.
    pub key: PropertyKey<'a>,
    /// Initialized value in the declaration, if present.
    pub value: Option<Expression<'a>>,
    pub computed: bool,
    pub r#static: bool,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ImportExpression<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub source: Expression<'a>,
    pub arguments: Vec<'a, Expression<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ImportDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    pub specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
    pub source: StringLiteral<'a>,
    /// Some(vec![]) for empty assertion
    pub with_clause: Option<WithClause<'a>>,
    /// `import type { foo } from 'bar'`
    pub import_kind: ImportOrExportKind,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum ImportDeclarationSpecifier<'a> {
    /// import {imported} from "source"
    /// import {imported as local} from "source"
    ImportSpecifier(Box<'a, ImportSpecifier<'a>>),
    /// import local from "source"
    ImportDefaultSpecifier(Box<'a, ImportDefaultSpecifier<'a>>),
    /// import * as local from "source"
    ImportNamespaceSpecifier(Box<'a, ImportNamespaceSpecifier<'a>>),
}

// import {imported} from "source"
// import {imported as local} from "source"
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ImportSpecifier<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub imported: ModuleExportName<'a>,
    /// The name of the imported symbol.
    ///
    /// ## Example
    /// ```ts
    /// // local and imported name are the same
    /// import { Foo } from 'foo';
    /// //       ^^^
    /// // imports can be renamed, changing the local name
    /// import { Foo as Bar } from 'foo';
    /// //              ^^^
    /// ```
    pub local: BindingIdentifier<'a>,
    pub import_kind: ImportOrExportKind,
}

/// Default Import Specifier
///
/// ## Example
/// ```ts
/// import local from "source";
/// ```
///
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ImportDefaultSpecifier<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// The name of the imported symbol.
    pub local: BindingIdentifier<'a>,
}

/// Namespace import specifier
///
/// ## Example
/// ```ts
/// import * as local from "source";
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ImportNamespaceSpecifier<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub local: BindingIdentifier<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct WithClause<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub attributes_keyword: IdentifierName<'a>, // `with` or `assert`
    pub with_entries: Vec<'a, ImportAttribute<'a>>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ImportAttribute<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub key: ImportAttributeKey<'a>,
    pub value: StringLiteral<'a>,
}

#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum ImportAttributeKey<'a> {
    Identifier(IdentifierName<'a>),
    StringLiteral(StringLiteral<'a>),
}

/// Named Export Declaration
///
/// ## Example
///
/// ```ts
/// //       ________ specifiers
/// export { Foo, Bar };
/// export type { Baz } from 'baz';
/// //     ^^^^              ^^^^^
/// // export_kind           source
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ExportNamedDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub declaration: Option<Declaration<'a>>,
    pub specifiers: Vec<'a, ExportSpecifier<'a>>,
    pub source: Option<StringLiteral<'a>>,
    /// `export type { foo }`
    pub export_kind: ImportOrExportKind,
    /// Some(vec![]) for empty assertion
    pub with_clause: Option<WithClause<'a>>,
}

/// Export Default Declaration
///
/// ## Example
///
/// ```ts
/// export default HoistableDeclaration
/// export default ClassDeclaration
/// export default AssignmentExpression
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type")]
pub struct ExportDefaultDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub declaration: ExportDefaultDeclarationKind<'a>,
    pub exported: ModuleExportName<'a>, // the `default` Keyword
}

/// Export All Declaration
///
/// ## Example
///
/// ```ts
/// //          _______ exported
/// export * as numbers from '../numbers.js';
/// //                       ^^^^^^^^^^^^^^^ source
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ExportAllDeclaration<'a> {
    #[serde(flatten)]
    pub span: Span,
    /// If this declaration is re-named
    pub exported: Option<ModuleExportName<'a>>,
    pub source: StringLiteral<'a>,
    /// Will be `Some(vec![])` for empty assertion
    pub with_clause: Option<WithClause<'a>>, // Some(vec![]) for empty assertion
    pub export_kind: ImportOrExportKind, // `export type *`
}

/// Export Specifier
///
/// Each [`ExportSpecifier`] is one of the named exports in an [`ExportNamedDeclaration`].
///
/// ## Example
///
/// ```ts
/// //       ____ export_kind
/// import { type Foo as Bar } from './foo';
/// //   exported ^^^    ^^^ local
/// ```
#[ast(visit)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ExportSpecifier<'a> {
    #[serde(flatten)]
    pub span: Span,
    pub local: ModuleExportName<'a>,
    pub exported: ModuleExportName<'a>,
    pub export_kind: ImportOrExportKind, // `export type *`
}

inherit_variants! {
/// Export Default Declaration Kind
///
/// Inherits variants from [`Expression`]. See [`ast` module docs] for explanation of inheritance.
///
/// [`ast` module docs]: `super`
#[ast(visit)]
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum ExportDefaultDeclarationKind<'a> {
    #[visit(args(flags = ScopeFlags::Function))]
    FunctionDeclaration(Box<'a, Function<'a>>) = 64,
    ClassDeclaration(Box<'a, Class<'a>>) = 65,

    TSInterfaceDeclaration(Box<'a, TSInterfaceDeclaration<'a>>) = 66,

    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

/// Module Export Name
///
/// Supports:
///   * `import {"\0 any unicode" as foo} from ""`
///   * `export {foo as "\0 any unicode"}`
/// * es2022: <https://github.com/estree/estree/blob/master/es2022.md#modules>
/// * <https://github.com/tc39/ecma262/pull/2154>
#[ast(visit)]
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(untagged)]
pub enum ModuleExportName<'a> {
    IdentifierName(IdentifierName<'a>),
    /// For `local` in `ExportSpecifier`: `foo` in `export { foo }`
    IdentifierReference(IdentifierReference<'a>),
    StringLiteral(StringLiteral<'a>),
}
